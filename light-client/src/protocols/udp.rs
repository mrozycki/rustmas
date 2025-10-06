use crate::LightClientError;
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::TryFutureExt;
use log::{debug, info};
use std::sync::Arc;
use tokio::{net::UdpSocket, sync::Mutex};

use super::ProtocolLightClient;

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

    async fn connect(&self) -> Result<UdpSocket, LightClientError> {
        debug!("Connecting to remote lights via UDP");
        let connect = UdpSocket::bind("0.0.0.0:1234").and_then(|s| async {
            s.connect(&self.url).await?;
            // hopefully this gets us some priority, see:
            // https://linuxreviews.org/Type_of_Service_(ToS)_and_DSCP_Values
            // this sets `high throughput` and `low delay` along with high precedence
            s.set_tos(152)?;
            Ok(s)
        });

        match connect.await {
            Ok(socket) => {
                info!("Successfully connected to UDP lights");
                Ok(socket)
            }
            Err(e) => Err(LightClientError::ConnectionLost {
                reason: e.to_string(),
            }),
        }
    }
}

#[async_trait]
impl ProtocolLightClient for UdpLightClient {
    async fn display_frame(&self, pixels: Bytes) -> Result<(), LightClientError> {
        let mut socket = self.socket.lock().await;

        let res = {
            let socket = if let Some(ref mut socket) = *socket {
                socket
            } else {
                *socket = Some(self.connect().await?);
                socket.as_mut().unwrap()
            };
            socket
                .send(&[(pixels.len() as u16).to_le_bytes().as_ref(), &pixels].concat())
                .await
        };

        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                *socket = None;
                Err(LightClientError::ConnectionLost {
                    reason: e.to_string(),
                })
            }
        }
    }
}
