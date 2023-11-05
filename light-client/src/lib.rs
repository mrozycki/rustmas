pub mod combined;
pub mod feedback;
pub mod tty;
#[cfg(feature = "websocket")]
pub mod websocket;

#[cfg(feature = "visualiser")]
use std::{error::Error, sync::mpsc};
use std::{
    fmt,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use async_trait::async_trait;
use lightfx::{Color, Frame};
use log::{debug, info};
#[cfg(feature = "visualiser")]
use log::{error, info};
use reqwest::header::CONNECTION;
use tokio::net::{self, UdpSocket};

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
pub struct HttpLightClient {
    url: String,
    http_client: reqwest::Client,
}

impl HttpLightClient {
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
impl LightClient for HttpLightClient {
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

#[derive(Clone)]
pub struct UdpLightClient {
    // url: String,
    socket: Arc<net::UdpSocket>,
}

impl UdpLightClient {
    pub async fn new(url: &str) -> Self {
        let url = url.replace("udp://", "");
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await.unwrap());
        info!("Bound socket: {}", socket.local_addr().unwrap());
        info!("Trying to connect to: {}", url);
        let _ = socket.connect(url).await;
        // we don't want old news
        socket.set_ttl(1).unwrap();
        // hopefully this gets us some priority
        socket.set_tos(152).unwrap();
        Self { socket }
    }
}

#[async_trait]
impl LightClient for UdpLightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        let pixels: Vec<_> = frame
            .pixels_iter()
            .cloned()
            .map(gamma_correction)
            .flat_map(|pixel| vec![pixel.g, pixel.r, pixel.b])
            .collect();

        match self.socket.send(&pixels).await {
            Ok(_) => Ok(()),
            Err(err) => {
                debug!("Failed to send frame to light client: {}", err);
                Err(LightClientError::ConnectionLost)
            }
        }
    }
}

#[cfg(feature = "visualiser")]
pub struct VisualiserLightClient {
    _join_handle: std::thread::JoinHandle<()>,
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
            _join_handle: std::thread::spawn(move || {
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
