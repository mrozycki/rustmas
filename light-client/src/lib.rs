use core::fmt;
#[cfg(feature = "visualiser")]
use std::{error::Error, sync::mpsc};
use std::{
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use async_trait::async_trait;
use lightfx::{Color, Frame};
use log::error;
#[cfg(feature = "visualiser")]
use log::info;
use reqwest::header::CONNECTION;

#[derive(Debug)]
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

pub struct MockLightClient {
    frames: Mutex<Vec<Frame>>,
}

impl MockLightClient {
    pub fn new() -> Self {
        Self {
            frames: Mutex::new(Vec::new()),
        }
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
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl LightClient for RemoteLightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        let pixels: Vec<_> = frame
            .pixels_iter()
            .flat_map(|pixel| vec![&pixel.g, &pixel.r, &pixel.b])
            .cloned()
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
                error!("Failed to send a frame to remote lights client: {}", err);
                Err(LightClientError::ConnectionLost)
            }
        }
    }
}

#[cfg(feature = "visualiser")]
pub struct VisualiserLightClient {
    _join_handle: tokio::task::JoinHandle<()>,
    light_tx: Mutex<mpsc::Sender<Vec<(f32, f32, f32)>>>,
}

#[cfg(feature = "visualiser")]
impl VisualiserLightClient {
    pub fn new(points: Vec<(f64, f64, f64)>) -> Result<Self, Box<dyn Error>> {
        let points = points
            .into_iter()
            .map(|(x, y, z)| (x as f32, y as f32, z as f32))
            .collect();

        let (tx, rx) = mpsc::channel();
        Ok(Self {
            _join_handle: tokio::spawn(async move {
                match rustmas_visualiser::visualise(points, rx) {
                    Ok(_) => info!("Visualiser completed without errors"),
                    Err(e) => error!("Visualiser returned an error: {}", e),
                }
            }),
            light_tx: Mutex::new(tx),
        })
    }
}

#[async_trait]
#[cfg(feature = "visualiser")]
impl LightClient for VisualiserLightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        let pixels = frame
            .pixels_iter()
            .map(|Color { r, g, b }| (*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0))
            .collect();

        self.light_tx
            .lock()
            .map_err(|_| LightClientError::Unlikely)?
            .send(pixels)
            .map_err(|_| LightClientError::ProcessExited)
    }
}
