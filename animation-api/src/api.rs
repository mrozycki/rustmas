use std::{collections::HashMap, fmt};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::schema::{ConfigurationSchema, GetSchema, ParameterValue};

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
    type Parameters: GetSchema + DeserializeOwned + Serialize + Default + Clone;

    fn animation_name(&self) -> &str;

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
    AnimationName,
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
