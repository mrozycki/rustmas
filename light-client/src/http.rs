use crate::{backoff_decorator::BackoffDecorator, LightClient, LightClientError};
use async_trait::async_trait;
use lightfx::Frame;
use log::debug;
use reqwest::header::CONNECTION;
use std::time::Duration;

#[derive(Clone)]
pub struct HttpLightClient {
    url: String,
    http_client: reqwest::Client,
}

impl HttpLightClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            http_client: reqwest::Client::builder()
                .http1_title_case_headers()
                .tcp_keepalive(Duration::from_secs(10))
                .timeout(Duration::from_secs(1))
                .build()
                .unwrap(),
        }
    }

    pub fn with_backoff(self) -> BackoffDecorator<Self> {
        BackoffDecorator::new(self)
    }
}

#[async_trait]
impl LightClient for HttpLightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        let pixels: Vec<_> = frame
            .pixels_iter()
            .cloned()
            .map(crate::gamma_correction)
            .flat_map(|pixel| vec![pixel.g, pixel.r, pixel.b])
            .collect();

        match self
            .http_client
            .post(&self.url)
            .header(CONNECTION, "keep-alive")
            .body(pixels)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                debug!("Failed to send frame to light client: {}", err);
                Err(LightClientError::ConnectionLost {
                    reason: err.to_string(),
                })
            }
        }
    }
}
