use std::{
    alloc::Layout,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

#[allow(dead_code)]
pub struct Vec<T> {
    ptr: NonNull<T>,
    len: usize,
    cap: usize,
}

pub struct IntoIter<T> {
    buf: NonNull<T>,
    cap: usize,
    start: *const T,
    end: *const T,
}

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

#[allow(dead_code)]
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

    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len, "Index out of bounds");
        if self.len == self.cap {
            self.grow();
        }
        unsafe {
            std::ptr::copy(
                self.ptr.as_ptr().add(index),
                self.ptr.as_ptr().add(index + 1),
                self.len - index,
            );
            std::ptr::write(self.ptr.as_ptr().add(index), value);
        }
        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "Index out of bounds");
        self.len -= 1;
        unsafe {
            let value = std::ptr::read(self.ptr.as_ptr().add(index));
            std::ptr::copy(
                self.ptr.as_ptr().add(index + 1),
                self.ptr.as_ptr().add(index),
                self.len - index,
            );
            value
        }
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let vec = ManuallyDrop::new(self);
        let ptr = vec.ptr;
        let len = vec.len;
        let cap = vec.cap;
        IntoIter {
            buf: ptr,
            cap,
            start: ptr.as_ptr(),
            end: if cap == 0 {
                // can not use `ptr.as_ptr().add(len)` here because it will panic if len is 0
                ptr.as_ptr()
            } else {
                unsafe { ptr.as_ptr().add(len) }
            },
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }
        let value = unsafe { std::ptr::read(self.start) };
        self.start = unsafe { self.start.add(1) };
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = unsafe { self.end.offset_from(self.start) } as usize;
        (remaining, Some(remaining))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }
        self.end = unsafe { self.end.sub(1) };
        let value = unsafe { std::ptr::read(self.end) };
        Some(value)
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
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

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            for i in 0..self.cap {
                unsafe {
                    std::ptr::drop_in_place(self.buf.as_ptr().add(i));
                }
            }
            unsafe {
                std::alloc::dealloc(
                    self.buf.as_ptr() as *mut u8,
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

    #[test]
    fn test_vec_insert_remove() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec.len, 3);

        vec.insert(1, 4); // Insert 4 at index 1
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 4);
        assert_eq!(vec[2], 2);
        assert_eq!(vec[3], 3);
        assert_eq!(vec.len, 4);

        let removed_value = vec.remove(2); // Remove value at index 2
        assert_eq!(removed_value, 2);
        assert_eq!(vec.len, 3);
    }

    #[test]
    fn test_vec_deref() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);

        let slice: &[i32] = &vec;
        assert_eq!(slice.len(), 3);
        assert_eq!(slice[0], 1);
        assert_eq!(slice[1], 2);
        assert_eq!(slice[2], 3);
    }

    #[test]
    fn test_vec_deref_mut() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);

        let slice: &mut [i32] = &mut vec;
        slice[0] = 10;
        slice[1] = 20;
        slice[2] = 30;

        assert_eq!(vec[0], 10);
        assert_eq!(vec[1], 20);
        assert_eq!(vec[2], 30);
    }

    #[test]
    fn test_into_iter() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let mut iter = vec.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None); // Should return None when empty

        // Test size_hint
        let (lower, upper) = iter.size_hint();
        assert_eq!(lower, 0);
        assert_eq!(upper, Some(0));
    }

    #[test]
    fn test_into_iter_double_ended() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let mut iter = vec.into_iter();
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next_back(), Some(2));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next_back(), None); // Should return None when empty

        // Test size_hint
        let (lower, upper) = iter.size_hint();
        assert_eq!(lower, 0);
        assert_eq!(upper, Some(0));
    }
}
