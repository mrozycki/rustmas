mod config;
mod controller;
mod factory;
mod jsonrpc;
mod plugin;
mod wasm;

pub use config::ControllerConfig;
pub use controller::{Controller, ControllerError};
pub use factory::{AnimationFactory, AnimationFactoryError, points_from_path};
