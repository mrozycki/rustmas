use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::schema::{ConfigurationSchema, GetSchema, ParameterValue};

#[derive(Serialize, Deserialize, Debug, thiserror::Error)]
#[error("animation error: {message}")]
pub struct AnimationError {
    pub message: String,
}

pub trait Animation {
    type Parameters: GetSchema + DeserializeOwned + Serialize + Default + Clone;
    type Wrapped: Animation<Parameters: GetSchema>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self;

    fn new_wrapped(points: Vec<(f64, f64, f64)>) -> Self::Wrapped {
        Self::Wrapped::new(points)
    }

    fn get_schema(&self) -> ConfigurationSchema
    where
        Self::Parameters: GetSchema,
    {
        Self::Parameters::schema()
    }

    fn set_parameters(&mut self, _parameters: Self::Parameters) {}

    fn get_parameters(&self) -> Self::Parameters
    where
        Self::Parameters: Default,
    {
        Self::Parameters::default()
    }

    fn get_fps(&self) -> f64 {
        30.0
    }

    fn update(&mut self, _delta: f64) {}

    fn on_event(&mut self, _event: crate::event::Event) {}

    fn render(&self) -> lightfx::Frame;
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum JsonRpcMethod {
    Initialize {
        points: Vec<(f64, f64, f64)>,
    },
    ParameterSchema,
    SetParameters {
        params: HashMap<String, ParameterValue>,
    },
    GetParameters,
    GetFps,
    Update {
        time_delta: f64,
    },
    OnEvent {
        event: crate::event::Event,
    },
    Render,
}
