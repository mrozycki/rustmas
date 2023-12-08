pub mod combined;
pub mod feedback;
pub mod http;
pub mod tcp;
pub mod tty;
pub mod udp;

use std::{
    fmt,
    sync::{Mutex, MutexGuard},
};

use async_trait::async_trait;
use lightfx::{Color, Frame};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LightClientError {
    Unlikely, // inspired by https://github.com/bluez/bluez/blob/58e6ef54e672798e2621cae87356c66de14d011f/attrib/att.h#L61
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
