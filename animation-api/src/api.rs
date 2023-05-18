use std::error::Error;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::parameter_schema::ParametersSchema;

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimationError {
    message: String,
}

pub trait Animation {
    fn animation_name(&self) -> &str;

    fn parameter_schema(&self) -> ParametersSchema {
        Default::default()
    }

    fn set_parameters(&mut self, _parameters: serde_json::Value) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn get_parameters(&self) -> serde_json::Value {
        json!({})
    }

    fn get_fps(&self) -> f64 {
        30.0
    }

    fn update(&mut self, delta: f64);
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
    Render,
}
