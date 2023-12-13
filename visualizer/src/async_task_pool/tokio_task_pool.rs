use std::future::Future;

use bevy::ecs::system::Resource;
use tokio::runtime::Runtime;

pub(crate) struct TaskPool {
    runtime: Runtime,
}

impl TaskPool {
    pub(crate) fn new() -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
        }
    }

    pub(crate) fn spawn(&self, task: impl Future<Output = ()> + Send + 'static) {
        self.runtime.spawn(task);
    }
}

impl Resource for TaskPool {}
