use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
    thread::Thread,
};

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

impl<T> Channel<T> {
    pub const fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        }
    }

    pub fn split(&mut self) -> (Sender<'_, T>, Receiver<'_, T>) {
        *self = Self::new();
        (
            Sender {
                channel: self,
                // Capture the current thread to ensure the sender can send messages
                // This is necessary to ensure the sender can wake up the receiver
                receiving_thread: std::thread::current(),
            },
            Receiver {
                channel: self,
                _send_marker: std::marker::PhantomData,
            },
        )
    }
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
    receiving_thread: Thread,
}

impl<'a, T> Sender<'a, T> {
    pub fn send(self, message: T) {
        unsafe {
            (*self.channel.message.get()).write(message);
        }
        self.channel.ready.store(true, Ordering::Release);
        Thread::unpark(&self.receiving_thread);
    }
}

pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
    // Use a marker to ensure the receiver does not send to other threads
    _send_marker: std::marker::PhantomData<*const ()>,
}

impl<T> Receiver<'_, T> {
    pub fn recv(self) -> T {
        if !self.channel.ready.swap(false, Ordering::Acquire) {
            std::thread::park();
        };
        unsafe {
            // SAFETY: We assume the message is ready to be read
            (*self.channel.message.get()).assume_init_read()
        }
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
        let mut channel = Channel::new();
        let (sender, receiver) = channel.split();

        sender.send(42);

        let res = receiver.recv();
        assert_eq!(res, 42);
    }

    #[test]
    fn test_channel_multi_thread() {
        use std::thread;

        let mut channel = Channel::new();
        let (sender, receiver) = channel.split();

        thread::scope(|s| {
            s.spawn(move || {
                sleep(Duration::from_millis(100));
                sender.send(42);
            });
            assert_eq!(receiver.recv(), 42);
        });
    }
}
