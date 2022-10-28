mod animations;
mod controller;

use std::{error::Error, fs::File};

use clap::Parser;
use client::LightClient;
use log::{info, LevelFilter};
use rustmas_light_client as client;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    lights_endpoint: Option<String>,
    #[arg(short, long, default_value = "lights.csv")]
    positions_file: String,
    #[arg(short, long, default_value = "rainbow_waterfall")]
    animation: String,
}

fn load_points(path: &str) -> Result<Vec<(f64, f64, f64)>, Box<dyn std::error::Error>> {
    Ok(csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?
        .deserialize()
        .filter_map(|record: Result<(f64, f64, f64), _>| record.ok())
        .collect())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("animator.log")?,
        ),
    ])?;

    let cli = Cli::parse();

    let points = load_points(&cli.positions_file)?;
    info!("Loaded {} points from {}", points.len(), cli.positions_file);

    let client: Box<dyn LightClient + Send + Sync> = match cli.lights_endpoint {
        Some(path) => {
            info!("Using remote light client with endpoint {}", path);
            Box::new(client::RemoteLightClient::new(&path))
        }
        None => {
            info!("Using local visualiser");
            Box::new(client::VisualiserLightClient::new(points.clone())?)
        }
    };

    let controller = controller::Controller::new(points, client)?;
    controller.switch_animation(&cli.animation)?;
    controller.join().await?;

    Ok(())
}
