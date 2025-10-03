use std::{
    alloc::Layout,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

// A raw vector that holds a pointer to the allocated memory and its capacity.
// This is a low-level representation of a vector, similar to `Vec<T>` in the standard library.
struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
    _marker: PhantomData<T>,
}

struct RawValIter<T> {
    start: *const T,
    end: *const T,
}

#[allow(dead_code)]
pub struct Vec<T> {
    buf: RawVec<T>,
    len: usize,
}

pub struct Drain<'a, T> {
    vec: PhantomData<&'a mut Vec<T>>,
    iter: RawValIter<T>,
}

pub struct IntoIter<T> {
    _buf: RawVec<T>,
    iter: RawValIter<T>,
}

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

impl<T> RawVec<T> {
    fn new() -> Self {
        RawVec {
            ptr: NonNull::dangling(),
            cap: if std::mem::size_of::<T>() == 0 {
                usize::MAX
            } else {
                0
            },
            _marker: PhantomData,
        }
    }

    fn grow(&mut self) {
        assert!(
            std::mem::size_of::<T>() != 0,
            "Capacity overflow for zero-sized type"
        );
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
}

impl<T> RawValIter<T> {
    /***
     * Creates a new `RawValIter` from a slice is unsafe
     * because it assumes that the slice is valid and that the pointers
     * are not null. If the slice is empty, it will set both `start`
     * and `end` to the same pointer, which is the null pointer.
     *
     * notice that the struct does not have a lifetime parameter,
     * so it can be used with any slice of `T` without requiring
     * a specific lifetime.
     */
    unsafe fn new(slice: &[T]) -> Self {
        RawValIter {
            start: slice.as_ptr(),
            end: if std::mem::size_of::<T>() == 0 {
                ((slice.as_ptr()) as usize + slice.len()) as *const T // For zero-sized types, we use the size of the slice
            } else if slice.is_empty() {
                slice.as_ptr()
            } else {
                unsafe { slice.as_ptr().add(slice.len()) }
            },
        }
    }
}

#[allow(dead_code)]
impl<T> Vec<T> {
    fn new() -> Self {
        Vec {
            buf: RawVec::new(),
            len: 0,
        }
    }

    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    pub fn cap(&self) -> usize {
        self.buf.cap
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap() {
            self.buf.grow();
        }
        unsafe {
            std::ptr::write(self.ptr().add(self.len), value);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        unsafe {
            let value = std::ptr::read(self.ptr().add(self.len));
            Some(value)
        }
    }

    pub fn drain(&mut self) -> Drain<'_, T> {
        let iter = unsafe { RawValIter::new(self) };
        self.len = 0; // Reset length to 0, as Drain will consume the elements
        Drain {
            vec: PhantomData,
            iter,
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len, "Index out of bounds");
        if self.len == self.cap() {
            self.buf.grow();
        }
        unsafe {
            std::ptr::copy(
                self.ptr().add(index),
                self.ptr().add(index + 1),
                self.len - index,
            );
            std::ptr::write(self.ptr().add(index), value);
        }
        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "Index out of bounds");
        self.len -= 1;
        unsafe {
            let value = std::ptr::read(self.ptr().add(index));
            std::ptr::copy(
                self.ptr().add(index + 1),
                self.ptr().add(index),
                self.len - index,
            );
            value
        }
    }
}

impl<T> Iterator for RawValIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else if std::mem::size_of::<T>() == 0 {
            self.start = (self.start as usize + 1) as *const T; // For zero-sized types, we use the size of the slice
            unsafe { Some(std::ptr::read(NonNull::<T>::dangling().as_ptr())) }
        } else {
            let ptr = self.start; // Save the current pointer
            self.start = unsafe { self.start.add(1) };
            unsafe { Some(std::ptr::read(ptr)) }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let elem_size = std::mem::size_of::<T>();
        let len =
            (self.end as usize - self.start as usize) / if elem_size == 0 { 1 } else { elem_size };
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for RawValIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else if std::mem::size_of::<T>() == 0 {
            self.end = (self.end as usize - 1) as *const T; // For zero-sized types, we use the size of the slice
            unsafe { Some(std::ptr::read(NonNull::<T>::dangling().as_ptr())) }
        } else {
            self.end = unsafe { self.end.sub(1) };
            unsafe { Some(std::ptr::read(self.end)) }
        }
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let iter = unsafe { RawValIter::new(&self) };
        let buf = unsafe { ptr::read(&self.buf) };
        std::mem::forget(self); // Prevent Vec from dropping its buffer
        IntoIter { iter, _buf: buf }
    }
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<T> Extend<T> for Vec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();

        if std::mem::size_of::<T>() != 0 {
            // TODO: Handle upper bound as well
            // TODO: single allocation for all elements
            while self.len() + lower > self.cap() {
                self.buf.grow();
            }
        }

        for item in iter {
            if std::mem::size_of::<T>() != 0 && self.len() == self.cap() {
                self.buf.grow();
            }
            unsafe {
                std::ptr::write(self.ptr().add(self.len), item);
            }
            self.len += 1;
        }
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr(), self.len) }
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        let elem_size = std::mem::size_of::<T>();
        if self.cap != 0 && elem_size != 0 {
            unsafe {
                std::alloc::dealloc(
                    self.ptr.as_ptr() as *mut u8,
                    Layout::array::<T>(self.cap).unwrap(),
                );
            }
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        unsafe {
            // Drop each element in the vector
            for i in 0..self.len {
                std::ptr::drop_in_place(self.ptr().add(i));
            }
        }
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        for _ in self.iter.by_ref() {
            // This is to ensure that the elements are dropped
        }
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        // Drop each element in the iterator
        for _ in self.iter.by_ref() {
            // This is to ensure that the elements are dropped
        }
    }
}

#[macro_export]
macro_rules! my_vec {
    () => {
        $crate::vec::Vec::new()
    };
    ($($elem:expr),+ $(,)?) => {
        {
            let mut vec = $crate::vec::Vec::new();
            $(
                vec.push($elem);
            )+
            vec
        }
    };
    ($elem:expr; $n:expr) => {
        {
            let mut vec = $crate::vec::Vec::new();
            vec.extend(std::iter::repeat_n($elem, $n));
            vec
        }
    }
}

mod tests {

    #![allow(unused_imports)]
    use std::mem;

    use super::*;

    #[test]
    fn test_vec_new() {
        let vec: Vec<i32> = Vec::new();
        assert_eq!(vec.len, 0);
        assert_eq!(vec.cap(), 0);
    }

    #[test]
    fn test_vec_push_pop() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec.len, 3);
        assert_eq!(vec.cap(), 4); // Initial capacity should be 4 after first grow

        assert_eq!(vec.pop(), Some(3));
        assert_eq!(vec.len, 2);
        assert_eq!(vec.cap(), 4);

        assert_eq!(vec.pop(), Some(2));
        assert_eq!(vec.len, 1);
        assert_eq!(vec.cap(), 4);

        assert_eq!(vec.pop(), Some(1));
        assert_eq!(vec.len, 0);
        assert_eq!(vec.cap(), 4);

        assert_eq!(vec.pop(), None); // Should return None when empty
    }

    #[test]
    fn test_vec_grow() {
        let mut vec = Vec::new();
        for i in 0..10 {
            vec.push(i);
        }
        assert_eq!(vec.len, 10);
        assert!(vec.cap() >= 10); // Capacity should be at least 10 after multiple pushes
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

    #[test]
    fn test_drain() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);
        vec.push(5);

        {
            let mut drain = vec.drain(); // Drain all elements
            assert_eq!(drain.next(), Some(1));
            assert_eq!(drain.next(), Some(2));
            assert_eq!(drain.next(), Some(3));
            assert_eq!(drain.next(), Some(4));
            assert_eq!(drain.next(), Some(5));
            assert_eq!(drain.next(), None); // Should return None when empty
        }

        // After draining, vec should be empty
        assert_eq!(vec.len, 0);
    }

    #[test]
    fn test_vec_size_hint() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let iter = vec.into_iter();
        let (lower, upper) = iter.size_hint();
        assert_eq!(lower, 3);
        assert_eq!(upper, Some(3));

        // Test with an empty vector
        let empty_vec: Vec<i32> = Vec::new();
        let empty_iter = empty_vec.into_iter();
        let (lower_empty, upper_empty) = empty_iter.size_hint();
        assert_eq!(lower_empty, 0);
        assert_eq!(upper_empty, Some(0));
    }

    #[test]
    fn test_vec_zero_sized_type() {
        #[derive(Debug)]
        struct ZeroSized;

        impl Drop for ZeroSized {
            fn drop(&mut self) {
                // No-op for zero-sized type
                println!("Dropping ZeroSized");
            }
        }

        assert_eq!(std::mem::size_of::<ZeroSized>(), 0);

        let mut vec = Vec::new();
        vec.push(ZeroSized);
        vec.push(ZeroSized);
        vec.push(ZeroSized);

        assert_eq!(vec.len, 3);
        assert_eq!(vec.cap(), usize::MAX); // Capacity should be usize::MAX for zero-sized types

        let popped = vec.pop();
        assert!(popped.is_some());
        assert_eq!(vec.len, 2);

        {
            let mut drain = vec.drain();
            assert!(drain.next().is_some());
            assert!(drain.next().is_some());
            assert!(drain.next().is_none());
        }

        assert_eq!(vec.len, 0);
        assert_eq!(vec.cap(), usize::MAX); // Capacity should remain usize::MAX after draining
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_vec_index_out_of_bounds() {
        let mut vec: Vec<i32> = Vec::new();
        vec.remove(0); // This should panic
    }

    #[test]
    #[should_panic(expected = "Capacity overflow for zero-sized type")]
    fn test_vec_zero_sized_type_overflow() {
        #[derive(Debug)]
        struct ZeroSized;
        let mut vec = Vec::new();
        vec.push(ZeroSized);
        vec.buf.grow(); // This should panic due to overflow
    }

    #[test]
    fn test_extend() {
        let mut vec = Vec::new();
        vec.extend(vec![1, 2, 3]);
        assert_eq!(vec.len, 3);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);

        vec.extend(vec![4, 5]);
        assert_eq!(vec.len, 5);
        assert_eq!(vec[3], 4);
        assert_eq!(vec[4], 5);
    }

    #[test]
    fn test_extend_empty() {
        let mut vec = Vec::new();
        vec.extend(std::iter::empty::<i32>());
        assert_eq!(vec.len, 0);
        assert_eq!(vec.cap(), 0);
    }

    #[test]
    fn test_extend_zero_sized() {
        #[derive(Debug, Clone)]
        struct ZeroSized;

        impl Drop for ZeroSized {
            fn drop(&mut self) {
                // No-op for zero-sized type
                println!("Dropping ZeroSized");
            }
        }

        assert_eq!(std::mem::size_of::<ZeroSized>(), 0);

        let mut vec = Vec::new();
        vec.extend(std::iter::repeat_n(ZeroSized, 5));
        assert_eq!(vec.len, 5);
        assert_eq!(vec.cap(), usize::MAX); // Capacity should be usize::MAX for zero-sized types

        vec.extend(std::iter::repeat_n(ZeroSized, 3));
        assert_eq!(vec.len, 8);
    }

    #[test]
    fn test_macro_empty() {
        let vec: Vec<i32> = my_vec![];
        assert_eq!(vec.len, 0);
        assert_eq!(vec.cap(), 0);
    }

    #[test]
    fn test_macro_single() {
        let vec = my_vec![1];
        assert_eq!(vec.len, 1);
        assert_eq!(vec[0], 1);
    }

    #[test]
    fn test_macro_multiple() {
        let vec = my_vec![1, 2, 3, 4, 5];
        assert_eq!(vec.len, 5);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);
        assert_eq!(vec[3], 4);
        assert_eq!(vec[4], 5);
    }

    #[test]
    fn test_macro_with_trailing_comma() {
        let vec = my_vec![1, 2, 3, 4, 5,];
        assert_eq!(vec.len, 5);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);
        assert_eq!(vec[3], 4);
        assert_eq!(vec[4], 5);
    }

    #[test]
    fn test_macro_repeat() {
        let vec = my_vec![42; 5];
        assert_eq!(vec.len, 5);
        for item in vec {
            assert_eq!(item, 42);
        }
    }
}
