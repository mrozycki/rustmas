use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};

use crate::unwrap::PluginUnwrapError;

#[derive(Debug, thiserror::Error)]
pub enum PluginConfigError {
    #[error("Failed to parse manifest: {reason}")]
    InvalidManifest { reason: String },

    #[error("Failed to unwrap plugin")]
    InvalidCrab(#[from] PluginUnwrapError),

    #[error("Directory containing plugin has non UTF-8 name")]
    NonUtf8DirectoryName,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    #[default]
    Native,
    Wasm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PluginApiVersion {
    #[serde(rename = "0.9")]
    V0_9,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub display_name: String,
    pub author: String,
    #[serde(default)]
    pub plugin_type: PluginType,
    pub api_version: PluginApiVersion,
    pub version: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub animation_id: String,
    pub manifest: PluginManifest,
    pub path: PathBuf,
}

impl PluginConfig {
    pub fn from_path(path: &Path) -> Result<Self, PluginConfigError> {
        let manifest: PluginManifest =
            serde_json::from_slice(&std::fs::read(path.join("manifest.json")).map_err(|e| {
                PluginConfigError::InvalidManifest {
                    reason: format!("IO error: {e}"),
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
            path: path.to_owned(),
        })
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, PluginConfigError> {
        Self::from_path(path.as_ref())
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
