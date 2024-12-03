use crate::LightClientError;
use async_trait::async_trait;
use bytes::{Buf, Bytes};
use futures_util::TryFutureExt;
use log::{debug, error, info};
use std::{io::Cursor, sync::Arc, time::Duration};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

use super::ProtocolLightClient;

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
impl ProtocolLightClient for TcpLightClient {
    async fn display_frame(&self, pixels: Bytes) -> Result<(), LightClientError> {
        let mut stream = self.stream.lock().await;

        let res = {
            let stream = if let Some(ref mut stream) = *stream {
                stream
            } else {
                *stream = Some(self.connect().await?);
                stream.as_mut().unwrap()
            };
            let mut buf =
                Cursor::new([(pixels.len() as u16).to_le_bytes().as_ref(), &pixels].concat());
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
