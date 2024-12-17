use std::collections::HashMap;

use log::warn;
use webapi_model::{Configuration, ConfigurationSchema, ParameterValue, ValueSchema};

use crate::config::RustmasConfig;
use crate::parameters;

#[derive(Debug, thiserror::Error)]
pub enum LogicError {
    #[error("failed to perform operation: {0}")]
    InternalError(String),

    #[error("no animation selected")]
    NoAnimationSelected,
}

pub struct Logic {
    storage: parameters::Storage,
}

impl Logic {
    fn new(storage: parameters::Storage) -> Self {
        Self { storage }
    }

    pub async fn from(config: &RustmasConfig) -> Result<Self, LogicError> {
        let parameters_storage = parameters::Storage::new(&config.database_path.to_string_lossy())
            .await
            .map_err(|e| LogicError::InternalError(e.to_string()))?;
        Ok(Self::new(parameters_storage))
    }

    pub async fn restore_from_db(
        &self,
        controller: &mut rustmas_animator::Controller,
        configuration: Configuration,
    ) -> Result<Configuration, LogicError> {
        let db_params = self
            .storage
            .fetch(&configuration.id)
            .await
            .map_err(|e| LogicError::InternalError(e.to_string()))?;

        let Some(values) = db_params else {
            match controller.get_parameter_values().await {
                Ok(params) => {
                    let _ = self.storage.save(&configuration.id, &params).await;
                }
                Err(e) => {
                    warn!("Failed to set parameters in DB: {}", e);
                }
            }
            return Ok(configuration);
        };

        let defaults = controller
            .get_parameter_values()
            .await
            .map_err(|e| LogicError::InternalError(e.to_string()))?;

        let values = reconcile_parameters(defaults, values, &configuration.schema);
        let _ = controller.set_parameters(&values).await;
        Ok(Configuration {
            values,
            ..configuration
        })
    }

    pub async fn save(&self, controller: &rustmas_animator::Controller) -> Result<(), LogicError> {
        let parameter_values = controller
            .get_parameter_values()
            .await
            .map_err(|e| LogicError::InternalError(e.to_string()))?;

        let animation = controller.current_animation().await;
        let animation_id = animation
            .as_ref()
            .map(|a| a.animation_id())
            .ok_or(LogicError::NoAnimationSelected)?;

        self.storage
            .save(animation_id, &parameter_values)
            .await
            .map_err(|e| LogicError::InternalError(e.to_string()))
    }

    pub async fn reset(
        &self,
        controller: &mut rustmas_animator::Controller,
    ) -> Result<Configuration, LogicError> {
        let animation = controller
            .current_animation()
            .await
            .ok_or(LogicError::NoAnimationSelected)?;

        match self.storage.fetch(animation.animation_id()).await {
            Ok(Some(params)) => controller
                .set_parameters(&params)
                .await
                .map_err(|e| LogicError::InternalError(e.to_string())),
            Ok(None) => Err(LogicError::InternalError(
                "No parameters stored for this animation".to_string(),
            )),
            Err(e) => Err(LogicError::InternalError(e.to_string())),
        }
    }
}

fn reconcile_parameters(
    defaults: HashMap<String, ParameterValue>,
    mut values: HashMap<String, ParameterValue>,
    schema: &ConfigurationSchema,
) -> HashMap<String, ParameterValue> {
    let mut schema_map = schema
        .parameters
        .iter()
        .map(|s| (s.id.clone(), s.value.clone()))
        .collect::<HashMap<_, _>>();

    defaults
        .into_iter()
        .map(|(id, v)| {
            let new_value = if let (Some(param_value), Some(schema_value)) =
                (values.remove(&id), schema_map.remove(&id))
            {
                match (param_value, schema_value) {
                    (n @ ParameterValue::Number(_), ValueSchema::Speed) => n,
                    (n @ ParameterValue::Number(_), ValueSchema::Percentage) => n,
                    (ParameterValue::Number(n), ValueSchema::Number { min, max, step }) => {
                        ParameterValue::Number(
                            ((n.clamp(min, max) - min) / step).round() * step + min,
                        )
                    }
                    (c @ ParameterValue::Color(_), ValueSchema::Color) => c,
                    (ParameterValue::EnumOption(e), ValueSchema::Enum { values }) => {
                        if values.into_iter().any(|en| en.value == e) {
                            ParameterValue::EnumOption(e)
                        } else {
                            v
                        }
                    }
                    _ => v,
                }
            } else {
                v
            };
            (id, new_value)
        })
        .collect()
}
