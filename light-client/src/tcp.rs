use crate::{backoff_decorator::BackoffDecorator, LightClient, LightClientError};
use async_trait::async_trait;
use bytes::Buf;
use futures_util::TryFutureExt;
use lightfx::Frame;
use std::{io::Cursor, sync::Arc, time::Duration};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct TcpLightClient {
    url: String,
    stream: Arc<Mutex<Option<TcpStream>>>,
    timeout: Duration,
}

impl TcpLightClient {
    pub fn new(url: &str) -> Self {
        let url = url.strip_prefix("tcp://").unwrap_or(url);
        Self {
            url: url.to_owned(),
            stream: Arc::new(Mutex::new(None)),
            timeout: Duration::from_secs(1),
        }
    }

    pub fn with_backoff(self) -> BackoffDecorator<Self> {
        BackoffDecorator::new(self)
    }

    async fn connect(&self) -> Result<TcpStream, LightClientError> {
        debug!("Connecting to remote lights via TCP");
        let connect = TcpStream::connect(&self.url).and_then(|s| async {
            s.set_nodelay(true)?;
            Ok(s)
        });
        match tokio::time::timeout(self.timeout, connect).await {
            Ok(Ok(stream)) => {
                info!("Successfully connected to TCP lights");
                Ok(stream)
            }
            Ok(Err(e)) => Err(LightClientError::ConnectionLost {
                reason: e.to_string(),
            }),
            Err(_) => Err(LightClientError::ConnectionLost {
                reason: "Timeout".into(),
            }),
        }
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

        let mut stream = self.stream.lock().await;

        let res = {
            let stream = if let Some(ref mut stream) = *stream {
                stream
            } else {
                *stream = Some(self.connect().await?);
                stream.as_mut().unwrap()
            };
            let mut buf =
                Cursor::new([&(pixels.len() as u16).to_le_bytes(), pixels.as_slice()].concat());
            let result = tokio::time::timeout(self.timeout, stream.write_all_buf(&mut buf)).await;
            if buf.remaining() != 0 && buf.remaining() != buf.get_ref().len() {
                error!(
                    "Write failed, {} out of {} bytes were not written",
                    buf.remaining(),
                    buf.get_ref().len()
                );
            }
            result
        };

        match res {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => {
                *stream = None;
                Err(LightClientError::ConnectionLost {
                    reason: e.to_string(),
                })
            }
            Err(_) => {
                *stream = None;
                Err(LightClientError::ConnectionLost {
                    reason: "Timeout".into(),
                })
            }
        }
    }
}
