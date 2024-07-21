use futures::{
    future::BoxFuture,
    task::{waker_ref, ArcWake},
};
use std::{
    future::Future,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    task::Context,
};

/// A future that can reschedule itself to be polled by an `Executor`.
struct Task {
    /// In-progress future that should be pushed to completion.
    ///
    /// The `Mutex` is not necessary for correctness, since we only have
    /// one thread executing tasks at once. However, Rust isn't smart
    /// enough to know that `future` is only mutated from one thread,
    /// so we need to use the `Mutex` to prove thread-safety. A production
    /// executor would not need this, and could use `UnsafeCell` instead.
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    /// Handle to place the task itself back onto the task queue.
    task_sender: Sender<Arc<Task>>,
}

impl Task {
    pub fn new(
        future: impl Future<Output = ()> + 'static + Send,
        task_sender: Sender<Arc<Task>>,
    ) -> Arc<Task> {
        Arc::new(Task {
            future: Mutex::new(Some(Box::pin(future))),
            task_sender,
        })
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Implement `wake` by sending this task back onto the task channel
        // so that it will be polled again by the executor.
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

struct Spawner {
    task_sender: Sender<Arc<Task>>,
}

impl Spawner {
    pub fn new(tx: Sender<Arc<Task>>) -> Spawner {
        Spawner { task_sender: tx }
    }

    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let task = Task::new(future, self.task_sender.clone());
        self.task_sender.send(task).unwrap();
    }
}

struct Executor {
    queue: Receiver<Arc<Task>>,
}

impl Executor {
    pub fn new(rx: Receiver<Arc<Task>>) -> Executor {
        Executor { queue: rx }
    }

    fn run(&self) {
        while let Ok(task) = self.queue.recv() {
            // Take the future, and if it has not yet completed (is still Some),
            // poll it in an attempt to complete it.
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // Create a `LocalWaker` from the task itself
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                // `BoxFuture<T>` is a type alias for
                // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
                // We can get a `Pin<&mut dyn Future + Send + 'static>`
                // from it by calling the `Pin::as_mut` method.
                if future.as_mut().poll(context).is_pending() {
                    // We're not done processing the future, so put it
                    // back in its task to be run again in the future.
                    *future_slot = Some(future);
                }
            }
        }
    }
}

pub struct MyTokio {
    spawner: Spawner,
    executor: Executor,
}

impl MyTokio {
    pub fn new() -> MyTokio {
        let (tx, rx) = mpsc::channel();

        MyTokio {
            spawner: Spawner::new(tx),
            executor: Executor::new(rx),
        }
    }

    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        self.spawner.spawn(future)
    }

    pub fn run(&self) {
        self.executor.run()
    }
}
