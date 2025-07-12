#![allow(dead_code)]

use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> &mut T {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            std::hint::spin_loop();
        }
        // SAFETY: We have exclusive access to the data while the lock is held
        unsafe { self.data.get().as_mut().unwrap() }
    }

    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinlock_single_threaded() {
        let lock = SpinLock::new(vec![]);

        let data = lock.lock();
        data.push(1);
        lock.unlock();

        let data = lock.lock();
        data.push(2);
        lock.unlock();

        let data = lock.lock();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], 1);
        assert_eq!(data[1], 2);
        lock.unlock();
    }

    #[test]
    fn spinlock_multi_threaded() {
        use std::thread;

        let lock = SpinLock::new(vec![]);

        thread::scope(|s| {
            s.spawn(|| {
                let data = lock.lock();
                data.push(1);
                lock.unlock();
            });
            thread::sleep(std::time::Duration::from_millis(100));
            let data = lock.lock();
            data.push(2);
            lock.unlock();
        });
        let data = lock.lock();
        data.push(3);
        assert_eq!(data.len(), 3);
        assert_eq!(data.iter().sum::<i32>(), 6);
        lock.unlock();
    }
}
