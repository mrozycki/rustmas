use std::{
    error::Error,
    ffi::OsStr,
    io::{BufRead, BufReader, BufWriter, Write},
    process::{Child, Command, Stdio},
    sync::Mutex,
};

use animation_api::{
    AnimationError, AnimationParameters, JsonRpcMessage, JsonRpcMethod, JsonRpcResponse,
    JsonRpcResult, StepAnimation,
};
use serde::de::DeserializeOwned;
use serde_json::json;

pub struct JsonRpcEndpoint {
    child_process: Mutex<Child>,
    id: Mutex<usize>,
}

impl JsonRpcEndpoint {
    pub fn new<P: AsRef<OsStr>>(executable_path: P) -> Result<Self, Box<dyn Error>> {
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
        writer.write(b"\n")?;
        Ok(())
    }

    fn receive<Res>(
        reader: &mut impl std::io::Read,
    ) -> Result<JsonRpcResponse<Res, AnimationError>, Box<dyn Error>>
    where
        Res: DeserializeOwned,
    {
        let mut reader = BufReader::new(reader);
        let mut buffer = String::new();
        reader.read_line(&mut buffer)?;
        Ok(serde_json::from_str(&buffer)?)
    }

    pub fn send_message<Res>(
        &self,
        payload: JsonRpcMethod,
    ) -> Result<JsonRpcResponse<Res, AnimationError>, Box<dyn Error>>
    where
        Res: DeserializeOwned,
    {
        let mut lock = self.child_process.lock().unwrap();
        let mut id = self.id.lock().unwrap();
        *id += 1;
        Self::send(
            lock.stdin.as_mut().unwrap(),
            JsonRpcMessage {
                id: Some(*id),
                payload,
            },
        )?;
        Self::receive(lock.stdout.as_mut().unwrap())
    }

    pub fn send_notification(&self, payload: JsonRpcMethod) -> Result<(), Box<dyn Error>> {
        let mut lock = self.child_process.lock().unwrap();
        Self::send(
            lock.stdin.as_mut().unwrap(),
            JsonRpcMessage { id: None, payload },
        )?;

        Ok(())
    }
}

pub struct AnimationPlugin {
    endpoint: JsonRpcEndpoint,
}

impl AnimationPlugin {
    pub fn new(endpoint: JsonRpcEndpoint, points: Vec<(f64, f64, f64)>) -> Box<dyn StepAnimation> {
        let _ = endpoint.send_notification(JsonRpcMethod::Initialize { points });
        Box::new(Self { endpoint })
    }
}

impl StepAnimation for AnimationPlugin {
    fn update(&mut self, time_delta: f64) {
        let _ = self
            .endpoint
            .send_notification(JsonRpcMethod::Update { time_delta });
    }

    fn render(&self) -> lightfx::Frame {
        if let JsonRpcResult::Result(frame) = self
            .endpoint
            .send_message(JsonRpcMethod::Render)
            .unwrap()
            .result
        {
            frame
        } else {
            lightfx::Frame::new_black(0)
        }
    }
}

impl AnimationParameters for AnimationPlugin {
    fn animation_name(&self) -> &str {
        "animation plugin"
    }

    fn parameter_schema(&self) -> lightfx::parameter_schema::ParametersSchema {
        if let JsonRpcResult::Result(schema) = self
            .endpoint
            .send_message(JsonRpcMethod::ParameterSchema)
            .unwrap()
            .result
        {
            schema
        } else {
            Default::default()
        }
    }

    fn set_parameters(&mut self, params: serde_json::Value) -> Result<(), Box<dyn Error>> {
        self.endpoint
            .send_notification(JsonRpcMethod::SetParameters { params })
    }

    fn get_parameters(&self) -> serde_json::Value {
        if let JsonRpcResult::Result(parameters) = self
            .endpoint
            .send_message(JsonRpcMethod::GetParameters)
            .unwrap()
            .result
        {
            parameters
        } else {
            json!({})
        }
    }

    fn get_fps(&self) -> f64 {
        if let JsonRpcResult::Result(fps) = self
            .endpoint
            .send_message(JsonRpcMethod::GetFps)
            .unwrap()
            .result
        {
            fps
        } else {
            30.0
        }
    }
}
