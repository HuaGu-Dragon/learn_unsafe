use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use atomic_wait::{wait, wake_all, wake_one};

use crate::mutex::MutexGuard;

pub struct Condvar {
    counter: AtomicU32,
    waiter: AtomicUsize,
}

impl Condvar {
    pub const fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
            waiter: AtomicUsize::new(0),
        }
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        let counter = self.counter.load(Ordering::Relaxed);
        self.waiter.fetch_add(1, Ordering::Relaxed);

        let mutex = guard.lock;
        drop(guard);

        wait(&self.counter, counter);
        self.waiter.fetch_sub(1, Ordering::Relaxed);

        mutex.lock()
    }

    pub fn notify_one(&self) {
        if self.waiter.load(Ordering::Relaxed) == 0 {
            return;
        }
        self.counter.fetch_add(1, Ordering::Relaxed);
        wake_one(&self.counter);
    }

    pub fn notify_all(&self) {
        if self.waiter.load(Ordering::Relaxed) == 0 {
            return;
        }
        self.counter.fetch_add(1, Ordering::Relaxed);
        wake_all(&self.counter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutex::Mutex;
    use std::collections::VecDeque;

    #[test]
    fn test_condvar() {
        let queue = Mutex::new(VecDeque::new());
        let not_empty = Condvar::new();

        std::thread::scope(|s| {
            s.spawn(|| {
                for _ in 0..100000 {
                    let mut q = queue.lock();
                    let _item = loop {
                        if let Some(item) = q.pop_front() {
                            break item;
                        } else {
                            q = not_empty.wait(q);
                        }
                    };
                    drop(q);
                }
            });

            for i in 0..100000 {
                queue.lock().push_back(i);
                not_empty.notify_one();
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        });
    }
}
