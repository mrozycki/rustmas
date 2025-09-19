use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use animation_wrapper::{
    config::{PluginConfig, PluginConfigError, PluginType},
    unwrap,
};
use itertools::Itertools;
use log::{info, warn};

use crate::{
    ControllerConfig,
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

    #[error("invalid points file: {0}")]
    InvalidPointsFile(#[from] csv::Error),
}

pub struct AnimationFactory {
    plugin_dir: PathBuf,
    points: Vec<(f64, f64, f64)>,
}

pub fn points_from_path(path: &Path) -> Result<Vec<(f64, f64, f64)>, AnimationFactoryError> {
    let points: Vec<_> = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?
        .deserialize()
        .try_collect()?;

    info!(
        "Loaded {} points from {}",
        points.len(),
        path.to_string_lossy()
    );
    Ok(points)
}

impl AnimationFactory {
    pub fn from_config(config: &ControllerConfig) -> Result<Self, AnimationFactoryError> {
        Ok(Self {
            plugin_dir: config.plugin_path.clone(),
            points: points_from_path(&config.points_path)?,
        })
    }

    pub fn discover(&self) -> Result<HashMap<String, PluginConfig>, AnimationFactoryError> {
        let (mut valid_plugins, invalid_plugins) = glob::glob(&format!(
            "{}/*/manifest.json",
            self.plugin_dir
                .to_str()
                .ok_or(AnimationFactoryError::InternalError {
                    reason: "Plugins directory path is not valid UTF-8".into()
                })?
        ))
        .map_err(|e| AnimationFactoryError::InternalError {
            reason: format!("Pattern error: {e}"),
        })?
        .map(|path| {
            path.map_err(|e| AnimationFactoryError::InternalError {
                reason: format!("Glob error: {e}"),
            })
            .and_then(|path| -> Result<_, AnimationFactoryError> {
                let path = path.parent().unwrap().to_owned();
                let plugin = PluginConfig::new(path)?;
                let id = plugin.animation_id.clone();
                Ok((id, plugin))
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .partition::<HashMap<_, _>, _>(|(_id, plugin)| plugin.is_executable());

        if !invalid_plugins.is_empty() {
            warn!(
                "Discovered {} plugins which are not executable. Please make sure the animations were built and have correct permissions.",
                invalid_plugins.len()
            );
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
            .map(|p| (p.animation_id.clone(), p));

        valid_plugins.extend(crab_plugins);

        Ok(valid_plugins)
    }

    pub async fn make_from_path(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Plugin>, AnimationFactoryError> {
        let config = if path
            .file_name()
            .and_then(|f| f.to_str())
            .is_some_and(|f| f.ends_with(".crab"))
        {
            unwrap::unwrap_plugin(&path).map_err(|e| {
                AnimationFactoryError::InvalidPlugin(PluginConfigError::InvalidCrab(e))
            })?
        } else {
            PluginConfig::new(path).map_err(AnimationFactoryError::InvalidPlugin)?
        };

        match config.manifest.plugin_type {
            PluginType::Native => Ok(Box::new(
                JsonRpcPlugin::new(config, self.points.clone()).await?,
            )),
            PluginType::Wasm => Ok(Box::new(
                WasmPlugin::new(config, self.points.clone()).await?,
            )),
        }
    }

    pub fn points(&self) -> &[(f64, f64, f64)] {
        &self.points
    }
}
