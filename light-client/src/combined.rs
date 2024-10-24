use std::error::Error;

use async_trait::async_trait;
use chrono::Duration;
use futures_util::future::join_all;
use itertools::Itertools;
use tracing::{error, info};
use url::Url;

use crate::{
    config::TtyLightsConfig, http::HttpLightClient, tcp::TcpLightClient, tty::TtyLightClient,
    udp::UdpLightClient, LightClient, LightClientError, LightsConfig,
};

pub struct CombinedLightClient {
    clients: Vec<Box<dyn LightClient + Send + Sync>>,
}

impl CombinedLightClient {
    pub fn builder() -> CombinedLightClientBuilder {
        CombinedLightClientBuilder::new()
    }
}

#[async_trait]
impl LightClient for CombinedLightClient {
    async fn display_frame(&self, frame: &lightfx::Frame) -> Result<(), LightClientError> {
        let futures = self
            .clients
            .iter()
            .map(|client| client.display_frame(frame))
            .collect_vec();
        let errors = join_all(futures)
            .await
            .into_iter()
            .flat_map(|r| r.err())
            .collect_vec();

        if errors.len() == self.clients.len() {
            if errors.iter().all(|e| *e == LightClientError::ProcessExited) {
                Err(LightClientError::ProcessExited)
            } else {
                Err(LightClientError::ConnectionLost {
                    reason: errors.iter().join("; "),
                })
            }
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
pub struct CombinedLightClientBuilder {
    clients: Vec<Box<dyn LightClient + Send + Sync>>,
}

impl CombinedLightClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, client: impl LightClient + Send + Sync + 'static) -> Self {
        self.clients.push(Box::new(client));
        self
    }

    pub fn with_config(mut self, configs: Vec<LightsConfig>) -> Result<Self, Box<dyn Error>> {
        for config in configs.into_iter() {
            match config {
                LightsConfig::Remote(url) => match url.scheme() {
                    "http" => self = self.http_lights(url),
                    "tcp" => self = self.tcp_lights(url),
                    "udp" => self = self.udp_lights(url),
                    scheme => {
                        error!("Unknown remote client protocol, ignoring");
                        Err(format!("Unknown remote client protocol: {scheme}"))?
                    }
                },
                LightsConfig::Tty(TtyLightsConfig::Detect) => self = self.local_lights()?,
                LightsConfig::Tty(TtyLightsConfig::Path(_path)) => unimplemented!(),
            }
        }
        Ok(self)
    }

    pub fn http_lights(self, url: Url) -> Self {
        info!("Using http light client with endpoint {}", url);
        self.with(HttpLightClient::new(url.as_ref()).with_backoff())
    }

    pub fn tcp_lights(self, url: Url) -> Self {
        info!("Using tcp light client with endpoint {}", url);
        self.with(
            TcpLightClient::new(url.as_ref())
                .with_backoff()
                .with_start_delay(Duration::milliseconds(125)),
        )
    }

    pub fn udp_lights(self, url: Url) -> Self {
        info!("Using udp light client with endpoint {}", url);
        self.with(UdpLightClient::new(url.as_ref()))
    }

    pub fn local_lights(self) -> Result<Self, Box<dyn Error>> {
        info!("Using tty lights client");
        Ok(self.with(TtyLightClient::new()?))
    }

    pub fn build(mut self) -> Box<dyn LightClient + Send + Sync> {
        if self.clients.len() == 1 {
            self.clients.remove(0)
        } else {
            Box::new(CombinedLightClient {
                clients: self.clients,
            })
        }
    }
}
