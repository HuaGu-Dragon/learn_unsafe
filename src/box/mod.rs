use std::{
    alloc::handle_alloc_error,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Box<T> {
    inner: NonNull<T>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Box<T> {
    pub fn new(value: T) -> Self {
        assert!(std::mem::size_of::<T>() > 0, "Cannot box zero-sized types");
        // Allocate memory for T on the heap
        // and write the value into that memory.
        let layout = std::alloc::Layout::new::<T>();
        let ptr = unsafe { std::alloc::alloc(layout) as *mut T };

        let ptr = match NonNull::new(ptr as *mut T) {
            Some(non_null_ptr) => non_null_ptr,
            None => handle_alloc_error(layout),
        };

        unsafe { ptr.as_ptr().write(value) };

        Self {
            inner: ptr,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn from_raw(ptr: *mut T) -> Self {
        assert!(!ptr.is_null(), "Cannot create Box from null pointer");
        let non_null_ptr = NonNull::new(ptr).expect("Non-null pointer expected");
        Self {
            inner: non_null_ptr,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn into_raw(self) -> *mut T {
        let ptr = self.inner.as_ptr();
        std::mem::forget(self); // Prevent the destructor from running
        ptr
    }

    pub fn as_ptr(&self) -> *const T {
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_ptr()
    }
}

impl<T> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.inner.as_mut() }
    }
}

impl<T> Drop for Box<T> {
    fn drop(&mut self) {
        // Deallocate the memory for T
        let layout = std::alloc::Layout::new::<T>();
        unsafe {
            // Use `std::ptr::drop_in_place` to call the destructor of T
            std::ptr::drop_in_place(self.inner.as_ptr());
            std::alloc::dealloc(self.inner.as_ptr() as *mut u8, layout);
        }
    }
}

mod test {
    use crate::r#box::Box;

    #[test]
    fn test_new() {
        let a = Box::new(String::from("value"));
        print!("{a:?}");
    }
}
