use std::{error::Error, io::Write, time::Duration};

use async_trait::async_trait;
use lightfx::Frame;
use serialport::SerialPort;
use tokio::sync::Mutex;

use crate::{LightClient, LightClientError};

pub struct TtyLightClient {
    tty: Mutex<Box<dyn SerialPort>>,
}

impl TtyLightClient {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            tty: Mutex::new(
                serialport::new(path, 921_600)
                    .timeout(Duration::from_millis(100))
                    .open()?,
            ),
        })
    }
}

fn component_gamma_correction(component: u8) -> u8 {
    (((component as f64) / 255.0).powi(2) * 255.0) as u8
}

#[async_trait]
impl LightClient for TtyLightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        let pixels: Vec<_> = frame
            .pixels_iter()
            .cloned()
            .flat_map(|pixel| vec![pixel.g, pixel.r, pixel.b])
            .map(component_gamma_correction)
            .collect();

        let mut tty = self.tty.lock().await;
        tty.write_all(&(pixels.len() as u16).to_le_bytes())
            .map_err(|_| LightClientError::ProcessExited)?;
        tty.write_all(&pixels)
            .map_err(|_| LightClientError::ProcessExited)
    }
}
