use crate::{backoff_decorator::BackoffDecorator, LightClient, LightClientError};
use async_trait::async_trait;
use lightfx::Frame;
use log::{debug, error, info};
use std::{error::Error, sync::Arc};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

#[derive(Clone)]
pub struct TcpLightClient {
    url: String,
    stream: Arc<Mutex<Option<TcpStream>>>,
}

impl TcpLightClient {
    pub fn new(url: &str) -> Self {
        let url = url.strip_prefix("tcp://").unwrap_or(url);
        Self {
            url: url.to_owned(),
            stream: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_backoff(self) -> BackoffDecorator<Self> {
        BackoffDecorator::new(self)
    }

    async fn connect(&self) -> Result<(), Box<dyn Error>> {
        debug!("Connecting to remote lights via TCP");
        let mut stream = self.stream.lock().await;
        let s = TcpStream::connect(&self.url).await?;
        s.set_nodelay(true)?;
        *stream = Some(s);
        Ok(())
    }
}

#[async_trait]
impl LightClient for TcpLightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        let pixels: Vec<_> = frame
            .pixels_iter()
            .cloned()
            .map(crate::gamma_correction)
            .flat_map(|pixel| vec![pixel.g, pixel.r, pixel.b])
            .collect();

        if self.stream.lock().await.is_none() {
            if let Err(e) = self.connect().await {
                error!("Failed to connect to TCP endpoint: {}", e);
            } else {
                info!("Successfully connected to TCP endpoint");
            }
        }

        let res = {
            let mut stream = self.stream.lock().await;
            let Some(stream) = stream.as_mut() else {
                debug!("TCP stream not connected!");
                return Err(LightClientError::ConnectionLost);
            };
            stream
                .write_all(&[&(pixels.len() as u16).to_le_bytes(), pixels.as_slice()].concat())
                .await
        };

        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to send frame to light client: {}", e);
                *self.stream.lock().await = None;
                Err(LightClientError::ConnectionLost)
            }
        }
    }
}
