use std::{
    error::Error,
    fmt::Write,
    io::{self, Read, Write as IoWrite},
    path::Path,
};

use client::LightClient;
use indicatif::{ProgressBar, ProgressFinish, ProgressIterator, ProgressState, ProgressStyle};
use rustmas_light_client as client;

use crate::cv;

pub struct Capturer {
    light_client: Box<dyn LightClient>,
    camera: cv::Camera,
    number_of_lights: usize,
}

impl Capturer {
    pub fn new(
        light_client: Box<dyn LightClient>,
        camera: cv::Camera,
        number_of_lights: usize,
    ) -> Self {
        Self {
            light_client,
            camera,
            number_of_lights,
        }
    }

    fn generate_all_white_frame(&self) -> client::Frame {
        client::Frame::new(self.number_of_lights, client::Color::white())
    }

    fn generate_single_light_frame(&self, index: usize) -> client::Frame {
        client::Frame::new_black(self.number_of_lights).with_pixel(index, client::Color::white())
    }

    pub async fn capture_perspective(&mut self) -> Result<Vec<(usize, usize)>, Box<dyn Error>> {
        let mut coords = Vec::new();

        let pb = ProgressBar::new(self.number_of_lights as u64)
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

        for i in (0..self.number_of_lights).progress_with(pb) {
            let frame = self.generate_single_light_frame(i);
            if let Err(_) = self.light_client.display_frame(&frame).await {
                eprintln!("Warning: failed to light up light #{}, skipping", i);
                continue;
            }

            let picture = self.camera.capture()?;
            coords.push(cv::find_light(&picture)?);
        }

        Ok(coords)
    }

    pub async fn wait_for_perspective(&mut self, prompt: &str) -> Result<(), Box<dyn Error>> {
        let mut stdin = io::stdin();
        self.light_client
            .display_frame(&self.generate_all_white_frame())
            .await?;
        println!("{}", prompt);
        print!("Press return to continue...");
        io::stdout().flush()?;
        stdin.read(&mut [0u8])?;
        Ok(())
    }

    pub fn merge_perspectives(
        front: Vec<(usize, usize)>,
        side: Vec<(usize, usize)>,
    ) -> Vec<(f64, f64, f64)> {
        vec![(0.0, 1.0, 0.0); front.len()]
    }

    pub fn save_positions<P: AsRef<Path>>(
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
}
