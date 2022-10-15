mod capture;
mod cv;

use std::{error::Error, fs::File};

use capture::Capturer;
use clap::{arg, Parser, Subcommand};
use cv::Camera;
use log::{info, LevelFilter};
use rustmas_light_client as light_client;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

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
        #[arg(short, long)]
        lights_endpoint: Option<String>,
        #[arg(short, long)]
        ip_camera: Option<String>,
        #[arg(short, long, default_value_t = 500)]
        number_of_lights: usize,
    },
    OpenCVExample {
        #[arg(short, long)]
        lights_endpoint: Option<String>,
        #[arg(short, long)]
        ip_camera: Option<String>,
        #[arg(short, long, default_value_t = 500)]
        number_of_lights: usize,
    },
    Visualise {
        input: String,
    },
}

fn capturer_from_options(
    lights_endpoint: Option<String>,
    ip_camera: Option<String>,
    number_of_lights: usize,
) -> Result<Capturer, Box<dyn Error>> {
    let camera = if let Some(path) = ip_camera {
        info!("Using camera from file: {}", path);
        Camera::new_from_file(&path)?
    } else {
        info!("Using default camera");
        Camera::new_default()?
    };

    let light_client: Box<dyn light_client::LightClient> = if let Some(endpoint) = lights_endpoint {
        info!("Using remote light client at endpoint: {}", endpoint);
        Box::new(light_client::RemoteLightClient::new(&endpoint))
    } else {
        info!("Using mock light client");
        Box::new(light_client::MockLightClient::new())
    };

    Ok(Capturer::new(light_client, camera, number_of_lights))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("configurator.log")?,
        ),
    ])?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Capture {
            output,
            number_of_lights,
            ip_camera,
            lights_endpoint,
        } => {
            let mut capturer = capturer_from_options(lights_endpoint, ip_camera, number_of_lights)?;

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
        Commands::OpenCVExample {
            ip_camera,
            number_of_lights,
            lights_endpoint,
        } => {
            let mut capturer = capturer_from_options(lights_endpoint, ip_camera, number_of_lights)?;

            capturer.opencv_example().await?;

            Ok(())
        }
        Commands::Visualise { input } => {
            let mut file = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_path(input)?;
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
