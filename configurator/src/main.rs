mod capture;
mod cv;

use std::{error::Error, time::Duration};

use capture::Capturer;
use clap::{arg, Parser, Subcommand};
use cv::{Camera, Display};
use rustmas_light_client as light_client;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Capture {
        #[arg(short, long, default_value = "lights.csv")]
        output: String,
        #[arg(short, long, default_value_t = 500)]
        lights_count: usize,
        #[arg(short, long)]
        ip_camera: Option<String>,
    },
    OpenCVExample {
        #[arg(short, long)]
        ip_camera: Option<String>,
    },
    Visualise {
        input: String,
    },
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Capture {
            output,
            lights_count,
            ip_camera,
        } => {
            let camera = if let Some(path) = ip_camera {
                Camera::new_from_file(&path)?
            } else {
                Camera::new_default()?
            };

            let mut capturer = Capturer::new(
                Box::new(light_client::MockLightClient::new()),
                camera,
                lights_count,
            );

            capturer
                .wait_for_perspective("Position camera to capture lights from the front")
                .await?;
            let front = capturer.capture_perspective().await?;

            capturer
                .wait_for_perspective("Position camera to capture lights from the right-hand side")
                .await?;
            let side = capturer.capture_perspective().await?;

            let light_positions = Capturer::merge_perspectives(front, side);
            Capturer::save_positions(output, &light_positions)?;

            Ok(())
        }
        Commands::OpenCVExample { ip_camera } => {
            let mut camera = if let Some(path) = ip_camera {
                Camera::new_from_file(&path)?
            } else {
                Camera::new_default()?
            };
            let window = Display::new("video capture")?;

            loop {
                let mut frame = camera.capture()?;
                let (x, y) = cv::find_light(&frame)?;
                frame.mark(x, y)?;

                window.show(&frame)?;
                if window.wait_for(Duration::from_millis(10))? > 0 {
                    break;
                }
            }
            Ok(())
        }
        Commands::Visualise { input } => {
            let mut file = csv::Reader::from_path(input)?;
            let points = file
                .deserialize()
                .filter_map(|record: Result<(f32, f32, f32), _>| record.ok())
                .collect();

            let (tx, rx) = std::sync::mpsc::channel();
            tx.send(vec![(0.0, 1.0, 0.0); 500]).unwrap();
            tokio::spawn(async move {
                rustmas_visualiser::visualise(points, rx).unwrap();
            })
            .await?;
            Ok(())
        }
    }
}
