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
        lock_contended(&self.locked);
        MutexGuard { lock: self }
    }
}

fn lock_contended(state: &AtomicU32) {
    let mut spin_count = 0;
    while let Err(e) = state.compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed) {
        if e == 1 {
            if spin_count < 100 {
                spin_count += 1;
                std::hint::spin_loop();
                continue;
            }
            _ = state.compare_exchange_weak(1, 2, Ordering::Acquire, Ordering::Relaxed);
        }
        wait(state, 2);
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
        if self.lock.locked.swap(0, Ordering::Release) == 2 {
            wake_one(&self.lock.locked);
        }
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
