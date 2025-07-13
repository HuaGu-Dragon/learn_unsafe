use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct Channel<T> {
    message: UnsafeCell<Option<T>>,
    ready: AtomicBool,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            message: UnsafeCell::new(None),
            ready: AtomicBool::new(false),
        }
    }

    /// SAFETY: Caller must ensure that the channel is not ready
    /// and that no other thread is sending a message at the same time.
    /// Only call it once.
    pub unsafe fn send(&self, message: T) {
        unsafe {
            self.message.get().write(Some(message));
        }
        self.ready.store(true, Ordering::Release);
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    /// Only call this if `is_ready` is true
    /// SAFETY: Caller must ensure that the channel is ready
    pub unsafe fn recv(&self) -> T {
        unsafe { self.message.get().read().expect("No message available") }
    }
}

unsafe impl<T: Send> Send for Channel<T> {}
unsafe impl<T: Send> Sync for Channel<T> {}

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
