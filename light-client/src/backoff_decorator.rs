use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use log::{info, warn};
use tokio::sync::Mutex;

use crate::{LightClient, LightClientError};

#[derive(Clone, Copy, Debug)]
pub struct BackoffConfig {
    pub start_delay: Duration,
    pub max_delay: Duration,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            start_delay: Duration::milliseconds(125),
            max_delay: Duration::seconds(1),
        }
    }
}

impl BackoffConfig {
    pub fn slow() -> Self {
        Self {
            start_delay: Duration::seconds(1),
            max_delay: Duration::seconds(8),
        }
    }
}

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
    state: Mutex<BackoffState>,
}

impl<T: LightClient> BackoffDecorator<T> {
    pub fn new(light_client: T, config: BackoffConfig) -> BackoffDecorator<T> {
        Self {
            inner: light_client,
            start_delay: config.start_delay,
            max_delay: config.max_delay,
            state: Mutex::new(BackoffState {
                status: ConnectionStatus::Healthy,
                delay: config.start_delay,
                next_check: Utc::now(),
            }),
        }
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
            return Err(LightClientError::ConnectionLost {
                reason: "Waiting to reconnect".into(),
            });
        }

        match self.inner.display_frame(frame).await {
            Ok(_) => {
                if state.status != ConnectionStatus::Healthy {
                    info!("Regained connection to lights");
                }
                state.status = ConnectionStatus::Healthy;
                state.delay = self.start_delay;
                state.next_check = now;
                Ok(())
            }
            Err(LightClientError::ConnectionLost { reason }) => {
                state.next_check = now + state.delay;
                if state.delay < self.max_delay {
                    state.status = ConnectionStatus::IntermittentFailure;
                    warn!(
                        "Failed to send frame to remote lights, will retry in {:.2} seconds; reason: {}",
                        state.delay.num_milliseconds() as f64 / 1000.0,
                        reason
                    );
                } else if state.status != ConnectionStatus::ProlongedFailure {
                    state.status = ConnectionStatus::ProlongedFailure;
                    warn!(
                        "Lost connection to lights, will continue retrying every {:.2} seconds; reason: {}",
                        self.max_delay.num_milliseconds() as f64 / 1000.0,
                        reason
                    );
                }
                state.delay = (state.delay * 2).min(self.max_delay);
                Err(LightClientError::ConnectionLost { reason })
            }
            Err(LightClientError::ProcessExited) => {
                warn!("Light client exited, exiting");
                Err(LightClientError::ProcessExited)
            }
            _ => Err(LightClientError::Unlikely),
        }
    }
}

pub trait WithBackoff: Sized + LightClient {
    fn with_backoff(self, config: BackoffConfig) -> BackoffDecorator<Self> {
        BackoffDecorator::new(self, config)
    }

    fn with_default_backoff(self) -> BackoffDecorator<Self> {
        Self::with_backoff(self, Default::default())
    }

    fn with_slow_backoff(self) -> BackoffDecorator<Self> {
        Self::with_backoff(self, BackoffConfig::slow())
    }
}
