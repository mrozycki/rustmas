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

use opencv::{core, highgui, imgproc, prelude::*, videoio, Result};

fn opencv_example() -> Result<(), Box<dyn Error>> {
    opencv::opencv_branch_32! {
        let mut cam = videoio::VideoCapture::new_default(0)?; // 0 is the default camera
    }
    opencv::not_opencv_branch_32! {
        let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // 0 is the default camera
    }
    let opened = videoio::VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to open default camera!");
    }
    let window = "video capture";
    highgui::start_window_thread()?;
    highgui::named_window(window, highgui::WINDOW_GUI_NORMAL)?;
    highgui::set_window_property(window, highgui::WND_PROP_AUTOSIZE, 1.0)?;
    highgui::set_window_property(window, highgui::WND_PROP_TOPMOST, 1.0)?;
    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame)?;

        if frame.size()?.width > 0 {
            let mut hsv = Mat::default();
            imgproc::cvt_color(&frame, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;
            let lower = core::Scalar::from((0.0, 0.0, 255.0));
            let upper = core::Scalar::from((5.0, 128.0, 255.0));
            let mut mask = Mat::default();
            _ = opencv::core::in_range(&hsv, &lower, &upper, &mut mask);
            let mut max_loc = core::Point::default();
            _ =opencv::core::min_max_loc(&mask, None, None, None, Some(&mut max_loc), &mask);
            println!("{:?}", max_loc);
            _ = imgproc::circle(
                &mut frame,
                max_loc,
                20,
                core::VecN::new(0.0, 0.0, 255.0, 255.0),
                2,
                imgproc::LINE_AA,
                0,
            );

            highgui::imshow(window, &frame)?;
        }
        if highgui::wait_key(10)? > 0 {
            break;
        }
    }
    cam.release()?;
    highgui::set_window_property(window, highgui::WND_PROP_TOPMOST, 0.0)?;
    highgui::set_window_property(window, highgui::WND_PROP_VISIBLE, 0.0)?;
    highgui::destroy_all_windows()?;
    Ok(())
}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[arg(short, long, default_value = "lights.csv")]
    output: String,
    #[arg(short, long, default_value_t = 500)]
    lights_count: usize,
    #[arg(long, default_value_t = false)]
    opencv_example: bool,
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
    if args.opencv_example {
        opencv_example()?;
    }

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
