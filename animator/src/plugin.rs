use std::{
    collections::HashMap,
    error::Error,
    path::PathBuf,
    process::{Command, Stdio},
};

use animation_api::{
    schema::{Configuration, ConfigurationSchema, ParameterValue},
    AnimationError,
};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum PluginConfigError {
    #[error("Failed to parse manifest: {reason}")]
    InvalidManifest { reason: String },

    #[error("Directory containing plugin has non UTF-8 name")]
    NonUtf8DirectoryName,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    #[default]
    Native,
    Wasm,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifest {
    display_name: String,
    #[serde(default)]
    plugin_type: PluginType,
}

#[derive(Debug, Clone)]
pub struct PluginConfig {
    animation_id: String,
    manifest: PluginManifest,
    path: PathBuf,
}

impl PluginConfig {
    pub fn new(path: PathBuf) -> Result<Self, PluginConfigError> {
        let manifest: PluginManifest =
            serde_json::from_slice(&std::fs::read(path.join("manifest.json")).map_err(|e| {
                PluginConfigError::InvalidManifest {
                    reason: format!("IO error: {}", e),
                }
            })?)
            .map_err(|e| PluginConfigError::InvalidManifest {
                reason: e.to_string(),
            })?;

        let animation_id = path
            .file_name()
            .unwrap()
            .to_str()
            .ok_or(PluginConfigError::NonUtf8DirectoryName)?
            .to_owned();

        Ok(Self {
            animation_id,
            manifest,
            path,
        })
    }

    pub fn animation_id(&self) -> &str {
        &self.animation_id
    }

    pub fn animation_name(&self) -> &str {
        &self.manifest.display_name
    }

    pub fn plugin_type(&self) -> PluginType {
        self.manifest.plugin_type
    }

    pub fn executable_path(&self) -> PathBuf {
        let executable_name = if self.manifest.plugin_type == PluginType::Wasm {
            "plugin.wasm"
        } else if cfg!(windows) {
            "plugin.exe"
        } else {
            "plugin"
        };

        self.path.join(executable_name)
    }

    pub fn is_executable(&self) -> bool {
        match self.manifest.plugin_type {
            PluginType::Native => Command::new(self.executable_path())
                .stdout(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
                .is_ok(),
            PluginType::Wasm => self.executable_path().exists(),
        }
    }
}

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
