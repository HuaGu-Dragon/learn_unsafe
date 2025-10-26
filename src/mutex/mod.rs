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
    pub(crate) lock: &'a Mutex<T>,
    _marker: std::marker::PhantomData<*mut T>,
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
        MutexGuard {
            lock: self,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn with_fn<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut lock = self.lock();
        f(&mut *lock)
    }
}

fn lock_contended(state: &AtomicU32) {
    let mut spin_count = 0;
    if state
        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        while state.swap(2, Ordering::Acquire) != 0 {
            if spin_count < 100 {
                spin_count += 1;
                std::hint::spin_loop();
            }
            wait(state, 2);
        }
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

    #[test]
    fn test_mutex_high_pressure() {
        let mutex = Mutex::new(0);

        std::thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    for _ in 0..10000 {
                        let mut guard = mutex.lock();
                        *guard += 1; // Increment the value
                    }
                });
            }
        });
        let guard = mutex.lock();
        assert_eq!(*guard, 100000); // Check the final value after high contention
    }

    #[test]
    fn test_mutex_with_fn() {
        let mutex = Mutex::new(vec![]);

        mutex.with_fn(|data| {
            data.push(1);
            data.push(2);
        });

        mutex.with_fn(|data| {
            data.push(3);
        });

        mutex.with_fn(|data| {
            assert_eq!(data.len(), 3);
            assert_eq!(data[0], 1);
            assert_eq!(data[1], 2);
            assert_eq!(data[2], 3);
        });
    }
}
