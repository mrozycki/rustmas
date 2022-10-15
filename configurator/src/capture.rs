use std::{
    error::Error,
    fmt::Write,
    io::{self, Read, Write as IoWrite},
    path::Path,
    time::Duration,
};

use client::LightClient;
use indicatif::{ProgressBar, ProgressFinish, ProgressIterator, ProgressState, ProgressStyle};
use log::{info, warn};
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

    fn all_lights_off(&self) -> client::Frame {
        client::Frame::new(self.number_of_lights, client::Color::black())
    }

    fn all_lights_on(&self) -> client::Frame {
        client::Frame::new(self.number_of_lights, client::Color::white())
    }

    fn single_light_on(&self, index: usize) -> client::Frame {
        client::Frame::new_black(self.number_of_lights).with_pixel(index, client::Color::white())
    }

    pub async fn capture_perspective(
        &mut self,
    ) -> Result<Vec<Option<(usize, usize)>>, Box<dyn Error>> {
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

        info!("Locating lights");
        for i in (0..self.number_of_lights).progress_with(pb) {
            let all_lights_off = self.all_lights_off();
            if let Err(_) = self.light_client.display_frame(&all_lights_off).await {
                warn!("Failed to clear the lights, stopping");
                break;
            }
            let base_picture = self.camera.capture()?;

            let frame = self.single_light_on(i);
            if let Err(_) = self.light_client.display_frame(&frame).await {
                warn!("Failed to light up light #{}, skipping", i);
                continue;
            }
            let led_picture = self.camera.capture()?;

            coords.push(cv::find_light_from_diff(&base_picture, &led_picture)?);
        }

        info!("Preparing output reference image");
        self.light_client
            .display_frame(&self.all_lights_on())
            .await?;
        let window = cv::Display::new("results")?;
        let mut base_picture = self.camera.capture()?;
        for point in &coords {
            if point.is_some() {
                base_picture.mark(point.unwrap().0, point.unwrap().1)?;
            }
        }
        window.show(&base_picture)?;
        window.wait_for(Duration::from_millis(10))?; // apparently needed to show the frame

        Ok(coords)
    }

    pub async fn wait_for_perspective(&mut self, prompt: &str) -> Result<(), Box<dyn Error>> {
        info!("Waiting for camera positioning");
        let mut stdin = io::stdin();
        self.light_client
            .display_frame(&self.all_lights_on())
            .await?;
        println!("{}", prompt);
        print!("Press return to continue...");
        io::stdout().flush()?;
        stdin.read(&mut [0u8])?;
        Ok(())
    }

    pub fn merge_perspectives(
        front: Vec<Option<(usize, usize)>>,
        side: Vec<Option<(usize, usize)>>,
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

    pub async fn opencv_example(&mut self) -> Result<(), Box<dyn Error>> {
        let window = cv::Display::new("video capture")?;
        for i in 0..self.number_of_lights {
            self.wait_for_perspective(format!("Press enter to configure led #{}", i + 1).as_str())
                .await?;
            self.light_client
                .display_frame(&self.all_lights_off())
                .await?;
            std::thread::sleep(Duration::from_millis(200));
            let base_picture = self.camera.capture()?;
            self.light_client
                .display_frame(&self.single_light_on(i))
                .await?;
            std::thread::sleep(Duration::from_millis(200));
            let mut led_picture = self.camera.capture()?;
            let maybe_coords = cv::find_light_from_diff(&base_picture, &led_picture)?;
            if maybe_coords.is_some() {
                led_picture.mark(maybe_coords.unwrap().0, maybe_coords.unwrap().1)?;
            }
            window.show(&led_picture)?;
            window.wait_for(Duration::from_millis(10))?; // apparently needed to show the frame
        }
        Ok(())
    }
}
