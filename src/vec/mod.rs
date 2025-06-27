use std::{alloc::Layout, ptr::NonNull};

#[allow(dead_code)]
pub struct Vec<T> {
    ptr: NonNull<T>,
    len: usize,
    cap: usize,
}

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

impl<T> Vec<T> {
    fn new() -> Self {
        Vec {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }

    fn grow(&mut self) {
        if std::mem::size_of::<T>() == 0 {
            panic!("zero-sized types are not supported");
        }
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, std::alloc::Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = self.cap << 1;
            /***
             * `Layout::array` is used to create a layout for an array of `T` with `new_cap` elements.
             * This is necessary because the size of the allocation needs to account for the number of elements
             * being allocated, not just the size of a single element.
             * If `new_cap` is 0, it will panic because `Layout::array` cannot create a layout for an array of zero elements.
             * The `unwrap()` is used to handle the case where the layout cannot be created, which should not happen in this context
             * since `new_cap` is guaranteed to be at least 1.
             * This ensures that the allocation is always valid and can hold at least one element of type `T`.
             *
             * `Layout::array` will check the space allocated if smaller than `usize::MAX` and will panic if it is not.
             * But because old_layout.size() <= isize::MAX as usize, we can safely assume that the new layout will also be valid.
             * so we can safely use `unwrap()` here.
             */
            let new_layout = std::alloc::Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Memory allocation size exceeds isize::MAX"
        );

        let new_ptr = if self.cap == 0 {
            unsafe { std::alloc::alloc(new_layout) }
        } else {
            unsafe {
                std::alloc::realloc(
                    self.ptr.as_ptr() as *mut u8,
                    Layout::array::<T>(self.cap).unwrap(),
                    new_layout.size(),
                )
            }
        };

        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(ptr) => ptr,
            None => std::alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }
        unsafe {
            std::ptr::write(self.ptr.as_ptr().add(self.len), value);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        unsafe {
            let value = std::ptr::read(self.ptr.as_ptr().add(self.len));
            Some(value)
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            unsafe {
                // Drop each element in the vector
                for i in 0..self.len {
                    std::ptr::drop_in_place(self.ptr.as_ptr().add(i));
                }
                // Deallocate the memory
                std::alloc::dealloc(
                    self.ptr.as_ptr() as *mut u8,
                    Layout::array::<T>(self.cap).unwrap(),
                );
            }
        }
    }
}

mod tests {

    #![allow(unused_imports)]
    use std::{isize, mem};

    use super::*;

    #[test]
    fn test_vec_new() {
        let vec: Vec<i32> = Vec::new();
        assert_eq!(vec.len, 0);
        assert_eq!(vec.cap, 0);
    }

    #[test]
    fn test_vec_push_pop() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec.len, 3);
        assert_eq!(vec.cap, 4); // Initial capacity should be 4 after first grow

        assert_eq!(vec.pop(), Some(3));
        assert_eq!(vec.len, 2);
        assert_eq!(vec.cap, 4);

        assert_eq!(vec.pop(), Some(2));
        assert_eq!(vec.len, 1);
        assert_eq!(vec.cap, 4);

        assert_eq!(vec.pop(), Some(1));
        assert_eq!(vec.len, 0);
        assert_eq!(vec.cap, 4);

        assert_eq!(vec.pop(), None); // Should return None when empty
    }

    #[test]
    fn test_vec_grow() {
        let mut vec = Vec::new();
        for i in 0..10 {
            vec.push(i);
        }
        assert_eq!(vec.len, 10);
        assert!(vec.cap >= 10); // Capacity should be at least 10 after multiple pushes
    }

    #[test]
    fn test_vec_drop() {
        struct DropCounter {
            value: i32,
        }
        impl Drop for DropCounter {
            fn drop(&mut self) {
                println!("Dropping {}", self.value);
            }
        }
        {
            let mut vec = Vec::new();
            for i in 0..5 {
                vec.push(DropCounter { value: i });
            }
            assert_eq!(vec.len, 5);
        } // vec goes out of scope here, and DropCounter instances should be dropped
    }
}
