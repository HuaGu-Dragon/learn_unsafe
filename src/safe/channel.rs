use std::{
    collections::VecDeque,
    marker::PhantomData,
    sync::{Arc, Condvar, Mutex},
};

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Send> Sync for Sender<T> {}

impl<T> Sender<T> {
    pub fn send(&self, value: T) {
        let mut shared = self.shared.inner.lock().unwrap();
        shared.queue.push_back(value);
        drop(shared);
        self.shared.available.notify_one();
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders += 1;
        drop(inner);

        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders -= 1;
        let no_senders = inner.senders == 0;
        drop(inner);
        if no_senders {
            self.shared.available.notify_one();
        }
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    // impl !Send and !Sync
    marker: PhantomData<*const T>,
}

// Receiver is !Sync because it contains a PhantomData with a raw pointer
unsafe impl<T: Send> Send for Receiver<T> {}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Option<T> {
        let mut shared = self.shared.inner.lock().unwrap();
        loop {
            match shared.queue.pop_front() {
                Some(value) => break Some(value),
                None if shared.senders == 0 => break None,
                None => shared = self.shared.available.wait(shared).unwrap(),
            }
        }
    }
}

struct Shared<T> {
    inner: Mutex<Inner<T>>,
    available: Condvar,
}

struct Inner<T> {
    queue: VecDeque<T>,
    senders: usize,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Shared {
        inner: Mutex::new(Inner {
            queue: VecDeque::new(),
            senders: 1,
        }),
        available: Condvar::new(),
    };

    let inner = Arc::new(inner);
    (
        Sender {
            shared: inner.clone(),
        },
        Receiver {
            shared: inner,
            marker: PhantomData,
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::safe::channel::channel;

    #[test]
    fn ping_pong() {
        let (tx, rx) = channel();
        tx.send(42);
        assert_eq!(rx.recv(), Some(42));
    }

    #[test]
    fn closed() {
        let (tx, rx) = channel::<i32>();
        drop(tx);
        assert_eq!(rx.recv(), None);
    }
}
