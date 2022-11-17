use std::{
    error::Error,
    fmt::Write,
    io::{self, Read, Write as IoWrite},
    path::Path,
    thread,
    time::Duration,
};

use client::LightClient;
use indicatif::{ProgressBar, ProgressFinish, ProgressIterator, ProgressState, ProgressStyle};
use itertools::Itertools;
use log::{info, warn};
use rustmas_light_client as client;

use crate::cv;

trait UnzipOption<T, U> {
    fn unzip_option(self) -> (Option<T>, Option<U>);
}

impl<T, U> UnzipOption<T, U> for Option<(T, U)> {
    fn unzip_option(self) -> (Option<T>, Option<U>) {
        match self {
            Some((a, b)) => (Some(a), Some(b)),
            None => (None, None),
        }
    }
}

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

    fn all_lights_off(&self) -> lightfx::Frame {
        lightfx::Frame::new(self.number_of_lights, lightfx::Color::black())
    }

    fn all_lights_on(&self) -> lightfx::Frame {
        lightfx::Frame::new(self.number_of_lights, lightfx::Color::gray(50))
    }

    fn single_light_on(&self, index: usize) -> lightfx::Frame {
        lightfx::Frame::new_black(self.number_of_lights).with_pixel(index, lightfx::Color::white())
    }

    pub fn read_coordinates_from_file(
        path: &str,
    ) -> Result<Vec<Option<(f64, f64)>>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_path(path)?;
        let coords = reader
            .records()
            .map(|r| -> Result<Option<(f64, f64)>, Box<dyn Error>> {
                let r = r?;
                Ok(match (&r[0], &r[1]) {
                    ("", "") => None,
                    (x_str, y_str) => Some((x_str.parse::<f64>()?, y_str.parse::<f64>()?)),
                })
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

        Ok(coords)
    }

    pub async fn capture_perspective(
        &mut self,
        perspective_name: &str,
        save_pictures: bool,
    ) -> Result<Vec<Option<(f64, f64)>>, Box<dyn Error>> {
        let mut coords = Vec::new();
        let timestamp = chrono::offset::Local::now().format("%FT%X");
        let dir = format!("captures/{}", timestamp);
        std::fs::create_dir_all(dir.as_str())?;

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
            thread::sleep(Duration::from_millis(30));
            let base_picture = self.camera.capture()?;

            let frame = self.single_light_on(i);
            if let Err(_) = self.light_client.display_frame(&frame).await {
                warn!("Failed to light up light #{}, skipping", i);
                continue;
            }
            thread::sleep(Duration::from_millis(30));
            let mut led_picture = self.camera.capture()?;
            let found_coords = cv::find_light_from_diff(&base_picture, &led_picture)?;
            if save_pictures {
                if let Some((x, y)) = found_coords {
                    led_picture.mark(x, y)?;
                }
                let filename = format!("{}/{:03}.jpg", dir, i);
                led_picture.save_to_file(filename.as_str())?;
            }
            coords.push(found_coords);
        }

        info!("Preparing output reference image");
        self.light_client
            .display_frame(&self.all_lights_on())
            .await?;
        thread::sleep(Duration::from_millis(30));
        let mut all_lights_picture = self.camera.capture()?;
        for point in &coords {
            if point.is_some() {
                all_lights_picture.mark(point.unwrap().0, point.unwrap().1)?;
            }
        }
        all_lights_picture.save_to_file(format!("{}/reference.jpg", dir).as_str())?;

        let coords = Self::normalize(coords);
        Self::save_2d_coordinates(format!("{dir}/{perspective_name}.csv").as_str(), &coords)?;
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

    fn normalize(raw_points: Vec<Option<(usize, usize)>>) -> Vec<Option<(f64, f64)>> {
        let (xs, ys): (Vec<_>, Vec<_>) = raw_points.iter().cloned().filter_map(|x| x).unzip();

        let (x_min, x_span) = match xs.iter().minmax() {
            itertools::MinMaxResult::MinMax(min, max) => (*min, *max - *min),
            _ => return Vec::new(),
        };
        let (y_min, y_span) = match ys.iter().minmax() {
            itertools::MinMaxResult::MinMax(min, max) => (*min, *max - *min),
            _ => return Vec::new(),
        };

        let scaling_factor = 2.0 / Ord::max(x_span, y_span) as f64;

        raw_points
            .into_iter()
            .map(|a| {
                a.map(|(x, y)| {
                    (
                        (x as isize - x_min as isize - x_span as isize / 2) as f64 * scaling_factor,
                        (y as isize - y_min as isize - y_span as isize / 2) as f64 * scaling_factor,
                    )
                })
            })
            .collect()
    }

    fn merge_point(
        front: Option<(f64, f64)>,
        right: Option<(f64, f64)>,
        back: Option<(f64, f64)>,
        left: Option<(f64, f64)>,
    ) -> Option<(f64, f64, f64)> {
        let (x_positive, y_negative_front) = front.unzip_option();
        let (z_positive, y_negative_right) = right.unzip_option();
        let (x_negative, y_negative_back) = back.unzip_option();
        let (z_negative, y_negative_left) = left.unzip_option();

        let yns = vec![
            y_negative_front,
            y_negative_right,
            y_negative_back,
            y_negative_left,
        ]
        .into_iter()
        .filter_map(|a| a)
        .collect_vec();
        if yns.len() == 0 {
            return None;
        }
        let y = -yns.iter().sum::<f64>() / yns.len() as f64;

        let x = match (x_positive, x_negative) {
            (Some(xp), Some(xn)) => (xp - xn) / 2.0,
            (Some(xp), None) => xp,
            (None, Some(xn)) => -xn,
            (None, None) => return None,
        };

        let z = match (z_positive, z_negative) {
            (Some(zp), Some(zn)) => (zp - zn) / 2.0,
            (Some(zp), None) => zp,
            (None, Some(zn)) => -zn,
            (None, None) => return None,
        };

        Some((x, y, z))
    }

    pub fn merge_perspectives(
        front: Vec<Option<(f64, f64)>>,
        right: Vec<Option<(f64, f64)>>,
        back: Vec<Option<(f64, f64)>>,
        left: Vec<Option<(f64, f64)>>,
    ) -> Vec<Option<(f64, f64, f64)>> {
        front
            .into_iter()
            .zip(back.into_iter())
            .zip(left.into_iter().zip(right.into_iter()))
            .map(|((front, back), (left, right))| Self::merge_point(front, right, back, left))
            .collect()
    }

    pub fn save_3d_coordinates<P: AsRef<Path>>(
        path: P,
        coordinates: &Vec<Option<(f64, f64, f64)>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::Writer::from_path(path)?;
        coordinates
            .iter()
            .map(|p| p.unwrap_or((-1.0, -1.0, -1.0)))
            .map(|light| writer.serialize(light))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    fn save_2d_coordinates<P: AsRef<Path>>(
        path: P,
        coordinates: &Vec<Option<(f64, f64)>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::Writer::from_path(path)?;
        coordinates
            .iter()
            .cloned()
            .map(UnzipOption::unzip_option)
            .map(|light| writer.serialize(light))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }
}
