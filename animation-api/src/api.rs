use std::fmt;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::parameter_schema::{GetParametersSchema, ParametersSchema};

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimationError {
    pub message: String,
}

impl fmt::Display for AnimationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnimationError: {}", self.message)
    }
}

impl std::error::Error for AnimationError {}

pub trait Animation {
    type Parameters: GetParametersSchema + DeserializeOwned + Serialize + Default + Clone;

    fn animation_name(&self) -> &str;

    fn parameter_schema(&self) -> ParametersSchema
    where
        Self::Parameters: GetParametersSchema,
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
    Initialize { points: Vec<(f64, f64, f64)> },
    AnimationName,
    ParameterSchema,
    SetParameters { params: serde_json::Value },
    GetParameters,
    GetFps,
    Update { time_delta: f64 },
    OnEvent { event: crate::event::Event },
    Render,
}
