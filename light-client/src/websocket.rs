use async_trait::async_trait;
use bevy_websocket_adapter::{client::Client, impl_message_type};
use serde::{Deserialize, Serialize};

use crate::{LightClient, LightClientError};

pub struct WebsocketLightClient {
    client: Client,
}

impl WebsocketLightClient {
    pub fn new(url: String) -> Self {
        let mut client = Client::new();
        client.connect(url);
        Self { client }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename = "frame", tag = "type")]
struct FrameEvent {
    pixels: Vec<lightfx::Color>,
}
impl_message_type!(FrameEvent, "frame");

#[async_trait]
impl LightClient for WebsocketLightClient {
    async fn display_frame(&self, frame: &lightfx::Frame) -> Result<(), LightClientError> {
        if !self.client.is_running() {
            return Err(LightClientError::ProcessExited);
        }

        self.client.send_message(&FrameEvent {
            pixels: frame.pixels_iter().cloned().collect(),
        });
        Ok(())
    }
}
