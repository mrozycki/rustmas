use std::collections::HashMap;

use animation_api::{event::Event, schema};
use animation_wasm_bindings::host::HostedPlugin;
use animation_wrapper::{config::PluginConfig, unwrap};
use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::plugin::{AnimationPluginError, Plugin};

pub struct WasmPlugin {
    inner: Mutex<HostedPlugin>,
    plugin_config: PluginConfig,
}

impl WasmPlugin {
    pub async fn new(
        config: PluginConfig,
        points: Vec<(f64, f64, f64)>,
    ) -> Result<Self, AnimationPluginError> {
        let plugin = if config
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "crab")
        {
            let reader = unwrap::reader_from_crab(&config.path())
                .map_err(|e| AnimationPluginError::CommunicationError(Box::new(e)))?;
            HostedPlugin::from_reader(reader, points).await
        } else {
            HostedPlugin::new(&config.executable_path(), points).await
        }
        .map_err(|e| AnimationPluginError::CommunicationError(Box::new(e)))?;

        Ok(Self {
            inner: Mutex::new(plugin),
            plugin_config: config,
        })
    }
}

#[async_trait]
impl Plugin for WasmPlugin {
    fn plugin_config(&self) -> &PluginConfig {
        &self.plugin_config
    }

    async fn configuration(&self) -> Result<schema::Configuration, AnimationPluginError> {
        Ok(schema::Configuration {
            id: self.plugin_config.animation_id().to_owned(),
            name: self.plugin_config.animation_name().to_owned(),
            schema: self.get_schema().await?,
            values: self.get_parameters().await?,
        })
    }

    async fn update(&mut self, time_delta: f64) -> Result<(), AnimationPluginError> {
        self.inner
            .lock()
            .await
            .update(time_delta)
            .await
            .map_err(|e| AnimationPluginError::CommunicationError(Box::new(e)))
    }

    async fn render(&self) -> Result<lightfx::Frame, AnimationPluginError> {
        let frame = self
            .inner
            .lock()
            .await
            .render()
            .await
            .map_err(|e| AnimationPluginError::CommunicationError(Box::new(e)))?;

        Ok(frame
            .into_iter()
            .map(|c| lightfx::Color::rgb(c.r, c.g, c.b))
            .into())
    }

    async fn get_schema(&self) -> Result<schema::ConfigurationSchema, AnimationPluginError> {
        self.inner
            .lock()
            .await
            .get_schema()
            .await
            .map_err(|e| AnimationPluginError::CommunicationError(Box::new(e)))
    }

    async fn set_parameters(
        &mut self,
        values: &HashMap<String, schema::ParameterValue>,
    ) -> Result<(), AnimationPluginError> {
        self.inner
            .lock()
            .await
            .set_parameters(values)
            .await
            .map_err(|e| AnimationPluginError::CommunicationError(Box::new(e)))
    }

    async fn get_parameters(
        &self,
    ) -> Result<HashMap<String, schema::ParameterValue>, AnimationPluginError> {
        self.inner
            .lock()
            .await
            .get_parameters()
            .await
            .map_err(|e| AnimationPluginError::CommunicationError(Box::new(e)))
    }

    async fn get_fps(&self) -> Result<f64, AnimationPluginError> {
        self.inner
            .lock()
            .await
            .get_fps()
            .await
            .map_err(|e| AnimationPluginError::CommunicationError(Box::new(e)))
    }

    async fn send_event(&self, _event: Event) -> Result<(), AnimationPluginError> {
        self.inner
            .lock()
            .await
            .send_event(_event)
            .await
            .map_err(|e| AnimationPluginError::CommunicationError(Box::new(e)))
    }
}
