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

use crate::cv::{self, Display};

type Point2 = (f64, f64);

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WithConfidence<T: Default> {
    pub inner: T,
    pub confidence: f64,
}

impl<T: Default> WithConfidence<T> {
    pub fn confident(&self) -> bool {
        self.confidence > 0.3
    }

    pub fn with_confidence(mut self, new_confidence: f64) -> Self {
        self.confidence = new_confidence;
        self
    }
}

const NOT_FOUND_3D: WithConfidence<Vector3<f64>> = WithConfidence::<Vector3<f64>> {
    confidence: 0.0,
    inner: Vector3::new(-1.0, -1.0, -1.0),
};

pub struct Capturer {
    light_client: Box<dyn LightClient>,
    camera: cv::Camera,
    number_of_lights: usize,
}

fn pause() {
    print!("Press return to continue...");
    io::stdout().flush().expect("Can't flush to stdout");
    let mut stdin = io::stdin();
    stdin.read_exact(&mut [0u8]).expect("Can't read from stdin");
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

    fn preview(&mut self) -> Result<(), Box<dyn Error>> {
        let display = Display::new("Preview")?;
        print!("Press return to continue...");
        io::stdout().flush().expect("Can't flush to stdout");
        let handler = thread::spawn(|| {
            let mut stdin = io::stdin();
            stdin.read_exact(&mut [0u8]).expect("Can't read from stdin");
        });

        loop {
            if handler.is_finished() {
                break;
            }
            display.show(&self.camera.capture()?)?;
            let key = display.wait_for(Duration::from_millis(10))?;
            if key > 0 && key != 255 {
                break;
            }
        }
        thread::sleep(Duration::from_millis(100));
        Ok(())
    }

    pub fn read_coordinates_from_file(
        path: &str,
    ) -> Result<Vec<WithConfidence<Point2>>, Box<dyn Error>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)?;
        Ok(Self::normalize(Self::mark_outliers_by_distance(
            reader.deserialize().filter_map(|a| a.ok()).collect_vec(),
            Self::distance_2d,
        )))
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
            if self.light_client.display_frame(frame).await.is_err() {
                warn!("Failed to update the lights, retrying...");
                attempts += 1;
                thread::sleep(DELAY * attempts);
            } else {
                break;
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
    ) -> Result<Vec<WithConfidence<Point2>>, Box<dyn Error>> {
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
            if self.light_client.display_frame(&frame).await.is_err() {
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

        Ok(Self::normalize(Self::mark_outliers_by_distance(
            coords,
            Self::distance_2d,
        )))
    }

    pub async fn wait_for_perspective(&mut self, prompt: &str) -> Result<(), Box<dyn Error>> {
        info!("Waiting for camera positioning");
        self.light_client
            .display_frame(&self.all_lights_on())
            .await?;
        println!("{}", prompt);
        self.preview()?;
        Ok(())
    }

    fn distance_2d((x1, y1): &(usize, usize), (x2, y2): &(usize, usize)) -> f64 {
        ((*x1 as f64 - *x2 as f64).powi(2) + (*y1 as f64 - *y2 as f64).powi(2)).sqrt()
    }

    pub fn mark_outliers_by_distance<T, F>(
        raw_points: Vec<WithConfidence<T>>,
        distance_fn: F,
    ) -> Vec<WithConfidence<T>>
    where
        T: Default + Clone,
        F: Fn(&T, &T) -> f64,
    {
        let distances = raw_points
            .iter()
            .tuple_windows()
            .filter(|(a, b)| a.confident() && b.confident())
            .map(|(a, b)| distance_fn(&a.inner, &b.inner))
            .collect_vec();
        let avg_distance = distances.iter().sum::<f64>() / distances.len() as f64;
        let stddev_distance =
            statistical::standard_deviation(distances.as_slice(), Some(avg_distance));
        let acceptable_distance = avg_distance + stddev_distance;

        [WithConfidence::default()]
            .into_iter()
            .chain(raw_points)
            .chain([WithConfidence::default()])
            .tuple_windows()
            .map(|(before, current, after)| {
                if before.confident()
                    && distance_fn(&before.inner, &current.inner) > acceptable_distance
                    || after.confident()
                        && distance_fn(&current.inner, &after.inner) > acceptable_distance
                {
                    current.with_confidence(0.1)
                } else {
                    current
                }
            })
            .collect()
    }

    fn normalize(raw_points: Vec<WithConfidence<(usize, usize)>>) -> Vec<WithConfidence<Point2>> {
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
            .map(|a| WithConfidence::<Point2> {
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
        front: WithConfidence<Point2>,
        right: WithConfidence<Point2>,
        back: WithConfidence<Point2>,
        left: WithConfidence<Point2>,
    ) -> WithConfidence<Vector3<f64>> {
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
        .flatten()
        .collect_vec();
        if yns.is_empty() {
            return NOT_FOUND_3D;
        }
        let y = -yns.iter().sum::<f64>() / yns.len() as f64;

        let x = match (x_positive, x_negative) {
            (Some(xp), Some(xn)) => (xp - xn) / 2.0,
            (Some(xp), None) => xp,
            (None, Some(xn)) => -xn,
            (None, None) => return NOT_FOUND_3D,
        };

        let z = match (z_positive, z_negative) {
            (Some(zp), Some(zn)) => (zp - zn) / 2.0,
            (Some(zp), None) => zp,
            (None, Some(zn)) => -zn,
            (None, None) => return NOT_FOUND_3D,
        };

        WithConfidence::<Vector3<f64>> {
            confidence: 1.0,
            inner: Vector3::new(x, y, z),
        }
    }

    pub fn merge_perspectives(
        front: Vec<WithConfidence<Point2>>,
        right: Vec<WithConfidence<Point2>>,
        back: Vec<WithConfidence<Point2>>,
        left: Vec<WithConfidence<Point2>>,
    ) -> Vec<WithConfidence<Vector3<f64>>> {
        front
            .into_iter()
            .zip(back)
            .zip(left.into_iter().zip(right))
            .map(|((front, back), (left, right))| Self::merge_point(front, right, back, left))
            .collect()
    }

    pub fn interpolate_gaps(
        mut points: Vec<WithConfidence<Vector3<f64>>>,
    ) -> Vec<WithConfidence<Vector3<f64>>> {
        let gaps: Vec<(usize, usize)> = points
            .iter()
            .enumerate()
            .group_by(|(_, p)| !p.confident())
            .into_iter()
            .filter(|(key, _)| *key)
            .filter_map(
                |(_, group)| match group.into_iter().map(|(index, _)| index).minmax() {
                    itertools::MinMaxResult::NoElements => None,
                    itertools::MinMaxResult::OneElement(a) => Some((a, a)),
                    itertools::MinMaxResult::MinMax(a, b) => Some((a, b)),
                },
            )
            .collect();

        for (start, end) in gaps {
            if start == 0 {
                continue;
            }
            if let (Some(before), Some(after)) =
                (points.get(start - 1).cloned(), points.get(end + 1).cloned())
            {
                let step = (after.inner - before.inner) / ((end - start + 2) as f64);
                let mut next = before;
                for i in start..=end {
                    next.inner += step;
                    *points.get_mut(i).unwrap() = next.clone();
                }
            }
        }

        points
    }

    fn extrapolate_beginning(
        mut points: Vec<WithConfidence<Vector3<f64>>>,
    ) -> Vec<WithConfidence<Vector3<f64>>> {
        let groups = points.iter().group_by(|x| x.confident());
        let first_two_groups = groups
            .into_iter()
            .take(2)
            .map(|(key, group)| (key, group.collect_vec()))
            .collect_vec();
        if !first_two_groups.get(0).unwrap().0 {
            // first group is not confident
            let extrapolation_vector = first_two_groups
                .get(1)
                .unwrap()
                .1
                .iter()
                .tuple_windows::<(_, _)>()
                .map(|(a, b)| a.inner - b.inner)
                .next()
                .unwrap();

            let first_good_coords = first_two_groups.get(1).unwrap().1.first().unwrap().inner;
            let number_to_extrapolate = first_two_groups.first().unwrap().clone().1.len();

            points
                .iter_mut()
                .enumerate()
                .take(number_to_extrapolate)
                .for_each(|(i, x)| {
                    *x = WithConfidence {
                        inner: first_good_coords
                            + (number_to_extrapolate - i) as f64 * extrapolation_vector,
                        confidence: 0.5,
                    }
                });
        }

        points
    }

    pub fn extrapolate_ends(
        mut points: Vec<WithConfidence<Vector3<f64>>>,
    ) -> Vec<WithConfidence<Vector3<f64>>> {
        points = Self::extrapolate_beginning(points);
        points.reverse();
        points = Self::extrapolate_beginning(points);
        points.reverse();
        points
    }

    pub fn save_3d_coordinates<P: AsRef<Path>>(
        path: P,
        coordinates: &[WithConfidence<Vector3<f64>>],
    ) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::Writer::from_path(path)?;
        coordinates
            .iter()
            .map(|p| {
                if p.confident() {
                    p.inner
                } else {
                    Vector3::new(-1.0, -1.0, -1.0)
                }
            })
            .map(|light| writer.serialize((light[0], light[1], light[2])))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    pub fn load_3d_coordinates<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<Vector3<f64>>, Box<dyn Error>> {
        let points = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)?
            .deserialize()
            .filter_map(|record: Result<(f64, f64, f64), _>| record.ok())
            .map(|(x, y, z)| Vector3::new(x, y, z))
            .collect();

        Ok(points)
    }

    fn save_2d_coordinates<P: AsRef<Path>, T: Default + Serialize>(
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
