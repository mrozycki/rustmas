mod animations;

use std::{error::Error, time::Duration};

use clap::Parser;
use client::LightClient;
use rustmas_light_client as client;

use animations::Animation;

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
    let client: Box<dyn LightClient> = match cli.lights_endpoint {
        Some(path) => Box::new(client::RemoteLightClient::new(&path)),
        None => Box::new(client::VisualiserLightClient::new(points.clone())?),
    };

    let animation: Box<dyn Animation> = match cli.animation.as_str() {
        "rainbow_cylinder" => Box::new(animations::RainbowCylinder::new(points)),
        "rainbow_sphere" => Box::new(animations::RainbowSphere::new(points)),
        "rainbow_waterfall" => Box::new(animations::RainbowWaterfall::new(points)),
        "sweep" => Box::new(animations::Sweep::new(points)),
        "rgb" => Box::new(animations::Rgb::new(points)),
        _ => panic!("Unknown animation pattern \"{}\"", cli.animation),
    };

    let mut t = 0.0;
    loop {
        client.display_frame(&animation.frame(t)).await?;
        std::thread::sleep(Duration::from_millis(33));
        t += 0.033;
    }
}
