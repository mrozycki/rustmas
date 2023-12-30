use std::{
    error::Error,
    path::PathBuf,
    process::{Command, Stdio},
};

use animation_api::{schema::ConfigurationSchema, AnimationError};
use log::info;
use serde::Deserialize;

use crate::jsonrpc::JsonRpcEndpoint;

#[derive(Debug, thiserror::Error)]
pub enum PluginConfigError {
    #[error("Failed to parse manifest: {reason}")]
    InvalidManifest { reason: String },

    #[error("Directory containing plugin has non UTF-8 name")]
    NonUtf8DirectoryName,
}

#[derive(Clone, Deserialize)]
pub struct PluginManifest {
    display_name: String,
}

#[derive(Clone)]
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

    fn executable_path(&self) -> PathBuf {
        #[cfg(windows)]
        let executable_name = "plugin.exe";

        #[cfg(not(windows))]
        let executable_name = "plugin";

        self.path.join(executable_name)
    }

    pub fn start(&self) -> std::io::Result<JsonRpcEndpoint> {
        info!(
            "Trying to start plugin app '{}' from directory '{:?}'",
            self.animation_id, self.path
        );
        JsonRpcEndpoint::new(self.executable_path())
    }

    pub fn is_executable(&self) -> bool {
        Command::new(self.executable_path())
            .stdout(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .is_ok()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AnimationPluginError {
    #[error("animation error: {0}")]
    AnimationError(#[from] AnimationError),

    #[error("Communication error: {0}")]
    CommunicationError(#[from] Box<dyn Error>),
}

pub trait Plugin {
    fn config(&self) -> &PluginConfig;
    fn update(&mut self, time_delta: f64) -> Result<(), AnimationPluginError>;
    fn render(&self) -> Result<lightfx::Frame, AnimationPluginError>;
    fn animation_name(&self) -> Result<String, AnimationPluginError>;
    fn get_schema(&self) -> Result<ConfigurationSchema, AnimationPluginError>;
    fn set_parameters(&mut self, params: serde_json::Value) -> Result<(), AnimationPluginError>;
    fn get_parameters(&self) -> Result<serde_json::Value, AnimationPluginError>;
    fn get_fps(&self) -> Result<f64, AnimationPluginError>;
    fn send_event(&self, event: animation_api::event::Event) -> Result<(), AnimationPluginError>;
}
