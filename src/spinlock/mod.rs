#![allow(dead_code)]

use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<'a, T> SpinLockGuard<'a, T> {
    pub fn new(lock: &'a SpinLock<T>) -> Self {
        // SAFETY: The lock must be held when creating a guard
        // This ensures that we have exclusive access to the data
        SpinLockGuard { lock }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        // When the guard is dropped, we release the lock
        self.lock.locked.store(false, Ordering::Release);
    }
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            std::hint::spin_loop();
        }
        // SAFETY: We have exclusive access to the data while the lock is held
        SpinLockGuard::new(self)
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

        {
            let mut data = lock.lock();
            data.push(1);
        }

        {
            let mut data = lock.lock();
            data.push(2);
        }

        let data = lock.lock();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], 1);
        assert_eq!(data[1], 2);
    }

    #[test]
    fn spinlock_multi_threaded() {
        use std::thread;

        let lock = SpinLock::new(vec![]);

        thread::scope(|s| {
            s.spawn(|| {
                let mut data = lock.lock();
                data.push(1);
            });
            thread::sleep(std::time::Duration::from_millis(100));
            let mut data = lock.lock();
            data.push(2);
        });
        let mut data = lock.lock();
        data.push(3);
        assert_eq!(data.len(), 3);
        assert_eq!(data.iter().sum::<i32>(), 6);
    }
}
