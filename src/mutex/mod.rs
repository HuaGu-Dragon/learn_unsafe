use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, Ordering},
};

use atomic_wait::{wait, wake_one};

pub struct Mutex<T> {
    locked: AtomicU32,
    data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>,
}

unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Mutex {
            locked: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        while self.locked.swap(1, Ordering::Acquire) == 1 {
            wait(&self.locked, 1);
        }
        MutexGuard { lock: self }
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(0, Ordering::Release);
        wake_one(&self.lock.locked);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutex_single_thread() {
        let mutex = Mutex::new(5);
        {
            let mut guard = mutex.lock();
            assert_eq!(*guard, 5);
            *guard = 10; // Modify the value
        }
        let guard = mutex.lock();
        assert_eq!(*guard, 10); // Check the modified value
    }

    #[test]
    fn test_mutex_multi_thread() {
        use std::thread;

        let mutex = Mutex::new(0);

        thread::scope(|s| {
            s.spawn(|| {
                let mut guard = mutex.lock();
                *guard += 1; // Increment the value
            });
            s.spawn(|| {
                let mut guard = mutex.lock();
                *guard += 2; // Increment the value again
            });
        });
        let guard = mutex.lock();
        assert_eq!(*guard, 3); // Check the final value
    }
}
