use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use animation_api::event::Event;
use animation_api::schema::{Configuration, ParameterValue};
use animation_wrapper::config::PluginConfig;
use chrono::{DateTime, Duration, Utc};
use client::combined::{CombinedLightClient, CombinedLightClientBuilder};
#[cfg(feature = "audio")]
use events::beat_generator::BeatEventGenerator;
use events::event_generator::EventGenerator;
#[cfg(feature = "audio")]
use events::fft_generator::FftEventGenerator;
#[cfg(feature = "midi")]
use events::midi_generator::MidiEventGenerator;
use lightfx::Frame;
use log::{info, warn};
use rustmas_light_client as client;
use rustmas_light_client::LightClientError;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;

use crate::factory::{AnimationFactory, AnimationFactoryError};
use crate::plugin::{AnimationPluginError, Plugin};
use crate::ControllerConfig;

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
    animation: Option<Box<dyn Plugin>>,
    last_frame: DateTime<Utc>,
    next_frame: DateTime<Utc>,
    fps: f64,
    event_generators: HashMap<String, Box<dyn EventGenerator>>,
}

impl ControllerState {
    async fn set_animation(
        &mut self,
        animation: Option<Box<dyn Plugin>>,
    ) -> Result<(), ControllerError> {
        let now = Utc::now();
        self.fps = if let Some(animation) = &animation {
            animation.get_fps().await?
        } else {
            0.0
        };
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
    event_sender: mpsc::Sender<Event>,
}

enum PollFrameResult {
    Ready(Frame),
    TryLater(DateTime<Utc>),
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

    pub async fn current_animation(&self) -> Option<PluginConfig> {
        self.state
            .lock()
            .await
            .animation
            .as_ref()
            .map(|animation| animation.plugin_config().clone())
    }

    #[allow(unused_variables)]
    fn start_generators(
        event_sender: mpsc::Sender<Event>,
    ) -> HashMap<String, Box<dyn EventGenerator>> {
        HashMap::from_iter([
            #[cfg(feature = "audio")]
            (
                "beat".into(),
                Box::new(BeatEventGenerator::new(60.0, event_sender.clone()))
                    as Box<dyn EventGenerator>,
            ),
            #[cfg(feature = "audio")]
            (
                "fft".into(),
                Box::new(FftEventGenerator::new(30.0, event_sender.clone())),
            ),
            #[cfg(feature = "midi")]
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

            let frame = match Self::poll_next_frame(&state, now, point_count).await {
                PollFrameResult::Ready(frame) => frame,
                PollFrameResult::TryLater(when) => {
                    next_check = when;
                    continue;
                }
            };

            if client.display_frame(&frame).await == Err(LightClientError::ProcessExited) {
                warn!("Light client exited, exiting");
                return;
            }
        }
    }

    async fn poll_next_frame(
        state: &Mutex<ControllerState>,
        now: DateTime<Utc>,
        frame_size: usize,
    ) -> PollFrameResult {
        let mut state = state.lock().await;
        let in_one_second = state.next_frame.min(now + Duration::seconds(1));
        if now < state.next_frame {
            return PollFrameResult::TryLater(in_one_second);
        }
        state.next_frame = if state.fps != 0.0 {
            now + Duration::milliseconds((1000.0 / state.fps) as i64)
        } else {
            now + Duration::days(1)
        };

        let delta = now - state.last_frame;
        state.last_frame = now;
        if let Some(ref mut animation) = state.animation {
            if animation
                .update(delta.num_milliseconds() as f64 / 1000.0)
                .await
                .is_err()
            {
                PollFrameResult::TryLater(in_one_second)
            } else if let Ok(frame) = animation.render().await {
                PollFrameResult::Ready(frame)
            } else {
                PollFrameResult::TryLater(in_one_second)
            }
        } else {
            PollFrameResult::Ready(lightfx::Frame::new_black(frame_size))
        }
    }

    async fn event_loop(state: Arc<Mutex<ControllerState>>, mut receiver: mpsc::Receiver<Event>) {
        while let Some(event) = receiver.recv().await {
            let state = state.lock().await;
            if let Some(animation) = &state.animation {
                let _ = animation.send_event(event).await;
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

    pub fn builder_from(config: ControllerConfig) -> Result<ControllerBuilder, Box<dyn Error>> {
        ControllerBuilder {
            points: None,
            plugin_dir_: Some(config.plugin_path),
            client_builder: CombinedLightClient::builder().with_config(config.lights)?,
        }
        .points_from_file(config.points_path)
    }

    pub fn points(&self) -> &[(f64, f64, f64)] {
        self.animation_factory.points()
    }

    pub async fn restart_event_generators(&self) {
        info!("Restarting event generators");
        self.state
            .lock()
            .await
            .event_generators
            .iter_mut()
            .for_each(|(_, evg)| evg.restart());
    }

    pub async fn get_event_generator_parameters(&self) -> Vec<Configuration> {
        self.state
            .lock()
            .await
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

    pub async fn set_event_generator_parameters(
        &self,
        values: &HashMap<String, HashMap<String, ParameterValue>>,
    ) -> Result<(), ControllerError> {
        let mut state = self.state.lock().await;

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

    pub async fn send_event(&self, event: Event) -> Result<(), ControllerError> {
        self.event_sender
            .send(event)
            .await
            .map_err(|e| ControllerError::InternalError {
                reason: e.to_string(),
            })
    }

    pub async fn reload_animation(&self) -> Result<Configuration, ControllerError> {
        let mut state = self.state.lock().await;
        let Some(id) = state
            .animation
            .as_ref()
            .map(|a| a.plugin_config().animation_id())
        else {
            return Err(ControllerError::NoAnimationSelected);
        };
        info!("Reloading animation \"{}\"", id);
        let animation = self.animation_factory.make(id).await?;
        let configuration = animation.configuration().await?;
        state.set_animation(Some(animation)).await?;
        Ok(configuration)
    }

    pub async fn switch_animation(
        &self,
        animation_id: &str,
    ) -> Result<Configuration, ControllerError> {
        info!("Trying to switch animation to \"{}\"", animation_id);
        let animation = self.animation_factory.make(animation_id).await?;
        let configuration = animation.configuration().await?;
        let mut state = self.state.lock().await;
        state.set_animation(Some(animation)).await?;
        Ok(configuration)
    }

    pub async fn turn_off(&self) {
        info!("Turning off the animation");
        let _ = self.state.lock().await.set_animation(None).await;
    }

    pub async fn get_parameters(&self) -> Result<Configuration, ControllerError> {
        let state = self.state.lock().await;
        let Some(animation) = &state.animation else {
            return Err(ControllerError::NoAnimationSelected);
        };
        Ok(animation.configuration().await?)
    }

    pub async fn get_parameter_values(
        &self,
    ) -> Result<HashMap<String, ParameterValue>, ControllerError> {
        if let Some(animation) = &self.state.lock().await.animation {
            Ok(animation.get_parameters().await?)
        } else {
            Ok(HashMap::new())
        }
    }

    pub async fn set_parameters(
        &mut self,
        parameters: &HashMap<String, ParameterValue>,
    ) -> Result<Configuration, ControllerError> {
        let mut state = self.state.lock().await;
        if let Some(ref mut animation) = state.animation {
            animation.set_parameters(parameters).await?;
            let configuration = animation.configuration().await?;
            state.next_frame = Utc::now();
            Ok(configuration)
        } else {
            Err(ControllerError::NoAnimationSelected)
        }
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
    pub fn points_from_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Box<dyn Error>> {
        fn points_from_path(path: &Path) -> Result<Vec<(f64, f64, f64)>, ControllerError> {
            let points: Vec<_> = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_path(path)
                .map_err(|e| ControllerError::InternalError {
                    reason: format!("Could not read CSV file: {}", e),
                })?
                .deserialize()
                .filter_map(|record: Result<(f64, f64, f64), _>| record.ok())
                .collect();
            info!(
                "Loaded {} points from {}",
                points.len(),
                path.to_string_lossy()
            );
            Ok(points)
        }

        let path = path.as_ref();
        self.points = Some(points_from_path(path)?);
        Ok(self)
    }

    pub fn lights(
        mut self,
        config: Vec<rustmas_light_client::LightsConfig>,
    ) -> Result<Self, Box<dyn Error>> {
        self.client_builder = self.client_builder.with_config(config)?;
        Ok(self)
    }

    pub fn lights_feedback(mut self, sender: mpsc::Sender<lightfx::Frame>) -> Self {
        self.client_builder = self
            .client_builder
            .with(client::feedback::FeedbackLightClient::new(sender));
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
