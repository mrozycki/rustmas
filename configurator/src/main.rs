use std::{
    error::Error,
    fmt::Write,
    io::{self, Read, Write as IoWrite},
    path::Path,
    thread,
    time::Duration,
};

use clap::Parser;
use indicatif::{ProgressBar, ProgressFinish, ProgressIterator, ProgressState, ProgressStyle};

use client::LightClient;
use rustmas_light_client as client;

type Picture = Vec<u8>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[arg(short, long, default_value = "lights.csv")]
    output: String,
    #[arg(short, long, default_value_t = 500)]
    lights_count: usize,
}

fn generate_single_light_frame(index: usize, size: usize) -> client::Frame {
    client::Frame::new_black(size).with_pixel(index, client::Color::white())
}

fn take_picture() -> Picture {
    thread::sleep(Duration::from_millis(1));
    Vec::new()
}

fn find_light(_: Picture) -> (usize, usize) {
    thread::sleep(Duration::from_millis(1));
    (21, 37)
}

async fn capture_perspective(
    light_client: &dyn LightClient,
    lights_count: usize,
) -> Vec<(usize, usize)> {
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

        let picture = take_picture();
        coords.push(find_light(picture));
    }

    coords
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

    let all_white = client::Frame::new(args.lights_count, client::Color::white());

    let mut stdin = io::stdin();
    let light_client = client::MockLightClient::new();

    light_client.display_frame(&all_white).await?;
    println!("Position camera to capture lights from the front");
    print!("Press return to continue...");
    io::stdout().flush()?;
    stdin.read(&mut [0u8])?;
    let front = capture_perspective(&light_client, args.lights_count).await;

    light_client.display_frame(&all_white).await?;
    println!("Position camera to capture lights from the right-hand side");
    print!("Press return to continue...");
    io::stdout().flush()?;
    stdin.read(&mut [0u8])?;
    let side = capture_perspective(&light_client, args.lights_count).await;

    let light_positions = merge_perspectives(front, side);
    save_positions(args.output, &light_positions)?;

    Ok(())
}
