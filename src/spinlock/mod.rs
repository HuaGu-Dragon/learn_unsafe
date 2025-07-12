#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};

pub struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    pub const fn new() -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            std::hint::spin_loop();
        }
    }

    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinlock_single_threaded() {
        let lock = SpinLock::new();
        let mut data = vec![];

        lock.lock();
        data.push(1);
        lock.unlock();

        lock.lock();
        print!("{:?}", data);
        lock.unlock();
    }

    #[test]
    fn spinlock_multi_threaded() {
        use std::thread;

        let lock = SpinLock::new();
        let mut data = vec![];

        thread::scope(|s| {
            s.spawn(|| {
                lock.lock();
                let data_ptr = &raw const data as *mut Vec<i32>;
                unsafe {
                    (*data_ptr).push(1);
                }
                lock.unlock();
            });
            thread::sleep(std::time::Duration::from_millis(100));
            lock.lock();
            let data_ptr = &raw const data as *mut Vec<i32>;
            unsafe {
                (*data_ptr).push(2);
            }
            lock.unlock();
        });
        lock.lock();
        data.push(3);
        println!("{:?}", data);
        lock.unlock();
    }
}
