use core::fmt;
use std::{
    error::Error,
    ffi::OsStr,
    io::{BufRead, BufReader, BufWriter, Write},
    process::{Child, Command, Stdio},
    sync::Mutex,
};

use animation_api::{
    AnimationError, JsonRpcMessage, JsonRpcMethod, JsonRpcResponse, JsonRpcResult,
};
use log::error;
use serde::de::DeserializeOwned;
use serde_json::json;

#[derive(Debug)]
pub enum JsonRpcEndpointError {
    ProcessExited,
    InvalidResponse(Box<dyn Error>),
    Other(Box<dyn Error>),
}

impl fmt::Display for JsonRpcEndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for JsonRpcEndpointError {}

pub struct JsonRpcEndpoint {
    child_process: Mutex<Child>,
    id: Mutex<usize>,
}

impl JsonRpcEndpoint {
    pub fn new<P: AsRef<OsStr>>(executable_path: P) -> Result<Self, JsonRpcEndpointError> {
        let mut command = Command::new(executable_path);

        let child_process = command
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| JsonRpcEndpointError::Other(Box::new(e)))?;

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
        writer.write(b"\n")?;
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
            .map_err(|_| JsonRpcEndpointError::ProcessExited)?;
        Ok(serde_json::from_str(&buffer)
            .map_err(|e| JsonRpcEndpointError::InvalidResponse(Box::new(e)))?)
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

pub struct AnimationPlugin {
    endpoint: JsonRpcEndpoint,
}

impl AnimationPlugin {
    pub fn new(endpoint: JsonRpcEndpoint, points: Vec<(f64, f64, f64)>) -> Self {
        let _ = endpoint.send_notification(JsonRpcMethod::Initialize { points });
        Self { endpoint }
    }

    pub fn update(&mut self, time_delta: f64) {
        let _ = self
            .endpoint
            .send_notification(JsonRpcMethod::Update { time_delta });
    }

    pub fn render(&self) -> lightfx::Frame {
        match self.endpoint.send_message(JsonRpcMethod::Render) {
            Ok(JsonRpcResult::Result(frame)) => frame,
            Ok(JsonRpcResult::Error(e)) => {
                error!("Plugin returned an error: {:?}", e);
                lightfx::Frame::new_black(0)
            }
            Err(e) => {
                error!("Plugin failed to respond: {:?}", e);
                lightfx::Frame::new_black(0)
            }
        }
    }

    pub fn animation_name(&self) -> String {
        match self.endpoint.send_message(JsonRpcMethod::AnimationName) {
            Ok(JsonRpcResult::Result(name)) => name,
            Ok(JsonRpcResult::Error(e)) => {
                error!("Plugin returned an error: {:?}", e);
                "Animation Plugin".into()
            }
            Err(e) => {
                error!("Plugin failed to respond: {:?}", e);
                "Animation Plugin".into()
            }
        }
    }

    pub fn parameter_schema(&self) -> lightfx::parameter_schema::ParametersSchema {
        match self.endpoint.send_message(JsonRpcMethod::ParameterSchema) {
            Ok(JsonRpcResult::Result(schema)) => schema,
            Ok(JsonRpcResult::Error(e)) => {
                error!("Plugin returned an error: {:?}", e);
                Default::default()
            }
            Err(e) => {
                error!("Plugin failed to respond: {:?}", e);
                Default::default()
            }
        }
    }

    pub fn set_parameters(&mut self, params: serde_json::Value) -> Result<(), Box<dyn Error>> {
        self.endpoint
            .send_notification(JsonRpcMethod::SetParameters { params })?;
        Ok(())
    }

    pub fn get_parameters(&self) -> serde_json::Value {
        match self.endpoint.send_message(JsonRpcMethod::GetParameters) {
            Ok(JsonRpcResult::Result(parameters)) => parameters,
            Ok(JsonRpcResult::Error(e)) => {
                error!("Plugin returned an error: {:?}", e);
                json!({})
            }
            Err(e) => {
                error!("Plugin failed to respond: {:?}", e);
                json!({})
            }
        }
    }

    pub fn get_fps(&self) -> f64 {
        match self.endpoint.send_message(JsonRpcMethod::GetFps) {
            Ok(JsonRpcResult::Result(fps)) => fps,
            Ok(JsonRpcResult::Error(e)) => {
                error!("Plugin returned an error: {:?}", e);
                30.0
            }
            Err(e) => {
                error!("Plugin failed to respond: {:?}", e);
                30.0
            }
        }
    }
}
