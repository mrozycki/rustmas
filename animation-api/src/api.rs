use std::error::Error;

use lightfx::parameter_schema::ParametersSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimationError {
    message: String,
}

pub trait AnimationParameters {
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
}

pub trait Animation: AnimationParameters + Sync + Send {
    fn frame(&mut self, time: f64) -> lightfx::Frame;
}

pub trait StepAnimation: AnimationParameters + Sync + Send {
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
