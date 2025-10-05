use std::{
    collections::VecDeque,
    marker::PhantomData,
    sync::{Arc, Condvar, Mutex},
};

pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Send> Sync for Sender<T> {}

impl<T> Sender<T> {
    pub fn send(&self, value: T) {
        let mut queue = self.inner.queue.lock().unwrap();
        queue.push_back(value);
        drop(queue);
        self.inner.available.notify_one();
    }
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
    // impl !Send and !Sync
    marker: PhantomData<*const T>,
}

// Receiver is !Sync because it contains a PhantomData with a raw pointer
unsafe impl<T: Send> Send for Receiver<T> {}

impl<T> Receiver<T> {
    pub fn recv(&self) -> T {
        let mut queue = self.inner.queue.lock().unwrap();
        loop {
            if let Some(value) = queue.pop_front() {
                break value;
            }
            queue = self.inner.available.wait(queue).unwrap();
        }
    }
}

struct Inner<T> {
    queue: Mutex<VecDeque<T>>,
    available: Condvar,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: Mutex::new(VecDeque::new()),
        available: Condvar::new(),
    };

    let inner = Arc::new(inner);
    (
        Sender {
            inner: inner.clone(),
        },
        Receiver {
            inner,
            marker: PhantomData,
        },
    )
}
