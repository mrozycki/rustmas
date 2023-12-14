use std::time::Duration as StdDuration;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use log::{info, warn};
use tokio::sync::Mutex;

use crate::{LightClient, LightClientError};

#[derive(PartialEq)]
enum ConnectionStatus {
    Healthy,
    IntermittentFailure,
    ProlongedFailure,
}

struct BackoffState {
    status: ConnectionStatus,
    delay: Duration,
    next_check: DateTime<Utc>,
}

pub struct BackoffDecorator<T: LightClient> {
    inner: T,
    start_delay: Duration,
    max_delay: Duration,
    timeout: StdDuration,
    state: Mutex<BackoffState>,
}

impl<T: LightClient> BackoffDecorator<T> {
    pub fn new(light_client: T) -> BackoffDecorator<T> {
        let default_start_delay = Duration::seconds(1);
        Self {
            inner: light_client,
            start_delay: default_start_delay,
            max_delay: Duration::seconds(8),
            timeout: StdDuration::from_millis(100),
            state: Mutex::new(BackoffState {
                status: ConnectionStatus::Healthy,
                delay: default_start_delay,
                next_check: Utc::now(),
            }),
        }
    }

    pub fn with_start_delay(mut self, delay: Duration) -> Self {
        self.start_delay = delay;
        self
    }

    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    pub fn with_timeout(mut self, timeout: StdDuration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[async_trait]
impl<T> LightClient for BackoffDecorator<T>
where
    T: LightClient + Send + Sync,
{
    async fn display_frame(&self, frame: &lightfx::Frame) -> Result<(), LightClientError> {
        let mut state = self.state.lock().await;
        let now = Utc::now();
        if now < state.next_check && state.status != ConnectionStatus::Healthy {
            return Err(LightClientError::ConnectionLost);
        }

        match tokio::time::timeout(self.timeout, self.inner.display_frame(frame)).await {
            Ok(Ok(_)) => {
                if state.status != ConnectionStatus::Healthy {
                    info!("Regained connection to light client");
                }
                state.status = ConnectionStatus::Healthy;
                state.delay = self.start_delay;
                state.next_check = now;
                Ok(())
            }
            Ok(Err(LightClientError::ConnectionLost)) | Err(_) => {
                state.next_check = now + state.delay;
                if state.delay < self.max_delay {
                    state.status = ConnectionStatus::IntermittentFailure;
                    warn!(
                        "Failed to send frame to remote lights, will retry in {:.2} seconds",
                        state.delay.num_milliseconds() as f64 / 1000.0
                    );
                } else if state.status != ConnectionStatus::ProlongedFailure {
                    state.status = ConnectionStatus::ProlongedFailure;
                    warn!(
                        "Lost connection to lights, will continue retrying every {:.2} seconds",
                        self.max_delay.num_milliseconds() as f64 / 1000.0
                    );
                }
                state.delay = (state.delay * 2).min(self.max_delay);
                Err(LightClientError::ConnectionLost)
            }
            Ok(Err(LightClientError::ProcessExited)) => {
                warn!("Light client exited, exiting");
                Err(LightClientError::ProcessExited)
            }
            _ => Err(LightClientError::Unlikely),
        }
    }
}
