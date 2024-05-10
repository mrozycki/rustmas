use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use log::warn;

use crate::{
    jsonrpc::JsonRpcPlugin,
    plugin::{AnimationPluginError, PluginConfig, PluginConfigError},
};

#[derive(Debug, thiserror::Error)]
pub enum AnimationFactoryError {
    #[error("internal error: {reason}")]
    InternalError { reason: String },

    #[error("animation not found")]
    AnimationNotFound,

    #[error("problem with plugin configuration: {0}")]
    InvalidPlugin(#[from] PluginConfigError),

    #[error("animation failed to start: {0}")]
    AnimationFailedToStart(#[from] std::io::Error),

    #[error(transparent)]
    AnimationError(#[from] AnimationPluginError),
}

pub struct AnimationFactory {
    plugin_dir: PathBuf,
    available_plugins: HashMap<String, PluginConfig>,
    points: Vec<(f64, f64, f64)>,
}

impl AnimationFactory {
    pub fn new<P: AsRef<Path>>(plugin_dir: P, points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            plugin_dir: plugin_dir.as_ref().to_owned(),
            available_plugins: HashMap::new(),
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
                let path = path.parent().unwrap().to_owned();
                let plugin = PluginConfig::new(path)?;
                let id = plugin.animation_id().to_owned();
                Ok((id, plugin))
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .partition::<HashMap<_, _>, _>(|(_id, plugin)| plugin.is_executable());

        if !invalid_plugins.is_empty() {
            warn!("Discovered {} plugins which are not executable. Please make sure the animations were built and have correct permissions.", invalid_plugins.len());
        }

        self.available_plugins = valid_plugins;

        Ok(())
    }

    pub fn list(&self) -> &HashMap<String, PluginConfig> {
        &self.available_plugins
    }

    pub fn make(&self, name: &str) -> Result<JsonRpcPlugin, AnimationFactoryError> {
        let Some(plugin) = self.available_plugins.get(name) else {
            return Err(AnimationFactoryError::AnimationNotFound);
        };

        Ok(JsonRpcPlugin::new(plugin.clone(), self.points.clone())?)
    }

    pub fn points(&self) -> &[(f64, f64, f64)] {
        &self.points
    }
}
