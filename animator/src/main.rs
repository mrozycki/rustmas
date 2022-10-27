mod animations;
mod controller;

use std::error::Error;

use clap::Parser;
use client::LightClient;
use rustmas_light_client as client;

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
    let cli = Cli::parse();

    let points = load_points(&cli.positions_file)?;
    let client: Box<dyn LightClient + Send + Sync> = match cli.lights_endpoint {
        Some(path) => Box::new(client::RemoteLightClient::new(&path)),
        None => Box::new(client::VisualiserLightClient::new(points.clone())?),
    };

    let controller = controller::Controller::new(points, client)?;
    controller.switch_animation(&cli.animation)?;
    controller.join().await?;

    Ok(())
}
