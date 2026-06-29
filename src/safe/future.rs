use std::{
    collections::VecDeque,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use mio::{Events, Token};

use crate::safe::future::reactor::{FUTURES, REACTOR, READY_QUEUE};

pub mod reactor;
mod waker;

pub fn block_on<F: Future>(f: F) -> F::Output {
    let mut f = Box::pin(f);
    let mut events = Events::with_capacity(1024);

    READY_QUEUE.with(|q| {
        q.borrow_mut()
            .expect("READY_QUEUE is thread_local")
            .push_back(Token(0))
    });

    loop {
        let mut ready_queue = VecDeque::new();

        READY_QUEUE.with(|q| {
            std::mem::swap(
                &mut *q.borrow_mut().expect("READY_QUEUE is thread_local"),
                &mut ready_queue,
            );
        });

        for token in ready_queue {
            if token == Token(0) {
                let waker = Waker::from(Arc::new(waker::Waker { token: Token(0) }));

                let mut cx = Context::from_waker(&waker);

                if let Poll::Ready(output) = f.as_mut().poll(&mut cx) {
                    FUTURES.with(|t| t.borrow_mut().expect("FUTURES_MAP is thread_local").clear());
                    return output;
                }
            } else {
                let done = FUTURES.with(|t| {
                    let mut futures = t.borrow_mut().unwrap();

                    if let Some(fut) = futures.get_mut(&token) {
                        let waker = Waker::from(Arc::new(waker::Waker { token }));
                        let mut cx = Context::from_waker(&waker);

                        matches!(fut.as_mut().poll(&mut cx), Poll::Ready(_))
                    } else {
                        false
                    }
                });

                if done {
                    FUTURES.with(|t| {
                        t.borrow_mut()
                            .expect("FUTURES_MAP is thread_local")
                            .remove(&token)
                    });
                }
            }
        }

        if READY_QUEUE.with(|q| q.borrow().expect("READY_QUEUE is thread_local").is_empty()) {
            REACTOR.with(|r| {
                r.poll
                    .borrow_mut()
                    .expect("REACTOR is thread_local")
                    .poll(&mut events, None)
                    .expect("poll failed");

                for event in events.iter() {
                    if let Some(waker) = r
                        .wakers
                        .borrow_mut()
                        .expect("REACTOR is thread_local")
                        .remove(&event.token())
                    {
                        waker.wake().unwrap();
                    }
                }
            });
        }
    }
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    let token = REACTOR.with(|r| {
        let id = r.next_token.get();
        r.next_token.set(id + 1);
        Token(id)
    });

    FUTURES.with(|t| {
        t.borrow_mut()
            .expect("FUTURES_MAP is thread_local")
            .insert(token, Box::pin(future))
    });

    READY_QUEUE.with(|q| {
        q.borrow_mut()
            .expect("READY_QUEUE is thread_local")
            .push_back(token)
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[test]
    fn test_block_on() {
        let future = async { 42 };

        assert_eq!(block_on(future), 42);
    }

    #[test]
    fn test_spawn() {
        let x = Arc::new(Mutex::new(42));

        let x_clone = x.clone();
        spawn(async move {
            let mut x = x_clone.lock().unwrap();
            *x += 1;
        });

        let x_clone = x.clone();
        let future = async move {
            let mut x = x_clone.lock().unwrap();
            *x += 1;
            *x
        };

        assert_eq!(block_on(future), 44);
        assert_eq!(*x.lock().unwrap(), 44);
    }
}
