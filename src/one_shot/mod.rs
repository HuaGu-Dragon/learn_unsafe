use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
};

pub struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    ready: Condvar,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            ready: Condvar::new(),
        }
    }

    pub fn send(&self, message: T) {
        self.queue.lock().unwrap().push_back(message);
        // Notify one waiting receiver that a message is available
        self.ready.notify_one();
    }

    pub fn recv(&self) -> T {
        let mut queue = self.queue.lock().unwrap();
        // Wait until there is a message to receive
        loop {
            if let Some(message) = queue.pop_front() {
                return message;
            }
            queue = self.ready.wait(queue).unwrap();
        }
    }
}
