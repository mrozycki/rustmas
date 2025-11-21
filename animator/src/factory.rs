use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use animation_api::plugin_config::PluginConfig;
use animation_wasm_bindings::host::{AnimationPlugin, AnimationPluginError};
use animation_wrapper::{PluginConfigError, unwrap};
use itertools::Itertools;
use log::info;

use crate::ControllerConfig;

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
        let plugins = self
            .plugin_dir
            .read_dir()
            .map_err(|e| AnimationFactoryError::InternalError {
                reason: format!("Failed to read plugin directory: {e}"),
            })?
            .filter_map(|d| d.ok())
            .filter(|d| d.file_name().to_str().is_some_and(|d| d.ends_with(".crab")))
            .filter_map(|d| Some(d.path().to_owned()).zip(unwrap::unwrap_plugin(d.path()).ok()))
            .map(|(path, manifest)| (manifest.id.clone(), PluginConfig { manifest, path }))
            .collect();

        Ok(plugins)
    }

    pub async fn make_from_path(
        &self,
        path: &Path,
    ) -> Result<AnimationPlugin, AnimationFactoryError> {
        Ok(AnimationPlugin::new(path, self.points.clone()).await?)
    }

    pub async fn install(&self, path: &Path) -> Result<PluginConfig, AnimationFactoryError> {
        let manifest = unwrap::unwrap_plugin(path)
            .map_err(|e| AnimationFactoryError::InvalidPlugin(PluginConfigError::InvalidCrab(e)))?;

        let new_path = self.plugin_dir.join(format!("{}.crab", manifest.id));
        tokio::fs::rename(path, &new_path).await?;

        Ok(PluginConfig {
            manifest,
            path: new_path,
        })
    }

    pub fn points(&self) -> &[(f64, f64, f64)] {
        &self.points
    }
}
