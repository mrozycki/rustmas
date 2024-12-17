use std::collections::HashMap;

pub use animation_api::event::Event;
pub use animation_api::schema::{
    Configuration, ConfigurationSchema, ParameterSchema, ParameterValue, ValueSchema,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Error { error: String },
    Success(T),
}

impl<T> From<ApiResponse<T>> for Result<T, String> {
    fn from(val: ApiResponse<T>) -> Self {
        match val {
            ApiResponse::Success(t) => Ok(t),
            ApiResponse::Error { error } => Err(error),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Parameter {
    schema: ParameterSchema,
    value: ParameterValue,
}

#[derive(Serialize, Deserialize)]
pub struct SwitchAnimationRequest {
    pub animation_id: String,
    pub params: Option<HashMap<String, ParameterValue>>,
}

#[derive(Serialize, Deserialize)]
pub struct SwitchAnimationResponse {
    pub animation: Configuration,
}

#[derive(Serialize, Deserialize)]
pub struct GetEventGeneratorSchemaResponse {
    pub event_generators: Vec<Configuration>,
}

#[derive(Serialize, Deserialize)]
pub struct SetEventGeneratorParametersRequest {
    pub event_generators: HashMap<String, HashMap<String, ParameterValue>>,
}

#[derive(Serialize, Deserialize)]
pub struct SendEventRequest {
    pub event: Event,
}

#[derive(Serialize, Deserialize)]
pub struct GetParametersResponse {
    pub animation: Option<Configuration>,
}

#[derive(Serialize, Deserialize)]
pub struct SetAnimationParametersRequest {
    pub values: HashMap<String, ParameterValue>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Animation {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListAnimationsResponse {
    pub animations: Vec<Animation>,
    pub current_animation_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetPointsResponse {
    pub points: Vec<(f32, f32, f32)>,
}
