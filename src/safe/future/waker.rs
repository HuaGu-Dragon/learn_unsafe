use std::task::Wake;

use mio::Token;

use crate::safe::future::reactor::READY_QUEUE;

pub struct Waker {
    pub token: Token,
}

impl Wake for Waker {
    fn wake(self: std::sync::Arc<Self>) {
        READY_QUEUE.with(|q| {
            q.borrow_mut()
                .expect("READY_QUEUE is thread_local")
                .push_back(self.token)
        })
    }
}
