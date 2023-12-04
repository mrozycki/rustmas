use crate::{LightClient, LightClientError};
use async_trait::async_trait;
use lightfx::Frame;
use log::{debug, error, info};
use std::{error::Error, sync::Arc};
use tokio::{net::UdpSocket, sync::Mutex};

#[derive(Clone)]
pub struct UdpLightClient {
    url: String,
    socket: Arc<Mutex<Option<UdpSocket>>>,
}

impl UdpLightClient {
    pub fn new(url: &str) -> Self {
        let url = url.strip_prefix("udp://").unwrap_or(url);
        Self {
            url: url.to_owned(),
            socket: Arc::new(Mutex::new(None)),
        }
    }

    async fn connect(&self) -> Result<(), Box<dyn Error>> {
        let mut socket = self.socket.lock().await;
        if socket.is_none() {
            info!("Connecting to remote lights via UDP");
            let s = UdpSocket::bind("0.0.0.0:0").await?;
            s.connect(&self.url).await?;
            // hopefully this gets us some priority, see:
            // https://linuxreviews.org/Type_of_Service_(ToS)_and_DSCP_Values
            // this sets `high throughput` and `low delay` along with high precedence
            s.set_tos(152)?;
            *socket = Some(s);
        }

        Ok(())
    }
}

#[async_trait]
impl LightClient for UdpLightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        let pixels: Vec<_> = frame
            .pixels_iter()
            .cloned()
            .map(crate::gamma_correction)
            .flat_map(|pixel| vec![pixel.g, pixel.r, pixel.b])
            .collect();

        if self.socket.lock().await.is_none() {
            if let Err(e) = self.connect().await {
                error!("Failed to connect to UDP endpoint: {}", e);
            } else {
                info!("Successfully connected to UDP endpoint");
            }
        }

        let res = {
            let mut stream = self.socket.lock().await;
            let Some(stream) = stream.as_mut() else {
                debug!("UDP endpoint not connected!");
                return Err(LightClientError::ConnectionLost);
            };
            stream
                .send(&[&(pixels.len() as u16).to_le_bytes(), pixels.as_slice()].concat())
                .await
        };

        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to send frame to light client: {}", e);
                *self.socket.lock().await = None;
                Err(LightClientError::ConnectionLost)
            }
        }
    }
}
