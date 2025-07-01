use std::{ptr::NonNull, sync::atomic::AtomicUsize};

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
