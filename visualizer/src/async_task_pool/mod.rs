#[cfg(target_arch = "wasm32")]
mod bevy_task_pool;
#[cfg(not(target_arch = "wasm32"))]
mod tokio_task_pool;

#[cfg(target_arch = "wasm32")]
pub(crate) use bevy_task_pool::TaskPool;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use tokio_task_pool::TaskPool;
