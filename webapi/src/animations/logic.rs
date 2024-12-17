use std::collections::HashMap;

use webapi_model::{Animation, Configuration, ListAnimationsResponse, ParameterValue};

use crate::parameters;

#[derive(Debug, thiserror::Error)]
pub enum LogicError {
    #[error("failed to perform operation: {0}")]
    InternalError(String),
}

pub struct Logic;

impl Logic {
    pub fn new() -> Self {
        Self
    }

    pub async fn reload(
        &self,
        controller: &mut rustmas_animator::Controller,
        parameters: &parameters::Logic,
    ) -> Result<Configuration, LogicError> {
        let configuration = controller
            .reload_animation()
            .await
            .map_err(|e| LogicError::InternalError(e.to_string()))?;

        parameters
            .restore_from_db(controller, configuration)
            .await
            .map_err(|e| LogicError::InternalError(e.to_string()))
    }

    pub async fn switch(
        &self,
        animation_id: &str,
        initial_parameters: Option<HashMap<String, ParameterValue>>,
        controller: &mut rustmas_animator::Controller,
        parameters: &parameters::Logic,
    ) -> Result<Configuration, LogicError> {
        let configuration = controller
            .switch_animation(animation_id)
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
        controller: &mut rustmas_animator::Controller,
    ) -> Result<ListAnimationsResponse, LogicError> {
        controller
            .discover_animations()
            .map_err(|e| LogicError::InternalError(e.to_string()))?;

        Ok(self.list(controller).await)
    }

    pub async fn list(&self, controller: &rustmas_animator::Controller) -> ListAnimationsResponse {
        ListAnimationsResponse {
            animations: controller
                .list_animations()
                .iter()
                .map(|(id, plugin)| Animation {
                    id: id.to_owned(),
                    name: plugin.animation_name().to_owned(),
                })
                .collect(),
            current_animation_id: controller
                .current_animation()
                .await
                .map(|a| a.animation_id().to_owned()),
        }
    }
}
