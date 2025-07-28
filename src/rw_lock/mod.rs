use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, Ordering},
    u32,
};

use atomic_wait::{wait, wake_all, wake_one};

pub struct RwLock<T> {
    state: AtomicU32,
    write_waker: AtomicU32,
    value: UnsafeCell<T>,
}

pub struct ReadGuard<'a, T> {
    lock: &'a RwLock<T>,
}

pub struct WriteGuard<'a, T> {
    lock: &'a RwLock<T>,
}

unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> RwLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            write_waker: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    pub fn read(&self) -> ReadGuard<'_, T> {
        let mut state = self.state.load(Ordering::Relaxed);
        loop {
            if state < u32::MAX {
                assert!(state != u32::MAX - 1, "too many readers");
                match self.state.compare_exchange_weak(
                    state,
                    state + 1,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        return ReadGuard { lock: self };
                    }
                    Err(new_state) => {
                        state = new_state;
                    }
                }
            }
            if state == u32::MAX {
                wait(&self.state, u32::MAX);
                state = self.state.load(Ordering::Relaxed);
            }
        }
    }

    pub fn write(&self) -> WriteGuard<'_, T> {
        while self
            .state
            .compare_exchange_weak(0, u32::MAX, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            let writes = self.write_waker.load(Ordering::Acquire);
            if self.state.load(Ordering::Relaxed) != 0 {
                wait(&self.write_waker, writes);
            }
        }
        WriteGuard { lock: self }
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        if self.lock.state.fetch_sub(1, Ordering::Release) == 1 {
            self.lock.write_waker.fetch_add(1, Ordering::Release);
            wake_one(&self.lock.write_waker);
        }
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Release);
        self.lock.write_waker.fetch_sub(1, Ordering::Release);
        wake_one(&self.lock.write_waker);
        wake_all(&self.lock.state);
    }
}

#[cfg(test)]
mod tests {
    use super::RwLock;

    #[test]
    fn test_single_thread() {
        let rw = RwLock::new(vec![1, 2, 3]);

        let r = rw.read();
        assert_eq!(r.len(), 3);

        let r2 = rw.read();
        assert_eq!(r.len(), 3);

        drop(r);
        drop(r2);

        let mut w = rw.write();
        w.push(4);
        drop(w);

        let r = rw.read();
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn test_multi_thread() {
        let rw = RwLock::new(vec![]);

        std::thread::scope(|s| {
            s.spawn(|| {
                let mut w = rw.write();
                w.push(1);
                w.push(2);
            });

            s.spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let r1 = rw.read();
                println!("{:?}", *r1);
                let r2 = rw.read();
                println!("{:?}", *r2);
            });
        })
    }
}
