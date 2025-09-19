use std::{collections::HashMap, error::Error};

use animation_api::{
    AnimationError,
    schema::{Configuration, ConfigurationSchema, ParameterValue},
};
use animation_wrapper::config::PluginConfig;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum AnimationPluginError {
    #[error("animation error: {0}")]
    AnimationError(#[from] AnimationError),

    #[error("Communication error: {0}")]
    CommunicationError(#[from] Box<dyn Error + Send + Sync>),
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn plugin_config(&self) -> &PluginConfig;
    async fn configuration(&self) -> Result<Configuration, AnimationPluginError>;
    async fn update(&mut self, time_delta: f64) -> Result<(), AnimationPluginError>;
    async fn render(&self) -> Result<lightfx::Frame, AnimationPluginError>;
    async fn get_schema(&self) -> Result<ConfigurationSchema, AnimationPluginError>;
    async fn set_parameters(
        &mut self,
        params: &HashMap<String, ParameterValue>,
    ) -> Result<(), AnimationPluginError>;
    async fn get_parameters(&self)
    -> Result<HashMap<String, ParameterValue>, AnimationPluginError>;
    async fn get_fps(&self) -> Result<f64, AnimationPluginError>;
    async fn send_event(
        &self,
        event: animation_api::event::Event,
    ) -> Result<(), AnimationPluginError>;
}
