mod factory;
mod jsonrpc_animation;

use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use animation_api::event::Event;
use chrono::{DateTime, Duration, Utc};
use client::combined::{CombinedLightClient, CombinedLightClientBuilder};
use events::beat_generator::BeatEventGenerator;
use events::fft_generator::FftEventGenerator;
use events::midi_generator::MidiEventGenerator;
use factory::{AnimationFactory, AnimationFactoryError, Plugin};
use jsonrpc_animation::{AnimationPlugin, AnimationPluginError};
use log::{info, warn};
use rustmas_light_client as client;
use rustmas_light_client::LightClientError;
use serde_json::json;
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;

#[derive(Debug, Error)]
pub enum ControllerError {
    #[error("animation controller error: {reason}")]
    InternalError { reason: String },

    #[error(transparent)]
    AnimationPluginError(#[from] AnimationPluginError),

    #[error("animation factory error: {0}")]
    AnimationFactoryError(#[from] AnimationFactoryError),
}

#[derive(PartialEq)]
enum ConnectionStatus {
    Healthy,
    IntermittentFailure,
    ProlongedFailure,
}

pub struct ControllerState {
    animation: Option<AnimationPlugin>,
    last_frame: DateTime<Utc>,
    next_frame: DateTime<Utc>,
    fps: f64,
    event_generators: Vec<Box<dyn Send + Sync>>,
}

pub struct Controller {
    animation_join_handle: JoinHandle<()>,
    event_generator_join_handle: JoinHandle<()>,
    state: Arc<Mutex<ControllerState>>,
    animation_factory: AnimationFactory,
    event_sender: Sender<Event>,
}

impl Controller {
    pub fn new<P: AsRef<Path>>(
        points: Vec<(f64, f64, f64)>,
        plugin_dir: P,
        client: Box<dyn rustmas_light_client::LightClient + Sync + Send>,
    ) -> Self {
        let now = Utc::now();
        let (event_sender, event_receiver) = mpsc::channel(16);

        let state = Arc::new(Mutex::new(ControllerState {
            last_frame: now,
            next_frame: now,
            fps: 0.0,
            animation: None,
            event_generators: Self::start_generators(event_sender.clone()),
        }));

        let animation_join_handle = tokio::spawn(Self::run(state.clone(), client, points.len()));
        let event_generator_join_handle =
            tokio::spawn(Self::event_loop(state.clone(), event_receiver));
        let animation_factory = AnimationFactory::new(plugin_dir, points);

        Self {
            state,
            animation_join_handle,
            event_generator_join_handle,
            animation_factory,
            event_sender,
        }
    }

    fn start_generators(event_sender: Sender<Event>) -> Vec<Box<dyn Send + Sync>> {
        let mut generators = vec![
            Box::new(BeatEventGenerator::new(60.0, event_sender.clone())) as Box<dyn Send + Sync>,
        ];
        match FftEventGenerator::new(30.0, event_sender.clone()) {
            Ok(generator) => generators.push(Box::new(generator)),
            Err(e) => {
                warn!("Failed to initialize FFT generator: {}", e);
            }
        };
        match MidiEventGenerator::new(event_sender) {
            Ok(generator) => generators.push(Box::new(generator)),
            Err(e) => {
                warn!("Failed to initialize MIDI generator: {}", e);
            }
        };
        generators
    }

    async fn run(
        state: Arc<Mutex<ControllerState>>,
        client: Box<dyn rustmas_light_client::LightClient + Sync + Send>,
        point_count: usize,
    ) {
        let start_backoff_delay: Duration = Duration::seconds(1);
        let max_backoff_delay: Duration = Duration::seconds(8);

        let mut backoff_delay = start_backoff_delay;
        let mut status = ConnectionStatus::Healthy;
        let now = Utc::now();
        let mut next_check = now;

        loop {
            tokio::time::sleep(
                (next_check - Utc::now())
                    .clamp(Duration::milliseconds(0), Duration::milliseconds(33))
                    .to_std()
                    .unwrap(),
            )
            .await;

            let now = Utc::now();
            let (Ok(frame),) = ({
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

                let delta = now - state.last_frame;
                state.last_frame = now;
                if let Some(ref mut animation) = state.animation {
                    animation
                        .update(delta.num_milliseconds() as f64 / 1000.0)
                        .and_then(|_| animation.render())
                } else {
                    Ok(lightfx::Frame::new_black(point_count))
                }
            },) else {
                continue;
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

    async fn event_loop(state: Arc<Mutex<ControllerState>>, mut receiver: mpsc::Receiver<Event>) {
        while let Some(event) = receiver.recv().await {
            let state = state.lock().await;
            if let Some(animation) = &state.animation {
                let _ = animation.send_event(event);
            }
        }
    }

    pub fn builder() -> ControllerBuilder {
        ControllerBuilder {
            points: None,
            plugin_dir_: None,
            client_builder: CombinedLightClient::builder(),
        }
    }

    pub fn points(&self) -> &[(f64, f64, f64)] {
        self.animation_factory.points()
    }

    pub async fn restart_event_generators(&self) {
        info!("Restarting event generators");
        let mut state = self.state.lock().await;

        state.event_generators = Vec::new();
        state.event_generators = Self::start_generators(self.event_sender.clone());
    }

    pub async fn switch_animation(&self, name: &str) -> Result<(), ControllerError> {
        info!("Trying to switch animation to \"{}\"", name);
        let mut state = self.state.lock().await;
        let new_animation = self.animation_factory.make(name)?;

        let now = Utc::now();
        state.fps = new_animation.get_fps()?;
        state.last_frame = now;
        state.next_frame = now;
        state.animation = Some(new_animation);
        Ok(())
    }

    pub async fn reload_animation(&self) -> Result<(), ControllerError> {
        let mut state = self.state.lock().await;
        let Some(name) = state
            .animation
            .as_ref()
            .map(AnimationPlugin::animation_name)
            .transpose()?
        else {
            return Ok(());
        };

        info!("Reloading animation \"{}\"", name);
        let new_animation = self.animation_factory.make(&name)?;

        let now = Utc::now();
        state.fps = new_animation.get_fps()?;
        state.last_frame = now;
        state.next_frame = now;
        state.animation = Some(new_animation);
        Ok(())
    }

    pub async fn turn_off(&self) {
        info!("Turning off the animation");
        let mut state = self.state.lock().await;

        let now = Utc::now();
        state.fps = 0.0;
        state.last_frame = now;
        state.next_frame = now;
        state.animation = None;
    }

    pub async fn parameters(&self) -> Result<serde_json::Value, ControllerError> {
        if let Some(animation) = &self.state.lock().await.animation {
            Ok(json!({
                "name": animation.animation_name()?,
                "schema": animation.parameter_schema()?,
                "values": animation.get_parameters()?,
            }))
        } else {
            Ok(json!(()))
        }
    }

    pub async fn parameter_values(&self) -> Result<serde_json::Value, ControllerError> {
        if let Some(animation) = &self.state.lock().await.animation {
            Ok(animation.get_parameters()?)
        } else {
            Ok(json!(()))
        }
    }

    pub async fn set_parameters(
        &mut self,
        parameters: serde_json::Value,
    ) -> Result<(), ControllerError> {
        let mut state = self.state.lock().await;
        if let Some(ref mut animation) = state.animation {
            animation.set_parameters(parameters)?;
            state.next_frame = Utc::now();
        }
        Ok(())
    }

    pub async fn join(self) -> Result<(), ControllerError> {
        self.animation_join_handle
            .await
            .map_err(|e| ControllerError::InternalError {
                reason: e.to_string(),
            })?;

        self.event_generator_join_handle
            .await
            .map_err(|e| ControllerError::InternalError {
                reason: e.to_string(),
            })?;

        Ok(())
    }

    pub fn discover_animations(&mut self) -> Result<(), ControllerError> {
        self.animation_factory.discover()?;
        Ok(())
    }

    pub fn list_animations(&self) -> &HashMap<String, Plugin> {
        self.animation_factory.list()
    }
}

pub struct ControllerBuilder {
    points: Option<Vec<(f64, f64, f64)>>,
    plugin_dir_: Option<PathBuf>,
    client_builder: CombinedLightClientBuilder,
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

    pub fn http_lights(mut self, path: &str) -> Result<Self, Box<dyn Error>> {
        info!("Using http light client with endpoint {}", path);
        self.client_builder = self
            .client_builder
            .with(Box::new(client::http::HttpLightClient::new(path)));
        Ok(self)
    }

    pub fn tcp_lights(mut self, path: &str) -> Result<Self, Box<dyn Error>> {
        info!("Using tcp light client with endpoint {}", path);
        self.client_builder = self
            .client_builder
            .with(Box::new(client::tcp::TcpLightClient::new(path)));
        Ok(self)
    }

    pub fn udp_lights(mut self, path: &str) -> Result<Self, Box<dyn Error>> {
        info!("Using udp light client with endpoint {}", path);
        self.client_builder = self
            .client_builder
            .with(Box::new(client::udp::UdpLightClient::new(path)));
        Ok(self)
    }

    pub fn local_lights(mut self) -> Result<Self, Box<dyn Error>> {
        info!("Using tty lights client");
        self.client_builder = self
            .client_builder
            .with(Box::new(client::tty::TtyLightClient::new()?));
        Ok(self)
    }

    pub fn lights_feedback(mut self, sender: mpsc::Sender<lightfx::Frame>) -> Self {
        self.client_builder = self
            .client_builder
            .with(Box::new(client::feedback::FeedbackLightClient::new(sender)));
        self
    }

    pub fn plugin_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.plugin_dir_ = Some(path.as_ref().into());
        self
    }

    pub fn build(self) -> Controller {
        Controller::new(
            self.points.unwrap(),
            self.plugin_dir_.unwrap(),
            self.client_builder.build(),
        )
    }
}
