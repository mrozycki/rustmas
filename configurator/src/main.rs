mod capture;
mod cv;

use std::{error::Error, time::Duration};

use capture::Capturer;
use clap::Parser;
use cv::{Camera, Display};
use rustmas_light_client as light_client;

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

    let mut configurator = Capturer::new(
        Box::new(light_client::MockLightClient::new()),
        camera,
        args.lights_count,
    );

    configurator
        .wait_for_perspective("Position camera to capture lights from the front")
        .await?;
    let front = configurator.capture_perspective().await?;

    configurator
        .wait_for_perspective("Position camera to capture lights from the right-hand side")
        .await?;
    let side = configurator.capture_perspective().await?;

    let light_positions = Capturer::merge_perspectives(front, side);
    Capturer::save_positions(args.output, &light_positions)?;

    Ok(())
}
