use std::{
    sync::{Arc, Mutex},
    task::{Poll, Waker},
    time::Duration,
};

type Inner = Arc<Mutex<TimerState>>;

pub struct Timer {
    state: Inner,
}

#[derive(Default)]
pub struct TimerState {
    completed: bool,
    waker: Option<Waker>,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        let state: Inner = Arc::default();
        let state_clone = state.clone();
        std::thread::spawn(move || {
            std::thread::sleep(duration);
            let mut state = state_clone.lock().unwrap();
            state.completed = true;
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        });
        Timer { state }
    }
}

impl Future for Timer {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if state.completed {
            Poll::Ready(())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
