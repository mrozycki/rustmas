use async_trait::async_trait;
use log::error;
use tokio::sync::mpsc;

use crate::{LightClient, LightClientError};

pub struct FeedbackLightClient {
    sender: mpsc::Sender<lightfx::Frame>,
}

impl FeedbackLightClient {
    pub fn new(sender: mpsc::Sender<lightfx::Frame>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl LightClient for FeedbackLightClient {
    async fn display_frame(&self, frame: &lightfx::Frame) -> Result<(), LightClientError> {
        self.sender.send(frame.clone()).await.map_err(|e| {
            error!("Error sending frame: {}", e);
            LightClientError::ConnectionLost
        })?;
        Ok(())
    }
}
