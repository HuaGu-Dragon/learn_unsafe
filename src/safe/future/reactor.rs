use std::{
    collections::{HashMap, VecDeque},
    pin::Pin,
};

use mio::{Token, Waker};

use crate::cell::{Cell, RefCell};

pub struct Reactor {
    pub poll: RefCell<mio::Poll>,
    pub wakers: RefCell<HashMap<Token, Waker>>,
    pub next_token: Cell<usize>,
}

type Future = Pin<Box<dyn std::future::Future<Output = ()>>>;

thread_local! {
    pub static REACTOR: Reactor = Reactor {
        poll: RefCell::new(mio::Poll::new().unwrap()),
        wakers: RefCell::new(HashMap::new()),
        next_token: Cell::new(1),
    };

    pub static READY_QUEUE: RefCell<VecDeque<Token>> = const { RefCell::new(VecDeque::new()) };

    pub static FUTURES: RefCell<HashMap<Token, Future>> = RefCell::new(HashMap::new());
}
