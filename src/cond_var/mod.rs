use std::sync::atomic::{AtomicU32, Ordering};

use atomic_wait::{wait, wake_all, wake_one};

use crate::mutex::MutexGuard;

struct Condvar {
    counter: AtomicU32,
}

impl Condvar {
    pub const fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
        }
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        let counter = self.counter.load(Ordering::Relaxed);

        let mutex = guard.lock;
        drop(guard);

        wait(&self.counter, counter);

        mutex.lock()
    }

    pub fn notify_one(&self) {
        self.counter.fetch_add(1, Ordering::Relaxed);
        wake_one(&self.counter);
    }

    pub fn notify_all(&self) {
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
                for _ in 0..10 {
                    let mut q = queue.lock();
                    let item = loop {
                        if let Some(item) = q.pop_front() {
                            break item;
                        } else {
                            q = not_empty.wait(q);
                        }
                    };
                    drop(q);
                    dbg!(item);
                }
            });

            for i in 0..10 {
                queue.lock().push_back(i);
                not_empty.notify_one();
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
    }
}
