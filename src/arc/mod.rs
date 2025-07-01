use std::{ops::Deref, ptr::NonNull, sync::atomic::AtomicUsize};

use crate::r#box::Box;

pub struct Arc<T> {
    ptr: NonNull<ArcInner<T>>,
    phantom: std::marker::PhantomData<T>,
}

struct ArcInner<T> {
    rc: AtomicUsize,
    data: T,
}

impl<T> Arc<T> {
    pub fn new(data: T) -> Self {
        // Create a Box containing the ArcInner structure
        // and initialize the reference count to 1.
        // This is done to ensure that the data is heap-allocated
        let boxed = Box::new(ArcInner {
            rc: AtomicUsize::new(1),
            data,
        });
        Arc {
            ptr: NonNull::new(Box::into_raw(boxed)).unwrap(),
            phantom: std::marker::PhantomData,
        }
    }
}

unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let inner = unsafe { self.ptr.as_ref() };
        &inner.data
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.ptr.as_ref() };
        // Increment the reference count atomically
        let old_rc = inner.rc.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if old_rc >= isize::MAX as usize {
            std::process::abort(); // Prevent overflow
        }
        Arc {
            ptr: self.ptr,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.ptr.as_ref() };

        if inner.rc.fetch_sub(1, std::sync::atomic::Ordering::Release) != 1 {
            return;
        }

        // std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire);
        inner.rc.load(std::sync::atomic::Ordering::Acquire);
        // If the reference count reaches zero, we can safely deallocate the memory
        unsafe {
            // Convert the pointer back to Box to deallocate
            Box::from_raw(self.ptr.as_ptr());
        }
    }
}
