mod animations;

use std::error::Error;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use log::{info, warn};
use rustmas_light_client as client;
use rustmas_light_client::LightClientError;
use serde_json::json;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use animations::Animation;

#[derive(PartialEq)]
enum ConnectionStatus {
    Healthy,
    IntermittentFailure,
    ProlongedFailure,
}

pub struct ControllerState {
    points: Vec<(f64, f64, f64)>,
    animation: Box<dyn Animation + Sync + Send>,
    next_frame: DateTime<Utc>,
    fps: f64,
}

pub struct Controller {
    join_handle: JoinHandle<()>,
    state: Arc<Mutex<ControllerState>>,
}

impl Controller {
    pub fn new(
        points: Vec<(f64, f64, f64)>,
        client: Box<dyn rustmas_light_client::LightClient + Sync + Send>,
    ) -> Result<Self, Box<dyn Error>> {
        let animation = animations::make_animation("blank", &points);
        let state = Arc::new(Mutex::new(ControllerState {
            points,
            next_frame: Utc::now(),
            fps: animation.get_fps(),
            animation,
        }));
        let join_handle = tokio::spawn(Self::run(state.clone(), client));

        Ok(Self { state, join_handle })
    }

    async fn run(
        state: Arc<Mutex<ControllerState>>,
        client: Box<dyn rustmas_light_client::LightClient + Sync + Send>,
    ) {
        let start_backoff_delay: Duration = Duration::seconds(1);
        let max_backoff_delay: Duration = Duration::seconds(8);

        let mut backoff_delay = start_backoff_delay;
        let mut status = ConnectionStatus::Healthy;
        let now = Utc::now();
        let first_frame = now;
        let mut next_check = now;

        loop {
            tokio::time::sleep(
                (next_check - Utc::now())
                    .max(Duration::seconds(0))
                    .to_std()
                    .unwrap(),
            )
            .await;

            let now = Utc::now();
            let frame = {
                let mut state = state.lock().await;
                if now < state.next_frame {
                    next_check = state.next_frame.min(now + backoff_delay);
                    continue;
                }
                state.next_frame = if state.fps != 0.0 {
                    now + Duration::milliseconds((1000.0 / state.fps) as i64)
                } else {
                    now + Duration::days(1)
                };

                state
                    .animation
                    .frame((now - first_frame).num_milliseconds() as f64 / 1000.0)
            };

            match client.display_frame(&frame).await {
                Ok(_) => {
                    if status != ConnectionStatus::Healthy {
                        info!("Regained connection to light client");
                    }
                    status = ConnectionStatus::Healthy;
                    backoff_delay = start_backoff_delay;
                }
                Err(LightClientError::ConnectionLost) => {
                    next_check = now + backoff_delay;
                    backoff_delay = (backoff_delay * 2).min(max_backoff_delay);
                    if backoff_delay < max_backoff_delay {
                        status = ConnectionStatus::IntermittentFailure;
                        warn!(
                            "Failed to send frame to remote lights, will retry in {:.2} seconds",
                            backoff_delay.num_milliseconds() as f64 / 1000.0
                        );
                    } else if status != ConnectionStatus::ProlongedFailure {
                        status = ConnectionStatus::ProlongedFailure;
                        warn!(
                            "Lost connection to lights, will continue retrying every {:.2} seconds",
                            max_backoff_delay.num_milliseconds() as f64 / 1000.0
                        );
                    }
                }
                Err(LightClientError::ProcessExited) => {
                    warn!("Light client exited, exiting");
                    return;
                }
                _ => (),
            };
        }
    }

    pub fn builder() -> ControllerBuilder {
        ControllerBuilder {
            points: None,
            client: None,
        }
    }

    pub async fn switch_animation(&self, name: &str) -> Result<(), Box<dyn Error>> {
        info!("Trying to switch animation to \"{}\"", name);
        let mut state = self.state.lock().await;
        state.animation = animations::make_animation(name, &state.points);

        let new_fps = state.animation.get_fps();
        if new_fps != state.fps {
            state.fps = new_fps;
            state.next_frame = Utc::now();
        }
        Ok(())
    }

    pub async fn parameters(&self) -> serde_json::Value {
        let animation = &self.state.lock().await.animation;
        json!({"schema": animation.parameter_schema(), "values": animation.get_parameters()})
    }

    pub async fn parameter_values(&self) -> serde_json::Value {
        self.state.lock().await.animation.get_parameters()
    }

    pub async fn set_parameters(
        &mut self,
        parameters: serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        self.state.lock().await.animation.set_parameters(parameters)
    }

    pub async fn join(self) -> Result<(), Box<dyn Error>> {
        self.join_handle.await?;

        Ok(())
    }
}

pub struct ControllerBuilder {
    points: Option<Vec<(f64, f64, f64)>>,
    client: Option<Box<dyn rustmas_light_client::LightClient + Sync + Send>>,
}

impl ControllerBuilder {
    pub fn points_from_file(mut self, path: &str) -> Result<Self, Box<dyn Error>> {
        let points: Vec<_> = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)?
            .deserialize()
            .filter_map(|record: Result<(f64, f64, f64), _>| record.ok())
            .collect();
        info!("Loaded {} points from {}", points.len(), path);

        self.points = Some(points);
        Ok(self)
    }

    pub fn remote_lights(mut self, path: &str) -> Result<Self, Box<dyn Error>> {
        info!("Using remote light client with endpoint {}", path);
        self.client = Some(Box::new(client::RemoteLightClient::new(&path)));
        Ok(self)
    }

    #[cfg(feature = "visualiser")]
    pub fn visualiser_lights(mut self) -> Result<Self, Box<dyn Error>> {
        info!("Using local visualiser");
        self.client = Some(Box::new(client::VisualiserLightClient::new(
            self.points.as_ref().unwrap().clone(),
        )?));
        Ok(self)
    }

    pub fn build(self) -> Result<Controller, Box<dyn Error>> {
        Controller::new(self.points.unwrap(), self.client.unwrap())
    }
}
