use std::{error::Error, sync::mpsc, time::Duration};

use log::{error, info, warn};
use rustmas_light_client::LightClientError;
use tokio::task::JoinHandle;

use crate::animations::{self, Animation};

pub struct Controller {
    join_handle: JoinHandle<()>,
    tx: mpsc::Sender<String>,
}

impl Controller {
    fn new_animation(
        name: &str,
        points: &Vec<(f64, f64, f64)>,
    ) -> Box<dyn Animation + Sync + Send> {
        match name {
            "rainbow_cylinder" => Box::new(animations::RainbowCylinder::new(points)),
            "rainbow_sphere" => Box::new(animations::RainbowSphere::new(points)),
            "rainbow_waterfall" => Box::new(animations::RainbowWaterfall::new(points)),
            "sweep" => Box::new(animations::Sweep::new(points)),
            "rgb" => Box::new(animations::Rgb::new(points)),
            _ => panic!("Unknown animation pattern \"{}\"", name),
        }
    }

    pub fn new(
        points: Vec<(f64, f64, f64)>,
        client: Box<dyn rustmas_light_client::LightClient + Sync + Send>,
    ) -> Result<Self, Box<dyn Error>> {
        let (tx, rx) = mpsc::channel::<String>();

        let join_handle = tokio::spawn(async move {
            let mut animation: Box<dyn Animation + Sync + Send> =
                Box::new(super::animations::RainbowWaterfall::new(&points));
            let mut t = 0.0;

            loop {
                animation = match rx.try_recv() {
                    Ok(name) => Self::new_animation(name.as_str(), &points),
                    Err(mpsc::TryRecvError::Empty) => animation,
                    _ => {
                        info!("Animation channel closed, exiting");
                        return;
                    }
                };

                match client.display_frame(&animation.frame(t)).await {
                    Err(LightClientError::ConnectionLost) => {
                        warn!("Lost connection to light client, exiting");
                        return;
                    }
                    _ => (),
                };

                let frame_step = 1.0 / 30.0;
                tokio::time::sleep(Duration::from_secs_f64(frame_step)).await;
                t += frame_step;
            }
        });

        Ok(Self { join_handle, tx })
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
