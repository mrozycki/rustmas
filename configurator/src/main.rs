mod capture;
mod cv;

use std::{error::Error, fs};

use capture::{Capturer, WithConfidence};
use clap::{arg, Parser, Subcommand};
use cv::Camera;
use itertools::Itertools;
use log::{debug, info, LevelFilter};
use nalgebra::Vector3;
use rustmas_light_client as light_client;
use simplelog::{
    ColorChoice, CombinedLogger, Config, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};

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
        #[arg(short, long, default_value_t = false)]
        save_pictures: bool,
        #[arg(long = "left")]
        left_coordinates: Option<String>,
        #[arg(long = "right")]
        right_coordinates: Option<String>,
        #[arg(long = "front")]
        front_coordinates: Option<String>,
        #[arg(long = "back")]
        back_coordinates: Option<String>,
    },
    Capture2d {
        #[arg(short, long, default_value = "lights_2d.csv")]
        output: String,
        #[arg(short, long)]
        lights_endpoint: Option<String>,
        #[arg(short, long)]
        ip_camera: Option<String>,
        #[arg(short, long, default_value_t = 500)]
        number_of_lights: usize,
        #[arg(short, long, default_value_t = false)]
        save_pictures: bool,
    },
    Center {
        #[arg(short, long, default_value = "lights.csv")]
        input: String,
        #[arg(short, long, default_value = "lights.csv")]
        output: String,
        #[arg(short, long, default_value_t = 0.0)]
        x: f64,
        #[arg(short, long, default_value_t = 0.0)]
        y: f64,
        #[arg(short, long, default_value_t = 0.0)]
        z: f64,
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

async fn capture_or_read_coordinates(
    capturer: &mut Capturer,
    side: &str,
    save_pictures: bool,
    coordinates_path: Option<String>,
) -> Result<Vec<WithConfidence<(f64, f64)>>, Box<dyn Error>> {
    if coordinates_path.is_none() {
        capturer
            .wait_for_perspective(
                format!("Position camera to capture lights from the {side}").as_str(),
            )
            .await?;
        let coords = capturer.capture_perspective(side, save_pictures).await?;
        debug!("Captured positions from the {side}: {:?}", coords);
        Ok(coords)
    } else {
        debug!(
            "Reading {} side positions from {}",
            side,
            coordinates_path.as_ref().unwrap()
        );
        Capturer::read_coordinates_from_file(&coordinates_path.unwrap())
    }
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
            #[cfg(debug_assertions)]
            LevelFilter::Debug,
            #[cfg(not(debug_assertions))]
            LevelFilter::Info,
            ConfigBuilder::new().set_time_format_rfc3339().build(),
            fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open("configurator.log")?,
        ),
    ])?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Capture {
            output,
            number_of_lights,
            ip_camera,
            lights_endpoint,
            save_pictures,
            left_coordinates,
            right_coordinates,
            front_coordinates,
            back_coordinates,
        } => {
            let mut capturer = capturer_from_options(lights_endpoint, ip_camera, number_of_lights)?;
            let front = capture_or_read_coordinates(
                &mut capturer,
                "front",
                save_pictures,
                front_coordinates,
            )
            .await?;
            let right = capture_or_read_coordinates(
                &mut capturer,
                "right",
                save_pictures,
                right_coordinates,
            )
            .await?;
            let back =
                capture_or_read_coordinates(&mut capturer, "back", save_pictures, back_coordinates)
                    .await?;
            let left =
                capture_or_read_coordinates(&mut capturer, "left", save_pictures, left_coordinates)
                    .await?;

            let light_positions = Capturer::merge_perspectives(front, right, back, left);
            let light_positions = Capturer::interpolate_gaps(light_positions);
            let light_positions =
                Capturer::mark_outliers_by_distance(light_positions, |a, b| a.metric_distance(b));
            let light_positions = Capturer::interpolate_gaps(light_positions);
            let light_positions = Capturer::extrapolate_ends(light_positions);

            debug!("Mapped 3D light positions: {:?}", &light_positions);
            Capturer::save_3d_coordinates(output, &light_positions)?;

            Ok(())
        }
        Commands::Capture2d {
            output,
            lights_endpoint,
            ip_camera,
            number_of_lights,
            save_pictures,
        } => {
            let mut capturer = capturer_from_options(lights_endpoint, ip_camera, number_of_lights)?;
            let coords = capturer.capture_perspective("2d", save_pictures).await?;
            let light_positions = coords
                .iter()
                .map(|x| WithConfidence::<Vector3<f64>> {
                    inner: Vector3::<f64>::new(x.inner.0, x.inner.1, 0.0),
                    confidence: x.confidence,
                })
                .collect();
            let light_positions = Capturer::interpolate_gaps(light_positions);
            let light_positions = Capturer::extrapolate_ends(light_positions);

            Capturer::save_3d_coordinates(output, &light_positions)?;
            Ok(())
        }
        Commands::Center {
            input,
            output,
            x,
            y,
            z,
        } => {
            let points = Capturer::load_3d_coordinates(input)?
                .into_iter()
                .map(|point| WithConfidence {
                    inner: point - Vector3::new(x, y, z),
                    confidence: 1.0,
                })
                .collect_vec();
            Capturer::save_3d_coordinates(output, &points)?;

            Ok(())
        }
    }
}
