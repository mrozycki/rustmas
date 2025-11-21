mod config;
mod controller;
mod factory;

pub use config::ControllerConfig;
pub use controller::{Controller, ControllerError};
pub use factory::{AnimationFactory, AnimationFactoryError, points_from_path};
