use std::{
    collections::HashMap,
    ffi::OsStr,
    io::{BufRead, BufReader, BufWriter, Write},
    process::{Child, Command, Stdio},
    result::Result,
    sync::Mutex,
};

use animation_api::{
    schema::{Configuration, ConfigurationSchema, ParameterValue},
    AnimationError, JsonRpcMessage, JsonRpcMethod, JsonRpcResponse, JsonRpcResult,
};
use log::error;
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::plugin::{AnimationPluginError, Plugin, PluginConfig};

#[derive(Debug, Error)]
pub enum JsonRpcEndpointError {
    #[error("endpoint process exited")]
    ProcessExited,

    #[error("endpoint returned invalid response: {0}")]
    InvalidResponse(String),
}

pub struct JsonRpcEndpoint {
    child_process: Mutex<Child>,
    id: Mutex<usize>,
}

impl JsonRpcEndpoint {
    pub fn new<P: AsRef<OsStr>>(executable_path: P) -> std::io::Result<Self> {
        let mut command = Command::new(executable_path);

        let child_process = command
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;

        Ok(Self {
            child_process: Mutex::new(child_process),
            id: Mutex::new(0),
        })
    }

    fn send(
        writer: &mut impl std::io::Write,
        message: JsonRpcMessage<JsonRpcMethod>,
    ) -> std::io::Result<()> {
        let mut writer = BufWriter::new(writer);
        serde_json::to_writer(&mut writer, &message)?;
        writer.write_all(b"\n")?;
        Ok(())
    }

    fn receive<Res>(
        reader: &mut impl std::io::Read,
    ) -> Result<JsonRpcResponse<Res, AnimationError>, JsonRpcEndpointError>
    where
        Res: DeserializeOwned,
    {
        let mut reader = BufReader::new(reader);
        let mut buffer = String::new();
        reader
            .read_line(&mut buffer)
            .map_err(|e| JsonRpcEndpointError::InvalidResponse(e.to_string()))
            .and_then(|count| {
                if count == 0 {
                    Err(JsonRpcEndpointError::ProcessExited)
                } else {
                    Ok(())
                }
            })?;
        serde_json::from_str(&buffer)
            .map_err(|e| JsonRpcEndpointError::InvalidResponse(e.to_string()))
    }

    pub fn send_message<Res>(
        &self,
        payload: JsonRpcMethod,
    ) -> Result<JsonRpcResult<Res, AnimationError>, JsonRpcEndpointError>
    where
        Res: DeserializeOwned,
    {
        let mut child = self.child_process.lock().unwrap();

        let mut id = self.id.lock().unwrap();
        *id += 1;
        Self::send(
            child.stdin.as_mut().unwrap(),
            JsonRpcMessage {
                id: Some(*id),
                payload,
            },
        )
        .map_err(|_| JsonRpcEndpointError::ProcessExited)?;

        Self::receive(child.stdout.as_mut().unwrap()).map(|response| response.result)
    }

    pub fn send_notification(&self, payload: JsonRpcMethod) -> Result<(), JsonRpcEndpointError> {
        let mut child = self.child_process.lock().unwrap();
        Self::send(
            child.stdin.as_mut().unwrap(),
            JsonRpcMessage { id: None, payload },
        )
        .map_err(|_| JsonRpcEndpointError::ProcessExited)
    }
}

pub struct JsonRpcPlugin {
    plugin_config: PluginConfig,
    endpoint: JsonRpcEndpoint,
}

impl JsonRpcPlugin {
    pub fn new(
        config: PluginConfig,
        points: Vec<(f64, f64, f64)>,
    ) -> Result<Self, AnimationPluginError> {
        let endpoint = config
            .start()
            .map_err(|e| AnimationPluginError::CommunicationError(Box::new(e)))?;

        match endpoint.send_message::<()>(JsonRpcMethod::Initialize { points }) {
            Ok(JsonRpcResult::Result(_)) => Ok(Self {
                endpoint,
                plugin_config: config,
            }),
            Ok(JsonRpcResult::Error(e)) => Err(AnimationPluginError::AnimationError(e.data)),
            Err(e) => Err(AnimationPluginError::CommunicationError(Box::new(e))),
        }
    }
}

impl Plugin for JsonRpcPlugin {
    fn plugin_config(&self) -> &PluginConfig {
        &self.plugin_config
    }

    fn configuration(&self) -> Result<Configuration, AnimationPluginError> {
        Ok(Configuration {
            id: self.plugin_config().animation_id().to_owned(),
            name: self.plugin_config().animation_name().to_owned(),
            schema: self.get_schema()?,
            values: self.get_parameters()?,
        })
    }

    fn update(&mut self, time_delta: f64) -> Result<(), AnimationPluginError> {
        if let Err(e) = self
            .endpoint
            .send_notification(JsonRpcMethod::Update { time_delta })
        {
            Err(AnimationPluginError::CommunicationError(Box::new(e)))
        } else {
            Ok(())
        }
    }

    fn render(&self) -> Result<lightfx::Frame, AnimationPluginError> {
        match self.endpoint.send_message(JsonRpcMethod::Render) {
            Ok(JsonRpcResult::Result(frame)) => Ok(frame),
            Ok(JsonRpcResult::Error(e)) => Err(AnimationPluginError::AnimationError(e.data)),
            Err(e) => Err(AnimationPluginError::CommunicationError(Box::new(e))),
        }
    }

    fn animation_name(&self) -> Result<String, AnimationPluginError> {
        match self.endpoint.send_message(JsonRpcMethod::AnimationName) {
            Ok(JsonRpcResult::Result(name)) => Ok(name),
            Ok(JsonRpcResult::Error(e)) => Err(AnimationPluginError::AnimationError(e.data)),
            Err(e) => Err(AnimationPluginError::CommunicationError(Box::new(e))),
        }
    }

    fn get_schema(&self) -> Result<ConfigurationSchema, AnimationPluginError> {
        match self.endpoint.send_message(JsonRpcMethod::ParameterSchema) {
            Ok(JsonRpcResult::Result(schema)) => Ok(schema),
            Ok(JsonRpcResult::Error(e)) => Err(AnimationPluginError::AnimationError(e.data)),
            Err(e) => Err(AnimationPluginError::CommunicationError(Box::new(e))),
        }
    }

    fn set_parameters(
        &mut self,
        params: &HashMap<String, ParameterValue>,
    ) -> Result<(), AnimationPluginError> {
        match self.endpoint.send_message(JsonRpcMethod::SetParameters {
            params: params.clone(),
        }) {
            Ok(JsonRpcResult::Result(())) => Ok(()),
            Ok(JsonRpcResult::Error(e)) => Err(AnimationPluginError::AnimationError(e.data)),
            Err(e) => Err(AnimationPluginError::CommunicationError(Box::new(e))),
        }
    }

    fn get_parameters(&self) -> Result<HashMap<String, ParameterValue>, AnimationPluginError> {
        match self.endpoint.send_message(JsonRpcMethod::GetParameters) {
            Ok(JsonRpcResult::Result(parameters)) => Ok(parameters),
            Ok(JsonRpcResult::Error(e)) => Err(AnimationPluginError::AnimationError(e.data)),
            Err(e) => Err(AnimationPluginError::CommunicationError(Box::new(e))),
        }
    }

    fn get_fps(&self) -> Result<f64, AnimationPluginError> {
        match self.endpoint.send_message(JsonRpcMethod::GetFps) {
            Ok(JsonRpcResult::Result(fps)) => Ok(fps),
            Ok(JsonRpcResult::Error(e)) => Err(AnimationPluginError::AnimationError(e.data)),
            Err(e) => Err(AnimationPluginError::CommunicationError(Box::new(e))),
        }
    }

    fn send_event(&self, event: animation_api::event::Event) -> Result<(), AnimationPluginError> {
        match self.endpoint.send_message(JsonRpcMethod::OnEvent { event }) {
            Ok(JsonRpcResult::Result(())) => Ok(()),
            Ok(JsonRpcResult::Error(e)) => Err(AnimationPluginError::AnimationError(e.data)),
            Err(e) => Err(AnimationPluginError::CommunicationError(Box::new(e))),
        }
    }
}
