use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
    used: AtomicBool,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
            used: AtomicBool::new(false),
        }
    }

    /// SAFETY: Caller must ensure that the channel is not ready
    /// and that no other thread is sending a message at the same time.
    /// Only call it once.
    pub unsafe fn send(&self, message: T) {
        if self.used.swap(true, Ordering::Relaxed) {
            panic!("Channel is already in use");
        }
        unsafe {
            (*self.message.get()).write(message);
        }
        self.ready.store(true, Ordering::Release);
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    /// Only call this if `is_ready` is true
    /// SAFETY: Caller must ensure that the channel is ready
    pub unsafe fn recv(&self) -> T {
        if !self.ready.swap(false, Ordering::Acquire) {
            panic!("Channel is not ready");
        }
        unsafe { (*self.message.get()).assume_init_read() }
    }
}

unsafe impl<T: Send> Send for Channel<T> {}
unsafe impl<T: Send> Sync for Channel<T> {}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            // SAFETY: We are dropping the channel, so we can safely assume the message is ready
            unsafe {
                (*self.message.get()).assume_init_drop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use super::*;

    #[test]
    fn test_channel_single_thread() {
        let channel = Channel::new();

        // Test sending a message
        unsafe {
            channel.send(42);
        }

        // Test receiving a message
        unsafe {
            assert!(channel.is_ready());
            let message = channel.recv();
            assert_eq!(message, 42);
        }
    }

    #[test]
    fn test_channel_multi_thread() {
        use std::thread;

        let channel = Channel::new();

        thread::scope(|s| {
            s.spawn(|| {
                sleep(Duration::from_millis(100));
                unsafe {
                    channel.send(100);
                }
            });

            loop {
                if channel.is_ready() {
                    let res = unsafe { channel.recv() };
                    assert_eq!(res, 100);
                    break;
                }
            }
        });
    }
}
