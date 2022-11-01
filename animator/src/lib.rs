mod animations;

use std::{error::Error, sync::mpsc, time::Duration};

use log::{error, info, warn};
use rustmas_light_client as client;
use rustmas_light_client::LightClientError;
use tokio::task::JoinHandle;

use animations::Animation;

pub struct Controller {
    join_handle: JoinHandle<()>,
    tx: mpsc::Sender<String>,
}

impl Controller {
    pub fn new(
        points: Vec<(f64, f64, f64)>,
        client: Box<dyn rustmas_light_client::LightClient + Sync + Send>,
    ) -> Result<Self, Box<dyn Error>> {
        let (tx, rx) = mpsc::channel::<String>();

        let join_handle = tokio::spawn(async move {
            const FRAME_STEP: f64 = 1.0 / 30.0;

            let mut animation: Box<dyn Animation + Sync + Send> =
                animations::make_animation("blank", &points);
            let mut t = 0.0;
            let mut delay = FRAME_STEP;

            loop {
                animation = match rx.try_recv() {
                    Ok(name) => animations::make_animation(name.as_str(), &points),
                    Err(mpsc::TryRecvError::Empty) => animation,
                    _ => {
                        info!("Animation channel closed, exiting");
                        return;
                    }
                };

                match client.display_frame(&animation.frame(t)).await {
                    Ok(_) => {
                        t += FRAME_STEP;
                        delay = FRAME_STEP; // restore default delay
                    }
                    Err(LightClientError::ConnectionLost) => {
                        delay = (delay * 2.0).min(5.0);
                        warn!(
                            "Lost connection to light client, will retry in {} seconds",
                            delay
                        );
                    }
                    Err(LightClientError::ProcessExited) => {
                        warn!("Light client exited, exiting");
                        return;
                    }
                    _ => (),
                };

                tokio::time::sleep(Duration::from_secs_f64(delay)).await;
            }
        });

        Ok(Self { join_handle, tx })
    }

    pub fn builder() -> ControllerBuilder {
        ControllerBuilder {
            points: None,
            client: None,
        }
    }

    pub fn switch_animation(&self, name: &str) -> Result<(), Box<dyn Error>> {
        info!("Trying to switch animation to \"{}\"", name);
        match self.tx.send(name.to_owned()) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to switch animation, reason: {}", e);
                Err(Box::new(e))
            }
        }
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
