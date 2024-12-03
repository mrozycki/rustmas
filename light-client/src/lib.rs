pub mod backoff_decorator;
pub mod combined;
mod config;
pub mod feedback;
mod protocols;

pub use config::{ByteOrder, LightsConfig, LightsEndpoint, TtyLightsConfig};

use async_trait::async_trait;
use lightfx::Frame;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LightClientError {
    #[error("unlikely")]
    Unlikely, // inspired by https://github.com/bluez/bluez/blob/58e6ef54e672798e2621cae87356c66de14d011f/attrib/att.h#L61

    #[error("connection to the light client lost: {reason}")]
    ConnectionLost { reason: String },

    #[error("light client process exited")]
    ProcessExited,
}

#[async_trait]
pub trait LightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError>;
}
