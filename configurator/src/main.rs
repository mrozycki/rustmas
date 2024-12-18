mod capture;
mod cv;

use std::{error::Error, path::PathBuf};

use capture::{Capturer, WithConfidence};
use clap::{arg, Parser, Subcommand};
use cv::Camera;
use itertools::Itertools;
use log::{debug, info};
use nalgebra::Vector3;
use rustmas_light_client::{
    combined::CombinedLightClient, ByteOrder, LightsConfig, LightsEndpoint, TtyLightsConfig,
};
use url::Url;

#[derive(Debug, thiserror::Error)]
#[error("Configurator error: {message}")]
pub struct ConfiguratorError {
    pub message: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get the difference of two pictures (mostly for demo purposes)
    Difference {
        #[arg(short, long, num_args = 2, value_name = "JPG_PATH")]
        pictures: Vec<PathBuf>,
        #[arg(short, long, default_value = std::env::current_dir().unwrap_or(".".into()).into_os_string())]
        output_dir: PathBuf,
    },

    /// Capture 3D coordinates of lights by measuring from 4 sides
    Capture {
        #[arg(short, long, default_value = "lights.csv")]
        output: String,
        #[arg(short, long)]
        lights_endpoint: Option<String>,
        #[arg(short, long)]
        camera: Option<String>,
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
    /// Capture 2D coordinates of lights by measuring from 1 side
    Capture2d {
        #[arg(short, long, default_value = "lights_2d.csv")]
        output: String,
        #[arg(short, long)]
        lights_endpoint: Option<String>,
        #[arg(short, long)]
        camera: Option<String>,
        #[arg(short, long, default_value_t = 500)]
        number_of_lights: usize,
        #[arg(short, long, default_value_t = false)]
        save_pictures: bool,
        #[arg(long = "override")]
        override_coordinates: Option<String>,
    },
    CaptureFastTest {
        #[arg(short, long, default_value = "lights_2d.csv")]
        output: String,
        #[arg(short, long)]
        lights_endpoint: Option<String>,
        #[arg(short, long)]
        camera: Option<String>,
        #[arg(short, long, default_value_t = 500)]
        number_of_lights: usize,
        #[arg(short, long, default_value_t = false)]
        save_pictures: bool,
    },
    /// Merge measurements taken from 4 perspectives to produce 3D coordinates
    Merge {
        #[arg(short, long, default_value = "lights.csv")]
        output: String,
        #[arg(long = "left")]
        left_coordinates: String,
        #[arg(long = "right")]
        right_coordinates: String,
        #[arg(long = "front")]
        front_coordinates: String,
        #[arg(long = "back")]
        back_coordinates: String,
    },
    /// Translate lights in 3D space to match tree's center
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
    camera: Option<String>,
    number_of_lights: usize,
) -> Result<Capturer, Box<dyn Error>> {
    let camera = if let Some(Ok(index)) = camera.as_ref().map(|s| s.parse::<i32>()) {
        Camera::new_local(index)?
    } else if let Some(path) = camera {
        info!("Using camera from file: {}", path);
        Camera::new_from_file(&path)?
    } else {
        info!("Using default camera");
        Camera::new_default()?
    };

    let light_client_config = lights_endpoint
        .and_then(|endpoint| {
            if endpoint.starts_with("http://")
                || endpoint.starts_with("tcp://")
                || endpoint.starts_with("udp://")
            {
                let Ok(url) = Url::parse(&endpoint) else {
                    eprintln!("Invalid url: {endpoint}");
                    return None;
                };
                Some(LightsEndpoint::Remote(url))
            } else {
                Some(LightsEndpoint::Tty(TtyLightsConfig::Detect))
            }
        })
        .map(|endpoint| LightsConfig {
            endpoint,
            byte_order: ByteOrder::Rgb,
        })
        .into_iter()
        .collect_vec();

    let light_client = CombinedLightClient::builder()
        .with_config(&light_client_config)?
        .build();

    Ok(Capturer::new(light_client, camera, number_of_lights))
}

async fn capture_or_read_coordinates(
    capturer: &mut Capturer,
    side: &str,
    save_pictures: bool,
    coordinates_path: Option<String>,
) -> Result<Vec<WithConfidence<(f64, f64)>>, Box<dyn Error>> {
    if let Some(coordinates_path) = coordinates_path {
        debug!("Reading {} side positions from {}", side, coordinates_path);
        Capturer::read_coordinates_from_file(&coordinates_path)
    } else {
        capturer
            .wait_for_perspective(
                format!("Position camera to capture lights from the {side}").as_str(),
            )
            .await?;
        let coords = capturer.capture_perspective(side, save_pictures).await?;
        debug!("Captured positions from the {side}: {:?}", coords);
        Ok(coords)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Difference {
            pictures,
            output_dir,
        } => {
            // since num_args is set to 2, clap will make sure there are two paths provided
            let before = cv::Picture::from_file(&pictures[0])?;
            let after = cv::Picture::from_file(&pictures[1])?;

            if !output_dir.exists() {
                std::fs::create_dir_all(&output_dir)?;
            }
            if !output_dir.is_dir() {
                Err(ConfiguratorError {
                    message: "Wrong output dir!".to_owned(),
                })?;
            }

            let coords = cv::find_light_from_diff_with_output(&before, &after, Some(&output_dir))?;
            println!(
                "Found coords: x:{},y:{}, confidence: {}",
                coords.inner.0, coords.inner.1, coords.confidence
            );

            Ok(())
        }
        Commands::Capture {
            output,
            number_of_lights,
            camera,
            lights_endpoint,
            save_pictures,
            left_coordinates,
            right_coordinates,
            front_coordinates,
            back_coordinates,
        } => {
            let mut capturer = capturer_from_options(lights_endpoint, camera, number_of_lights)?;
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
            camera,
            number_of_lights,
            save_pictures,
            override_coordinates,
        } => {
            let mut capturer = capturer_from_options(lights_endpoint, camera, number_of_lights)?;
            let coords = capture_or_read_coordinates(
                &mut capturer,
                "2d",
                save_pictures,
                override_coordinates,
            )
            .await?;
            let light_positions = coords
                .iter()
                .map(|x| WithConfidence::<Vector3<f64>> {
                    inner: Vector3::<f64>::new(x.inner.0, -x.inner.1, 0.0),
                    confidence: x.confidence,
                })
                .collect();
            let light_positions = Capturer::interpolate_gaps(light_positions);
            let light_positions = Capturer::extrapolate_ends(light_positions);

            Capturer::save_3d_coordinates(output, &light_positions)?;
            Ok(())
        }
        Commands::CaptureFastTest {
            output,
            lights_endpoint,
            camera,
            number_of_lights,
            save_pictures,
        } => {
            let mut capturer = capturer_from_options(lights_endpoint, camera, number_of_lights)?;
            let coords = capturer
                .capture_perspective_fast("2d", save_pictures)
                .await?;
            let light_positions = coords
                .iter()
                .map(|x| WithConfidence::<Vector3<f64>> {
                    inner: Vector3::<f64>::new(x.inner.0, -x.inner.1, 0.0),
                    confidence: x.confidence,
                })
                .collect_vec();
            Capturer::save_3d_coordinates(output, &light_positions)?;
            Ok(())
        }
        Commands::Merge {
            output,
            left_coordinates,
            right_coordinates,
            front_coordinates,
            back_coordinates,
        } => {
            let left = Capturer::read_coordinates_from_file(&left_coordinates)?;
            let right = Capturer::read_coordinates_from_file(&right_coordinates)?;
            let front = Capturer::read_coordinates_from_file(&front_coordinates)?;
            let back = Capturer::read_coordinates_from_file(&back_coordinates)?;

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
