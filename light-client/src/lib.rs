pub mod combined;
pub mod feedback;
pub mod tty;
#[cfg(feature = "websocket")]
pub mod websocket;

use std::{
    fmt,
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use async_trait::async_trait;
use lightfx::{Color, Frame};
use log::debug;
use reqwest::header::CONNECTION;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LightClientError {
    Unlikely,
    ConnectionLost,
    ProcessExited,
}

impl fmt::Display for LightClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for LightClientError {}

#[async_trait]
pub trait LightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError>;
}

#[derive(Default)]
pub struct MockLightClient {
    frames: Mutex<Vec<Frame>>,
}

impl MockLightClient {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_frames(&self) -> MutexGuard<Vec<Frame>> {
        self.frames.lock().unwrap()
    }
}

#[async_trait]
impl LightClient for MockLightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        self.frames.lock().unwrap().push(frame.clone());
        Ok(())
    }
}

#[derive(Clone)]
pub struct RemoteLightClient {
    url: String,
    http_client: reqwest::Client,
}

impl RemoteLightClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            http_client: reqwest::Client::builder()
                .http1_title_case_headers()
                .tcp_keepalive(Duration::from_secs(10))
                .timeout(Duration::from_secs(1))
                .build()
                .unwrap(),
        }
    }
}

fn component_gamma_correction(component: u8) -> u8 {
    (((component as f64) / 255.0).powi(2) * 255.0) as u8
}

fn gamma_correction(color: Color) -> Color {
    Color {
        r: component_gamma_correction(color.r),
        g: component_gamma_correction(color.g),
        b: component_gamma_correction(color.b),
    }
}

#[async_trait]
impl LightClient for RemoteLightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        let pixels: Vec<_> = frame
            .pixels_iter()
            .cloned()
            .map(gamma_correction)
            .flat_map(|pixel| vec![pixel.g, pixel.r, pixel.b])
            .collect();

        match self
            .http_client
            .post(&self.url)
            .header(CONNECTION, "keep-alive")
            .body(pixels)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                debug!("Failed to send frame to light client: {}", err);
                Err(LightClientError::ConnectionLost)
            }
        }
    }
}
