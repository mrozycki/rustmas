use core::fmt;
use std::{
    error::Error,
    sync::{mpsc, Mutex, MutexGuard},
    time::Duration,
};

use async_trait::async_trait;

#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn gray(shade: u8) -> Self {
        Self::rgb(shade, shade, shade)
    }

    pub fn black() -> Self {
        Self::gray(0)
    }

    pub fn white() -> Self {
        Self::gray(255)
    }
}

#[derive(Clone)]
pub struct Frame {
    pixels: Vec<Color>,
}

impl Frame {
    pub fn new(light_count: usize, color: Color) -> Self {
        Self {
            pixels: vec![color; light_count],
        }
    }
    pub fn new_black(light_count: usize) -> Self {
        Self::new(light_count, Color::black())
    }

    pub fn set_pixel(&mut self, index: usize, color: Color) {
        self.pixels[index] = color;
    }

    pub fn with_pixel(mut self, index: usize, color: Color) -> Self {
        self.pixels[index] = color;
        self
    }

    pub fn pixels_iter(&self) -> impl Iterator<Item = &Color> {
        self.pixels.iter()
    }
}

impl From<Vec<Color>> for Frame {
    fn from(pixels: Vec<Color>) -> Self {
        Self { pixels }
    }
}

#[derive(Debug)]
pub enum LightClientError {
    Unlikely,
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
        let pixels = frame
            .pixels_iter()
            .flat_map(|pixel| vec![&pixel.r, &pixel.g, &pixel.b]);
        let request: Vec<_> = "data=".as_bytes().iter().chain(pixels).cloned().collect();

        match self.http_client.post(&self.url).body(request).send().await {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!("{:?}", err);
                Err(LightClientError::Unlikely)
            }
        }
    }
}

pub struct VisualiserLightClient {
    _join_handle: tokio::task::JoinHandle<()>,
    light_tx: Mutex<mpsc::Sender<Vec<(f32, f32, f32)>>>,
}

impl VisualiserLightClient {
    pub fn new(input_path: &str) -> Result<Self, Box<dyn Error>> {
        let mut file = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(input_path)?;
        let points: Vec<_> = file
            .deserialize()
            .filter_map(|record: Result<(f32, f32, f32), _>| record.ok())
            .collect();

        let (tx, rx) = mpsc::channel();
        Ok(Self {
            _join_handle: tokio::spawn(async move {
                rustmas_visualiser::visualise(points, rx).unwrap();
            }),
            light_tx: Mutex::new(tx),
        })
    }
}

#[async_trait]
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
            .map_err(|_| LightClientError::Unlikely)
    }
}
