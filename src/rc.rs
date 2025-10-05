use std::{cell::Cell, marker::PhantomData, ops::Deref, ptr::NonNull};

pub struct Rc<T> {
    inner: NonNull<Inner<T>>,
    _marker: PhantomData<Inner<T>>,
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.inner.as_ref() };
        let c = inner.strong.get();
        inner.strong.set(c + 1);
        Self {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

struct Inner<T> {
    strong: Cell<usize>,
    value: T,
}

impl<T> Inner<T> {
    fn new(value: T) -> Self {
        Self {
            strong: Cell::new(1),
            value,
        }
    }
}

impl<T> Rc<T> {
    pub fn new(value: T) -> Self {
        let inner = Box::new(Inner::new(value));
        Self {
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) },
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.inner.as_ref().value }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.inner.as_ref() };
        let c = inner.strong.get();
        if c == 1 {
            unsafe { drop(Box::from_raw(self.inner.as_ptr())) };
        } else {
            inner.strong.set(c - 1);
        }
    }
}
