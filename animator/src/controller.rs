use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use animation_api::event::Event;
use animation_api::schema::{Configuration, ParameterValue};
use chrono::{DateTime, Duration, Utc};
use client::combined::{CombinedLightClient, CombinedLightClientBuilder};
use events::beat_generator::BeatEventGenerator;
use events::event_generator::EventGenerator;
use events::fft_generator::FftEventGenerator;
use events::midi_generator::MidiEventGenerator;
use log::{info, warn};
use rustmas_light_client as client;
use rustmas_light_client::LightClientError;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::factory::{AnimationFactory, AnimationFactoryError};
use crate::jsonrpc::JsonRpcPlugin;
use crate::plugin::{AnimationPluginError, Plugin, PluginConfig};

#[derive(Debug, thiserror::Error)]
pub enum ControllerError {
    #[error("animation controller error: {reason}")]
    InternalError { reason: String },

    #[error(transparent)]
    AnimationPluginError(#[from] AnimationPluginError),

    #[error("animation factory error: {0}")]
    AnimationFactoryError(#[from] AnimationFactoryError),

    #[error("no animation selected")]
    NoAnimationSelected,
}

struct ControllerState {
    animation: Option<JsonRpcPlugin>,
    last_frame: DateTime<Utc>,
    next_frame: DateTime<Utc>,
    fps: f64,
    event_generators: HashMap<String, Box<dyn EventGenerator>>,
}

impl ControllerState {
    fn set_animation(&mut self, animation: Option<JsonRpcPlugin>) -> Result<(), ControllerError> {
        let now = Utc::now();
        self.fps = animation
            .as_ref()
            .map(|a| a.get_fps())
            .transpose()?
            .unwrap_or_default();
        self.last_frame = now;
        self.next_frame = now;
        self.animation = animation;
        Ok(())
    }
}

pub struct Controller {
    animation_join_handle: JoinHandle<()>,
    event_generator_join_handle: JoinHandle<()>,
    state: Arc<Mutex<ControllerState>>,
    animation_factory: AnimationFactory,
}

impl Controller {
    fn new<P: AsRef<Path>>(
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
            event_generators: Self::start_generators(event_sender),
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
        }
    }

    pub fn current_animation(&self) -> Option<PluginConfig> {
        self.state
            .lock()
            .unwrap()
            .animation
            .as_ref()
            .map(|animation| animation.plugin_config().clone())
    }

    fn start_generators(
        event_sender: mpsc::Sender<Event>,
    ) -> HashMap<String, Box<dyn EventGenerator>> {
        HashMap::from_iter([
            (
                "beat".into(),
                Box::new(BeatEventGenerator::new(60.0, event_sender.clone()))
                    as Box<dyn EventGenerator>,
            ),
            (
                "fft".into(),
                Box::new(FftEventGenerator::new(30.0, event_sender.clone())),
            ),
            (
                "midi".into(),
                Box::new(MidiEventGenerator::new(event_sender)),
            ),
        ])
    }

    async fn run(
        state: Arc<Mutex<ControllerState>>,
        client: Box<dyn rustmas_light_client::LightClient + Sync + Send>,
        point_count: usize,
    ) {
        let mut next_check = Utc::now();

        loop {
            let now = Utc::now();
            tokio::time::sleep(
                (next_check - now)
                    .clamp(Duration::milliseconds(0), Duration::milliseconds(33))
                    .to_std()
                    .unwrap(),
            )
            .await;

            let (Ok(frame),) = ({
                let mut state = state.lock().unwrap();
                if now < state.next_frame {
                    next_check = state.next_frame.min(now + Duration::seconds(1));
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

            if client.display_frame(&frame).await == Err(LightClientError::ProcessExited) {
                warn!("Light client exited, exiting");
                return;
            }
        }
    }

    async fn event_loop(state: Arc<Mutex<ControllerState>>, mut receiver: mpsc::Receiver<Event>) {
        while let Some(event) = receiver.recv().await {
            let state = state.lock().unwrap();
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

    pub fn restart_event_generators(&self) {
        info!("Restarting event generators");
        self.state
            .lock()
            .unwrap()
            .event_generators
            .iter_mut()
            .for_each(|(_, evg)| evg.restart());
    }

    pub fn get_event_generator_parameters(&self) -> Vec<Configuration> {
        self.state
            .lock()
            .unwrap()
            .event_generators
            .iter()
            .map(|(id, evg)| Configuration {
                id: id.clone(),
                name: evg.get_name().to_owned(),
                schema: evg.get_schema(),
                values: evg.get_parameters(),
            })
            .collect()
    }

    pub fn set_event_generator_parameters(
        &self,
        values: &HashMap<String, HashMap<String, ParameterValue>>,
    ) -> Result<(), ControllerError> {
        let mut state = self.state.lock().unwrap();

        for (id, parameters) in values {
            let Some(evg) = state.event_generators.get_mut(id) else {
                warn!("No such event generator: {}", id);
                continue;
            };

            evg.set_parameters(parameters)
                .map_err(|e| ControllerError::InternalError {
                    reason: e.to_string(),
                })?;
        }

        Ok(())
    }

    pub fn reload_animation(&self) -> Result<Configuration, ControllerError> {
        let mut state = self.state.lock().unwrap();
        let Some(id) = state
            .animation
            .as_ref()
            .map(|a| a.plugin_config().animation_id())
        else {
            return Err(ControllerError::NoAnimationSelected);
        };
        info!("Reloading animation \"{}\"", id);
        let animation = self.animation_factory.make(id)?;
        let configuration = animation.configuration()?;
        state.set_animation(Some(animation))?;
        Ok(configuration)
    }

    pub fn switch_animation(&self, animation_id: &str) -> Result<Configuration, ControllerError> {
        info!("Trying to switch animation to \"{}\"", animation_id);
        let mut state = self.state.lock().unwrap();
        let animation = self.animation_factory.make(animation_id)?;
        let configuration = animation.configuration()?;
        state.set_animation(Some(animation))?;
        Ok(configuration)
    }

    pub fn turn_off(&self) {
        info!("Turning off the animation");
        let _ = self.state.lock().unwrap().set_animation(None);
    }

    pub fn get_parameters(&self) -> Result<Option<Configuration>, ControllerError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .animation
            .as_ref()
            .map(Plugin::configuration)
            .transpose()?)
    }

    pub fn get_parameter_values(&self) -> Result<HashMap<String, ParameterValue>, ControllerError> {
        if let Some(animation) = &self.state.lock().unwrap().animation {
            Ok(animation.get_parameters()?)
        } else {
            Ok(HashMap::new())
        }
    }

    pub fn set_parameters(
        &mut self,
        parameters: &HashMap<String, ParameterValue>,
    ) -> Result<(), ControllerError> {
        let mut state = self.state.lock().unwrap();
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

    pub fn list_animations(&self) -> &HashMap<String, PluginConfig> {
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
            .from_path(path)
            .map_err(|e| ControllerError::InternalError {
                reason: format!("Could not read CSV file: {}", e),
            })?
            .deserialize()
            .filter_map(|record: Result<(f64, f64, f64), _>| record.ok())
            .collect();
        info!("Loaded {} points from {}", points.len(), path);

        self.points = Some(points);
        Ok(self)
    }

    pub fn http_lights(mut self, path: &str) -> Result<Self, Box<dyn Error>> {
        info!("Using http light client with endpoint {}", path);
        self.client_builder = self.client_builder.with(Box::new(
            client::http::HttpLightClient::new(path).with_backoff(),
        ));
        Ok(self)
    }

    pub fn tcp_lights(mut self, path: &str) -> Result<Self, Box<dyn Error>> {
        info!("Using tcp light client with endpoint {}", path);
        self.client_builder = self.client_builder.with(Box::new(
            client::tcp::TcpLightClient::new(path)
                .with_backoff()
                .with_start_delay(Duration::milliseconds(125)),
        ));
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
