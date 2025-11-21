use std::collections::HashMap;
use std::path::Path;

use log::warn;
use rustmas_animator::{AnimationFactory, AnimationFactoryError};
use webapi_model::{Animation, Configuration, ListAnimationsResponse, ParameterValue};

use crate::animations;
use crate::config::RustmasConfig;
use crate::db::SharedDbConnection;
use crate::parameters;

#[derive(Debug, thiserror::Error)]
pub enum LogicError {
    #[error("failed to perform operation: {0}")]
    InternalError(String),

    #[error("no such animation: {0}")]
    NoSuchAnimation(String),

    #[error("selected animation plugin exists, but is not valid: {0}")]
    InvalidAnimation(#[from] AnimationFactoryError),

    #[error("no animation selected")]
    NoAnimationSelected,
}

pub struct Logic {
    storage: animations::Storage,
    animation_factory: AnimationFactory,
}

impl Logic {
    pub fn new(storage: animations::Storage, animation_factory: AnimationFactory) -> Self {
        Self {
            storage,
            animation_factory,
        }
    }

    pub fn from(conn: SharedDbConnection, config: &RustmasConfig) -> anyhow::Result<Self> {
        Ok(Self::new(
            animations::Storage::new(conn),
            AnimationFactory::from_config(&config.controller)?,
        ))
    }

    pub async fn reload(
        &self,
        controller: &mut rustmas_animator::Controller,
        parameters: &parameters::Logic,
    ) -> Result<Configuration, LogicError> {
        let animation_id = controller
            .current_animation()
            .await
            .map(|a| a.animation_id)
            .ok_or(LogicError::NoAnimationSelected)?;

        self.switch(&animation_id, None, controller, parameters)
            .await
    }

    pub async fn switch(
        &self,
        animation_id: &str,
        initial_parameters: Option<HashMap<String, ParameterValue>>,
        controller: &mut rustmas_animator::Controller,
        parameters: &parameters::Logic,
    ) -> Result<Configuration, LogicError> {
        let db_plugin = self
            .storage
            .fetch_by_id(animation_id)
            .await
            .map_err(|e| LogicError::InternalError(e.to_string()))?
            .ok_or_else(|| LogicError::NoSuchAnimation(animation_id.to_owned()))?;

        let plugin = self
            .animation_factory
            .make_from_path(&db_plugin.path)
            .await?;

        let configuration = controller
            .switch_animation(plugin)
            .await
            .map_err(|e| LogicError::InternalError(e.to_string()))?;

        if let Some(values) = initial_parameters {
            let _ = controller.set_parameters(&values).await;
            Ok(Configuration {
                values,
                ..configuration
            })
        } else {
            parameters
                .restore_from_db(controller, configuration)
                .await
                .map_err(|e| LogicError::InternalError(e.to_string()))
        }
    }

    pub async fn turn_off(&self, controller: &mut rustmas_animator::Controller) {
        controller.turn_off().await;
    }

    pub async fn discover(
        &self,
        controller: &rustmas_animator::Controller,
    ) -> Result<ListAnimationsResponse, LogicError> {
        let animations = self
            .animation_factory
            .discover()
            .map_err(|e| LogicError::InternalError(e.to_string()))?;

        for (id, config) in animations.iter() {
            let _ = self.storage.install(config).await.inspect_err(|e| {
                warn!(
                    "Failed to install animation plugin with ID {id}, from path {:?}: {e}",
                    config.path
                )
            });
        }

        self.list(controller).await
    }

    pub async fn install(
        &self,
        controller: &rustmas_animator::Controller,
        path: &Path,
    ) -> Result<ListAnimationsResponse, LogicError> {
        let plugin_config = self.animation_factory.install(path).await?;
        self.storage
            .install(&plugin_config)
            .await
            .map_err(|e| LogicError::InternalError(e.to_string()))?;
        self.list(controller).await
    }

    pub async fn remove(
        &self,
        controller: &rustmas_animator::Controller,
        animation_id: &str,
    ) -> Result<ListAnimationsResponse, LogicError> {
        let path = self
            .storage
            .delete(animation_id)
            .await
            .map_err(|e| LogicError::InternalError(e.to_string()))?;

        tokio::fs::remove_file(path)
            .await
            .map_err(|e| LogicError::InternalError(e.to_string()))?;

        self.list(controller).await
    }

    pub async fn list(
        &self,
        controller: &rustmas_animator::Controller,
    ) -> Result<ListAnimationsResponse, LogicError> {
        Ok(ListAnimationsResponse {
            animations: self
                .storage
                .fetch_all()
                .await
                .map_err(|e| LogicError::InternalError(e.to_string()))?
                .into_iter()
                .map(|db_plugin| Animation {
                    id: db_plugin.animation_id,
                    name: db_plugin.manifest.display_name,
                })
                .collect(),
            current_animation_id: controller.current_animation().await.map(|a| a.animation_id),
        })
    }
}
