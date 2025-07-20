#![cfg(target_os = "linux")]
#![allow(dead_code)]

use std::{
    io::Result,
    net::TcpStream,
    os::{
        fd::AsRawFd,
        raw::{c_int, c_void},
    },
};

#[repr(C)]
#[cfg_attr(target_arch = "x86_64", repr(packed))]
#[derive(Debug, Clone, Copy)]
pub struct EpollEvent {
    pub events: u32,
    pub data: EpollData,
}

#[repr(C)]
#[cfg_attr(target_arch = "x86_64", repr(packed))]
#[derive(Clone, Copy)]
pub union EpollData {
    pub ptr: *mut c_void,
    pub fd: c_int,
    pub u32_val: u32,
    pub u64_val: u64,
}

impl std::fmt::Debug for EpollData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EpollData {{ ... }}")
    }
}

// epoll 事件类型常量
pub const EPOLLIN: u32 = 0x001;
pub const EPOLLPRI: u32 = 0x002;
pub const EPOLLOUT: u32 = 0x004;
pub const EPOLLERR: u32 = 0x008;
pub const EPOLLHUP: u32 = 0x010;
pub const EPOLLNVAL: u32 = 0x020;
pub const EPOLLRDNORM: u32 = 0x040;
pub const EPOLLRDBAND: u32 = 0x080;
pub const EPOLLWRNORM: u32 = 0x100;
pub const EPOLLWRBAND: u32 = 0x200;
pub const EPOLLMSG: u32 = 0x400;
pub const EPOLLRDHUP: u32 = 0x2000;
pub const EPOLLEXCLUSIVE: u32 = 1u32 << 28;
pub const EPOLLWAKEUP: u32 = 1u32 << 29;
pub const EPOLLONESHOT: u32 = 1u32 << 30;
pub const EPOLLET: u32 = 1u32 << 31;

// epoll_ctl 操作常量
pub const EPOLL_CTL_ADD: c_int = 1;
pub const EPOLL_CTL_DEL: c_int = 2;
pub const EPOLL_CTL_MOD: c_int = 3;

// epoll_create1 标志
pub const EPOLL_CLOEXEC: c_int = 0o2000000;

mod ffi {
    use super::*;

    #[link(name = "c")]
    unsafe extern "C" {
        /// create a new epoll instance, returns the epoll file descriptor
        /// size parameter is ignored but must be greater than 0 (for backward compatibility)
        pub fn epoll_create(size: c_int) -> c_int;

        /// controls the epoll instance
        /// epfd: epoll file descriptor
        /// op: operation type (EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD)
        /// fd: file descriptor to monitor
        /// event: event structure
        pub fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut EpollEvent) -> c_int;

        /// wait for events to occur
        /// epfd: epoll file descriptor
        /// events: array of events
        /// maxevents: maximum number of events
        /// timeout: timeout in milliseconds, -1 means wait indefinitely
        pub fn epoll_wait(
            epfd: c_int,
            events: *mut EpollEvent,
            maxevents: c_int,
            timeout: c_int,
        ) -> c_int;

        /// closes the file descriptor
        pub fn close(fd: c_int) -> c_int;
    }
}

pub struct Epoll {
    fd: c_int,
}

pub struct Poll {
    register: Register,
}

pub struct Register {
    fd: c_int,
}

impl Register {
    pub fn register(&self, source: TcpStream, interests: u32) -> Result<()> {
        let mut event = EpollEvent {
            events: interests,
            data: EpollData {
                fd: source.as_raw_fd(),
            },
        };
        let res =
            unsafe { ffi::epoll_ctl(self.fd, EPOLL_CTL_ADD, source.as_raw_fd(), &raw mut event) };
        if res < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }
}

impl Drop for Register {
    fn drop(&mut self) {
        let res = unsafe { ffi::close(self.fd) };

        if res < 0 {
            let err = std::io::Error::last_os_error();
            println!("Failed to close epoll fd: {}", err);
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        collections::HashMap,
        ffi::CStr,
        io::{BufRead, BufReader, Write},
        net::TcpStream,
        os::fd::AsRawFd,
    };

    use super::*;

    #[test]
    fn test_ffi_work() {
        unsafe {
            let fd = ffi::epoll_create(1);
            assert!(fd >= 0, "Failed to create epoll instance");

            let mut streams = HashMap::new();
            let mut tcp =
                TcpStream::connect("127.0.0.1:8080").expect("Failed to connect to server");
            tcp.set_nonblocking(true)
                .expect("Failed to set non-blocking mode");
            tcp.write("Hello, epoll!\0".as_bytes())
                .expect("Failed to write to TCP stream");
            let tcp_fd = tcp.as_raw_fd();
            streams.insert(tcp_fd, tcp);

            let mut event = EpollEvent {
                // use one-shot mode: trigger once when ready, then rearm explicitly
                // only monitor read readiness; drop EPOLLOUT to avoid immediate wake-ups
                events: EPOLLIN | EPOLLET | EPOLLONESHOT,
                data: EpollData { fd: tcp_fd },
            };
            // allocate buffer for one event
            let mut events: Vec<EpollEvent> = Vec::with_capacity(1);
            // initialize the vector length to hold one event
            events.set_len(1);
            // register the socket with epoll

            let ctl_res = ffi::epoll_ctl(fd, EPOLL_CTL_ADD, tcp_fd, &mut event);
            assert_eq!(ctl_res, 0, "epoll_ctl failed");

            // wait for events
            let n = ffi::epoll_wait(fd, events.as_mut_ptr(), 1, 1000);
            assert!(n >= 0, "epoll_wait failed");

            let n = n as usize;
            for e in &events[..n] {
                let fd = e.data.fd;
                let mut buf = Vec::new();
                let tcp = streams.get_mut(&fd).expect("Failed to get TCP stream");
                let mut reader = BufReader::new(tcp);
                loop {
                    match reader.read_until(0, &mut buf) {
                        Ok(_) => break,
                        // Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        //     // Wait briefly before retrying
                        //     std::thread::sleep(std::time::Duration::from_millis(50));
                        //     continue;
                        // }
                        Err(e) => panic!("Failed to read from TCP stream: {:?}", e),
                    }
                }
                if buf.is_empty() {
                    println!("No data read from fd {}", fd);
                } else {
                    println!(
                        "Data read from fd {}: {:?}",
                        fd,
                        CStr::from_bytes_with_nul_unchecked(&buf)
                    );
                }
            }
            ffi::close(fd);
        }
    }
}
