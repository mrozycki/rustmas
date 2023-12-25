use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use log::{info, warn};
use serde::Deserialize;
use thiserror::Error;

use crate::jsonrpc_animation::{AnimationPlugin, AnimationPluginError, JsonRpcEndpoint};

#[derive(Debug, Error)]
pub enum AnimationFactoryError {
    #[error("internal error: {reason}")]
    InternalError { reason: String },

    #[error("animation not found")]
    AnimationNotFound,

    #[error("malformed animation manifest: {reason}")]
    InvalidManifest { reason: String },

    #[error("animation failed to start: {0}")]
    AnimationFailedToStart(#[from] std::io::Error),

    #[error(transparent)]
    AnimationError(#[from] AnimationPluginError),
}

#[derive(Clone, Deserialize)]
pub struct PluginManifest {
    pub display_name: String,
}

pub struct Plugin {
    pub manifest: PluginManifest,
    pub path: PathBuf,
}

impl Plugin {
    fn executable_path(&self) -> PathBuf {
        #[cfg(windows)]
        let executable_name = "plugin.exe";

        #[cfg(not(windows))]
        let executable_name = "plugin";

        self.path.join(executable_name)
    }

    fn start(&self) -> std::io::Result<JsonRpcEndpoint> {
        JsonRpcEndpoint::new(self.executable_path())
    }

    fn is_executable(&self) -> bool {
        Command::new(self.executable_path())
            .stdout(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .is_ok()
    }
}

pub struct AnimationFactory {
    plugin_dir: PathBuf,
    plugins: HashMap<String, Plugin>,
    points: Vec<(f64, f64, f64)>,
}

impl AnimationFactory {
    pub fn new<P: AsRef<Path>>(plugin_dir: P, points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            plugin_dir: plugin_dir.as_ref().to_owned(),
            plugins: HashMap::new(),
            points,
        }
    }

    pub fn discover(&mut self) -> Result<(), AnimationFactoryError> {
        let (valid_plugins, invalid_plugins) = glob::glob(&format!(
            "{}/*/manifest.json",
            self.plugin_dir
                .to_str()
                .ok_or(AnimationFactoryError::InternalError {
                    reason: "Plugins directory path is not valid UTF-8".into()
                })?
        ))
        .map_err(|e| AnimationFactoryError::InternalError {
            reason: format!("Pattern error: {}", e),
        })?
        .map(|path| {
            path.map_err(|e| AnimationFactoryError::InternalError {
                reason: format!("Glob error: {}", e),
            })
            .and_then(|path| -> Result<_, AnimationFactoryError> {
                let manifest: PluginManifest =
                    serde_json::from_slice(&std::fs::read(&path).map_err(|e| {
                        AnimationFactoryError::InvalidManifest {
                            reason: format!("IO error: {}", e),
                        }
                    })?)
                    .map_err(|e| {
                        AnimationFactoryError::InvalidManifest {
                            reason: e.to_string(),
                        }
                    })?;
                let path = path.parent().unwrap().to_owned();
                let id = path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .ok_or(AnimationFactoryError::InternalError {
                        reason: "Plugins directory path is not valid UTF-8".into(),
                    })?
                    .to_owned();
                Ok((id, Plugin { path, manifest }))
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .partition::<HashMap<_, _>, _>(|(_id, plugin)| plugin.is_executable());

        if !invalid_plugins.is_empty() {
            warn!("Discovered {} plugins which are not executable. Please make sure the animations were built and have correct permissions.", invalid_plugins.len());
        }

        self.plugins = valid_plugins;

        Ok(())
    }

    pub fn list(&self) -> &HashMap<String, Plugin> {
        &self.plugins
    }

    pub fn make(&self, name: &str) -> Result<AnimationPlugin, AnimationFactoryError> {
        let Some(plugin) = self.plugins.get(name) else {
            return Err(AnimationFactoryError::AnimationNotFound);
        };

        info!(
            "Trying to start plugin app '{}' from directory '{:?}'",
            name, plugin.path
        );

        Ok(AnimationPlugin::new(plugin.start()?, self.points.clone())?)
    }

    pub fn points(&self) -> &[(f64, f64, f64)] {
        &self.points
    }
}
