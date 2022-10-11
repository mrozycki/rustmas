mod cv;

use std::{
    error::Error,
    fmt::Write,
    io::{self, Read, Write as IoWrite},
    path::Path,
    time::Duration,
};

use clap::Parser;
use cv::{Camera, Display};
use indicatif::{ProgressBar, ProgressFinish, ProgressIterator, ProgressState, ProgressStyle};

use client::LightClient;
use rustmas_light_client as client;

fn opencv_example(camera: &mut Camera) -> Result<(), Box<dyn Error>> {
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
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[arg(short, long, default_value = "lights.csv")]
    output: String,
    #[arg(short, long, default_value_t = 500)]
    lights_count: usize,
    #[arg(short, long)]
    ip_camera: Option<String>,
    #[arg(long, default_value_t = false)]
    opencv_example: bool,
}

fn generate_single_light_frame(index: usize, size: usize) -> client::Frame {
    client::Frame::new_black(size).with_pixel(index, client::Color::white())
}

async fn capture_perspective(
    light_client: &dyn LightClient,
    camera: &mut Camera,
    lights_count: usize,
) -> Result<Vec<(usize, usize)>, Box<dyn Error>> {
    let mut coords = Vec::new();

    let pb = ProgressBar::new(lights_count as u64)
        .with_style(
            ProgressStyle::with_template(
                "{msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
            })
            .progress_chars("#>-"),
        )
        .with_message("Locating lights")
        .with_finish(ProgressFinish::AndLeave);

    for i in (0..lights_count).progress_with(pb) {
        let frame = generate_single_light_frame(i, lights_count);
        if let Err(_) = light_client.display_frame(&frame).await {
            eprintln!("Warning: failed to light up light #{}, skipping", i);
            continue;
        }

        let picture = camera.capture()?;
        coords.push(cv::find_light(&picture)?);
    }

    Ok(coords)
}

fn merge_perspectives(
    front: Vec<(usize, usize)>,
    side: Vec<(usize, usize)>,
) -> Vec<(f64, f64, f64)> {
    vec![(0.0, 1.0, 0.0); front.len()]
}

fn save_positions<P: AsRef<Path>>(
    path: P,
    positions: &Vec<(f64, f64, f64)>,
) -> Result<(), Box<dyn Error>> {
    let mut writer = csv::Writer::from_path(path)?;
    positions
        .iter()
        .map(|light| writer.serialize(light))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();
    let mut camera = if let Some(path) = args.ip_camera {
        Camera::new_from_file(&path)?
    } else {
        Camera::new_default()?
    };
    if args.opencv_example {
        opencv_example(&mut camera)?;
        return Ok(());
    }

    let mut stdin = io::stdin();
    let light_client = client::MockLightClient::new();
    let all_white = client::Frame::new(args.lights_count, client::Color::white());

    light_client.display_frame(&all_white).await?;
    println!("Position camera to capture lights from the front");
    print!("Press return to continue...");
    io::stdout().flush()?;
    stdin.read(&mut [0u8])?;
    let front = capture_perspective(&light_client, &mut camera, args.lights_count).await?;

    light_client.display_frame(&all_white).await?;
    println!("Position camera to capture lights from the right-hand side");
    print!("Press return to continue...");
    io::stdout().flush()?;
    stdin.read(&mut [0u8])?;
    let side = capture_perspective(&light_client, &mut camera, args.lights_count).await?;

    let light_positions = merge_perspectives(front, side);
    save_positions(args.output, &light_positions)?;

    Ok(())
}
