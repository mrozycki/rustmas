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
use nalgebra::Vector3;
use rustmas_light_client as client;
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithConfidence<T> {
    pub inner: T,
    pub confidence: f64,
}

impl<T> WithConfidence<T> {
    pub fn confident(&self) -> bool {
        return self.confidence > 0.3;
    }
}

pub struct Capturer {
    light_client: Box<dyn LightClient>,
    camera: cv::Camera,
    number_of_lights: usize,
}

fn pause() {
    print!("Press return to continue...");
    io::stdout().flush().expect("Can't flush to stdout");
    let mut stdin = io::stdin();
    stdin.read(&mut [0u8]).expect("Can't read from stdin");
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
    ) -> Result<Vec<WithConfidence<(f64, f64)>>, Box<dyn Error>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)?;
        Ok(Self::normalize(
            reader.deserialize().filter_map(|a| a.ok()).collect_vec(),
        ))
    }

    async fn display_frame_with_retry(&self, frame: &lightfx::Frame) {
        const MAX_ATTEMPTS: u32 = 10;
        const DELAY: Duration = Duration::from_millis(30);

        let mut attempts = 0;

        loop {
            if attempts >= MAX_ATTEMPTS {
                warn!("Please check the lights connection.");
                pause();
                attempts = 0;
            }
            if let Err(_) = self.light_client.display_frame(frame).await {
                warn!("Failed to update the lights, retrying...");
                attempts += 1;
                thread::sleep(DELAY * attempts);
            }
        }
    }

    fn capture_with_retry(&mut self) -> cv::Picture {
        const MAX_ATTEMPTS: u32 = 10;
        const DELAY: Duration = Duration::from_millis(30);

        let mut attempts = 0;

        loop {
            match self.camera.capture() {
                Ok(picture) => break picture,
                Err(_) => {
                    if attempts >= MAX_ATTEMPTS {
                        warn!("Failed to capture an image. Please check the camera.");
                        pause();
                        attempts = 0;
                        continue;
                    }
                    warn!("Failed to capture an image, retrying...");
                    attempts += 1;
                    thread::sleep(DELAY * attempts);
                }
            }
        }
    }

    pub async fn capture_perspective(
        &mut self,
        perspective_name: &str,
        save_pictures: bool,
    ) -> Result<Vec<WithConfidence<(f64, f64)>>, Box<dyn Error>> {
        let mut coords = Vec::new();
        let timestamp = chrono::offset::Local::now().format("%FT%X");
        let dir = format!("captures/{}", timestamp);
        std::fs::create_dir_all(dir.as_str().to_owned() + "/img")?;

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
            self.display_frame_with_retry(&self.all_lights_off()).await;

            thread::sleep(Duration::from_millis(30));
            let base_picture = self.camera.capture()?;

            let frame = self.single_light_on(i);
            if let Err(_) = self.light_client.display_frame(&frame).await {
                warn!("Failed to light up light #{}, skipping", i);
                continue;
            }
            thread::sleep(Duration::from_millis(30));

            let mut led_picture = self.capture_with_retry();

            let found_coords = cv::find_light_from_diff(&base_picture, &led_picture)?;
            if save_pictures {
                if found_coords.confident() {
                    led_picture.mark(found_coords.inner.0, found_coords.inner.1)?;
                }
                let filename = format!("{}/img/{:03}.jpg", dir, i);
                led_picture.save_to_file(filename.as_str())?;
            }
            coords.push(found_coords);
        }

        info!("Preparing output reference image");
        self.display_frame_with_retry(&self.all_lights_on()).await;

        thread::sleep(Duration::from_millis(30));
        let mut all_lights_picture = self.capture_with_retry();
        for point in &coords {
            if point.confident() {
                all_lights_picture.mark(point.inner.0, point.inner.1)?;
            }
        }
        all_lights_picture.save_to_file(format!("{}/reference.jpg", dir).as_str())?;
        Self::save_2d_coordinates(format!("{dir}/{perspective_name}.csv").as_str(), &coords)?;

        Ok(Self::normalize(coords))
    }

    pub async fn wait_for_perspective(&mut self, prompt: &str) -> Result<(), Box<dyn Error>> {
        info!("Waiting for camera positioning");
        self.light_client
            .display_frame(&self.all_lights_on())
            .await?;
        println!("{}", prompt);
        pause();
        Ok(())
    }

    fn normalize(
        raw_points: Vec<WithConfidence<(usize, usize)>>,
    ) -> Vec<WithConfidence<(f64, f64)>> {
        let (xs, ys): (Vec<_>, Vec<_>) = raw_points
            .iter()
            .cloned()
            .filter_map(|x| if x.confident() { Some(x.inner) } else { None })
            .unzip();

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
            .map(|a| WithConfidence::<(f64, f64)> {
                inner: (
                    (a.inner.0 as isize - x_min as isize - x_span as isize / 2) as f64
                        * scaling_factor,
                    (a.inner.1 as isize - y_min as isize - y_span as isize / 2) as f64
                        * scaling_factor,
                ),
                confidence: a.confidence,
            })
            .collect()
    }

    fn merge_point(
        front: WithConfidence<(f64, f64)>,
        right: WithConfidence<(f64, f64)>,
        back: WithConfidence<(f64, f64)>,
        left: WithConfidence<(f64, f64)>,
    ) -> Option<Vector3<f64>> {
        let (x_positive, y_negative_front) = if front.confident() {
            (Some(front.inner.0), Some(front.inner.1))
        } else {
            (None, None)
        };
        let (z_positive, y_negative_right) = if right.confident() {
            (Some(right.inner.0), Some(right.inner.1))
        } else {
            (None, None)
        };
        let (x_negative, y_negative_back) = if back.confident() {
            (Some(back.inner.0), Some(back.inner.1))
        } else {
            (None, None)
        };
        let (z_negative, y_negative_left) = if left.confident() {
            (Some(left.inner.0), Some(left.inner.1))
        } else {
            (None, None)
        };

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

        Some(Vector3::new(x, y, z))
    }

    pub fn merge_perspectives(
        front: Vec<WithConfidence<(f64, f64)>>,
        right: Vec<WithConfidence<(f64, f64)>>,
        back: Vec<WithConfidence<(f64, f64)>>,
        left: Vec<WithConfidence<(f64, f64)>>,
    ) -> Vec<Option<Vector3<f64>>> {
        front
            .into_iter()
            .zip(back.into_iter())
            .zip(left.into_iter().zip(right.into_iter()))
            .map(|((front, back), (left, right))| Self::merge_point(front, right, back, left))
            .collect()
    }

    pub fn interpolate_gaps(mut points: Vec<Option<Vector3<f64>>>) -> Vec<Option<Vector3<f64>>> {
        let gaps = points
            .iter()
            .enumerate()
            .group_by(|(_, p)| p.is_none())
            .into_iter()
            .filter(|(key, _)| *key)
            .filter_map(|(_, group)| match group.minmax() {
                itertools::MinMaxResult::NoElements => None,
                itertools::MinMaxResult::OneElement((a, _)) => Some((a, a)),
                itertools::MinMaxResult::MinMax((a, _), (b, _)) => Some((a, b)),
            })
            .collect::<Vec<_>>();

        for (start, end) in gaps {
            if start == 0 {
                continue;
            }
            if let (Some(Some(before)), Some(Some(after))) =
                (points.get(start - 1).cloned(), points.get(end + 1).cloned())
            {
                let step = (after - before) / ((end - start + 2) as f64);
                let mut next = before;
                for i in start..=end {
                    next += step;
                    *points.get_mut(i).unwrap() = Some(next);
                }
            }
        }

        points
    }

    pub fn save_3d_coordinates<P: AsRef<Path>>(
        path: P,
        coordinates: &Vec<Option<Vector3<f64>>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::Writer::from_path(path)?;
        coordinates
            .iter()
            .map(|p| p.unwrap_or(Vector3::new(-1.0, -1.0, -1.0)))
            .map(|light| writer.serialize((light[0], light[1], light[2])))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    fn save_2d_coordinates<P: AsRef<Path>, T: Serialize>(
        path: P,
        coordinates: &Vec<WithConfidence<(T, T)>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_path(path)?;
        for coords in coordinates {
            writer.serialize::<&WithConfidence<(T, T)>>(coords)?;
        }
        Ok(())
    }
}
