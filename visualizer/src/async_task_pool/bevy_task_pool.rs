use std::future::Future;

use bevy::ecs::system::Resource;
use bevy::tasks::AsyncComputeTaskPool;

pub(crate) struct TaskPool {}

impl TaskPool {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn spawn(&self, task: impl Future<Output = ()> + 'static) {
        AsyncComputeTaskPool::get().spawn(task).detach()
    }
}

impl Resource for TaskPool {}
