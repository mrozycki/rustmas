use std::{error::Error, io::Write, time::Duration};

use async_trait::async_trait;
use lightfx::Frame;
use log::info;
use serialport::{SerialPort, SerialPortType};
use tokio::sync::Mutex;

use crate::{LightClient, LightClientError};

pub struct TtyLightClient {
    tty: Mutex<Box<dyn SerialPort>>,
}

impl TtyLightClient {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let port_name = get_port()?;
        Ok(Self {
            tty: Mutex::new(
                serialport::new(port_name, 921_600)
                    .timeout(Duration::from_millis(100))
                    .open()?,
            ),
        })
    }
}

fn get_port() -> Result<String, Box<dyn Error>> {
    for port in serialport::available_ports()? {
        if let SerialPortType::UsbPort(port_info) = port.port_type {
            if let Some(manufacturer) = port_info.manufacturer {
                if manufacturer == "krzmaz" {
                    info!("Found endpoint: {}", port.port_name);
                    return Ok(port.port_name);
                }
            }
        }
    }
    Err(Box::new(LightClientError::ConnectionLost {
        reason: "TTY endpoint not found".into(),
    }))
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
        let bytes_written = tty.write_all(&(pixels.len() as u16).to_le_bytes());
        if bytes_written.is_ok() {
            tty.write_all(&pixels)
                .map_err(|_| LightClientError::ProcessExited)
        } else {
            let port_name = get_port().map_err(|_| LightClientError::ProcessExited)?;

            *tty = serialport::new(port_name, 921_600)
                .timeout(Duration::from_millis(100))
                .open()
                .map_err(|_| LightClientError::ProcessExited)?;
            Ok(())
        }
    }
}
