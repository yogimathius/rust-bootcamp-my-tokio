use std::{
    future::Future,
    sync::{mpsc::Sender, Arc},
};

use crate::task::Task;
pub struct Spawner {
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
