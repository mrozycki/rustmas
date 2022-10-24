use std::{error::Error, time::Duration};

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

fn to_polar((x, y, z): &(f64, f64, f64)) -> (f64, f64, f64) {
    ((x.powi(2) + z.powi(2)).sqrt(), (x / z).atan(), *y)
}

fn generate_rainbow_cylinder(t: f64, points: &Vec<(f64, f64, f64)>) -> client::Frame {
    points
        .iter()
        .map(to_polar)
        .map(|(_, a, _)| client::Color::hsv(t + a / (2.0 * std::f64::consts::PI), 1.0, 1.0))
        .collect::<Vec<_>>()
        .into()
}

fn generate_rainbow_sphere(t: f64, points: &Vec<(f64, f64, f64)>) -> client::Frame {
    points
        .iter()
        .map(|(x, y, z)| (x.powi(2) + y.powi(2) + z.powi(2)).sqrt())
        .map(|r| client::Color::hsv(r + t, 1.0, 1.0))
        .collect::<Vec<_>>()
        .into()
}

fn generate_rainbow_waterfall(t: f64, points: &Vec<(f64, f64, f64)>) -> client::Frame {
    points
        .iter()
        .map(|(_, h, _)| (h + 1.0) / 2.0)
        .map(|h| client::Color::hsv(h + t, 1.0, 0.5))
        .collect::<Vec<_>>()
        .into()
}

fn generate_sweep(t: f64, points: &Vec<(f64, f64, f64)>) -> client::Frame {
    let t = t.fract() * 2.0 - 1.0;

    points
        .iter()
        .map(|(_, h, _)| {
            if *h > t && *h < t + 0.2 {
                client::Color::white()
            } else {
                client::Color::black()
            }
        })
        .collect::<Vec<_>>()
        .into()
}

fn generate_rgb(t: f64, points: &Vec<(f64, f64, f64)>) -> client::Frame {
    (0..points.len() as isize)
        .into_iter()
        .map(|x| match (x + ((t * 3.0) as isize)) % 3 {
            0 => client::Color::rgb(255, 0, 0),
            1 => client::Color::rgb(0, 255, 0),
            _ => client::Color::rgb(0, 0, 255),
        })
        .collect::<Vec<_>>()
        .into()
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

    let animation = match cli.animation.as_str() {
        "rainbow_cylinder" => generate_rainbow_cylinder,
        "rainbow_sphere" => generate_rainbow_sphere,
        "rainbow_waterfall" => generate_rainbow_waterfall,
        "sweep" => generate_sweep,
        "rgb" => generate_rgb,
        _ => panic!("Unknown animation pattern \"{}\"", cli.animation),
    };

    let mut t = 0.0;
    loop {
        client.display_frame(&animation(t, &points)).await?;
        std::thread::sleep(Duration::from_millis(33));
        t += 0.033;
    }
}
