use std::{
    future::Future,
    sync::mpsc::{self},
};

use crate::{executor::Executor, spawner::Spawner};

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
