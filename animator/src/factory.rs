use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use animation_wrapper::{
    config::{PluginConfig, PluginConfigError, PluginType},
    unwrap,
};
use log::warn;

use crate::{
    jsonrpc::JsonRpcPlugin,
    plugin::{AnimationPluginError, Plugin},
    wasm::WasmPlugin,
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

        let crab_plugins = self
            .plugin_dir
            .read_dir()
            .map_err(|e| AnimationFactoryError::InternalError {
                reason: format!("Failed to read plugin directory: {e}"),
            })?
            .filter_map(|d| d.ok())
            .filter(|d| d.file_name().to_str().is_some_and(|d| d.ends_with(".crab")))
            .filter_map(|d| unwrap::unwrap_plugin(&d.path()).ok())
            .map(|p| (p.animation_id().to_owned(), p));

        self.available_plugins = valid_plugins;
        self.available_plugins.extend(crab_plugins);

        Ok(())
    }

    pub fn list(&self) -> &HashMap<String, PluginConfig> {
        &self.available_plugins
    }

    pub async fn make(&self, name: &str) -> Result<Box<dyn Plugin>, AnimationFactoryError> {
        let Some(plugin) = self.available_plugins.get(name) else {
            return Err(AnimationFactoryError::AnimationNotFound);
        };

        match plugin.plugin_type() {
            PluginType::Native => Ok(Box::new(
                JsonRpcPlugin::new(plugin.clone(), self.points.clone()).await?,
            )),
            PluginType::Wasm => Ok(Box::new(
                WasmPlugin::new(plugin.clone(), self.points.clone()).await?,
            )),
        }
    }

    pub fn points(&self) -> &[(f64, f64, f64)] {
        &self.points
    }
}
