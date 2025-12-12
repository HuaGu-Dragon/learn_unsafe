use std::{
    pin::Pin,
    sync::{
        Arc,
        mpsc::{Receiver, SyncSender, sync_channel},
    },
    task::Context,
};

use futures::{
    FutureExt,
    task::{self, ArcWake},
};

use crate::mutex::Mutex;

pub mod timer;

pub struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

impl Executor {
    pub fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            let mut future_slot = task.future.lock();
            if let Some(mut future) = future_slot.take() {
                let waker = task::waker_ref(&task);
                let context = &mut Context::from_waker(&waker);

                if future.as_mut().poll(context).is_pending() {
                    *future_slot = Some(future);
                }
            }
        }
    }
}

pub struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("task queue full");
    }
}

pub struct Task {
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,

    task_sender: SyncSender<Arc<Task>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &std::sync::Arc<Self>) {
        let cloned = arc_self.clone();
        arc_self.task_sender.send(cloned).expect("task queue full");
    }
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    let (task_sender, ready_queue) = sync_channel(10_000);
    (Executor { ready_queue }, Spawner { task_sender })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::future::timer::Timer;

    use super::*;

    #[test]
    fn test_executor() {
        let (executor, spawner) = new_executor_and_spawner();

        spawner.spawn(async {
            println!("Hello from the future!");
        });

        drop(spawner);

        executor.run();
    }

    #[test]
    fn test_timer() {
        let (executor, spawner) = new_executor_and_spawner();

        spawner.spawn(async {
            println!("howdy!");
            Timer::new(Duration::from_secs(2)).await;
            println!("done!");
        });

        drop(spawner);

        executor.run();
    }

    #[test]
    fn test_multiple_timers() {
        let (executor, spawner) = new_executor_and_spawner();

        spawner.spawn(async {
            println!("Task 1 started");
            Timer::new(Duration::from_secs(3)).await;
            println!("Task 1 finished (3s)");
        });

        spawner.spawn(async {
            println!("Task 2 started");
            Timer::new(Duration::from_secs(1)).await;
            println!("Task 2 finished (1s)");
        });

        spawner.spawn(async {
            println!("Task 3 started");
            Timer::new(Duration::from_secs(2)).await;
            println!("Task 3 finished (2s)");
        });

        drop(spawner);

        executor.run();
    }
}
