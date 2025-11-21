use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::schema::{ConfigurationSchema, GetEnumOptions, GetSchema};

#[derive(Serialize, Deserialize, Debug, thiserror::Error)]
#[error("animation error: {message}")]
pub struct AnimationError {
    pub message: String,
}

pub trait Animation {
    type Parameters: GetSchema + DeserializeOwned + Serialize + Default + Clone;
    type CustomTriggers: GetEnumOptions + DeserializeOwned + Serialize + Clone;
    type Wrapped: Animation<Parameters: GetSchema>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self;

    fn new_wrapped(points: Vec<(f64, f64, f64)>) -> Self::Wrapped {
        Self::Wrapped::new(points)
    }

    fn get_schema(&self) -> ConfigurationSchema
    where
        Self::Parameters: GetSchema,
    {
        ConfigurationSchema {
            parameters: Self::Parameters::schema(),
            custom_triggers: Self::CustomTriggers::enum_options(),
        }
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
