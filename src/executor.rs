use futures::task::waker_ref;
use std::{
    sync::{mpsc::Receiver, Arc},
    task::Context,
};

use crate::task::Task;

pub struct Executor {
    queue: Receiver<Arc<Task>>,
}

impl Executor {
    pub fn new(rx: Receiver<Arc<Task>>) -> Executor {
        Executor { queue: rx }
    }

    pub fn run(&self) {
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
