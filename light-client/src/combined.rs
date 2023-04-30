use async_trait::async_trait;
use futures_util::future::join_all;

use crate::{LightClient, LightClientError};

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
        let futures: Vec<_> = self
            .clients
            .iter()
            .map(|client| client.display_frame(frame))
            .collect();
        let errors: Vec<_> = join_all(futures)
            .await
            .into_iter()
            .flat_map(|r| r.err())
            .collect();

        if errors.len() == self.clients.len() {
            if errors.iter().all(|e| *e == LightClientError::ProcessExited) {
                Err(LightClientError::ProcessExited)
            } else {
                Err(LightClientError::ConnectionLost)
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

    pub fn with(mut self, client: Box<dyn LightClient + Send + Sync>) -> Self {
        self.clients.push(client);
        self
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
