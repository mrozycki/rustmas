use core::fmt;
use std::{
    error::Error,
    sync::{mpsc, Mutex, MutexGuard},
    time::Duration,
};

use async_trait::async_trait;
use log::{error, info};

#[derive(Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Produces a color with given RGB values. The values range from 0 to 255.
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Produces a color with given RGB values. The values range from 0.0 to 1.0.
    pub fn rgb_unit(r: f64, g: f64, b: f64) -> Self {
        Self::rgb((255.0 * r) as u8, (255.0 * g) as u8, (255.0 * b) as u8)
    }

    /// Produces a color for a given hue, saturation and value.
    ///
    /// Full hue circle extends from 0.0 to 1.0, but values from outside this
    /// range all also accepted and will be mapped onto the hue circle.
    /// For example 0.1, 2.1 and -0.9 correspond to the same hue.
    ///
    /// Saturation and value are expected to be within the 0.0 to 1.0 range.
    /// If they are below 0, they will be truncated to 0, and if they are
    /// above 1, they will be truncated to 1.
    pub fn hsv(hue: f64, saturation: f64, value: f64) -> Self {
        let h = if hue < 0.0 {
            1.0 - hue.fract()
        } else {
            hue.fract()
        };
        let s = saturation.max(0.0).min(1.0);
        let v = value.max(0.0).min(1.0);

        let c = v * s;
        let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
        let m = v - c;
        let (r1, g1, b1) = match (h * 6.0).trunc() as i32 {
            0 => (c, x, 0f64),
            1 => (x, c, 0f64),
            2 => (0f64, c, x),
            3 => (0f64, x, c),
            4 => (x, 0f64, c),
            _ => (c, 0f64, c),
        };

        Self {
            r: ((r1 + m) * 255.0) as u8,
            g: ((g1 + m) * 255.0) as u8,
            b: ((b1 + m) * 255.0) as u8,
        }
    }

    /// Produces a white color of a given temperature.
    ///
    /// Temperature is expected to be provided in kelvin. The algorithm was
    /// devised for values between 1000K and 40,000K. Outside this range the
    /// quality of the result is not guaranteed.
    pub fn kelvin(temp: i32) -> Self {
        let temp = temp as f64 / 100.0;

        let r = if temp <= 66.0 {
            255.0
        } else {
            329.7 * (temp - 60.0).powf(-0.133)
        };

        let g = if temp <= 66.0 {
            99.47 * temp.ln() - 161.1
        } else {
            288.1 * (temp - 60.0).powf(-0.0755)
        };

        let b = if temp < 19.0 {
            0.0
        } else if temp < 66.0 {
            138.5 * (temp - 10.0).ln() - 305.0
        } else {
            255.0
        };

        Self {
            r: r.min(255.0).max(0.0) as u8,
            g: g.min(255.0).max(0.0) as u8,
            b: b.min(255.0).max(0.0) as u8,
        }
    }

    /// Produces a dimmer version of the color. The dimming factor is expected
    /// to be in the range [0.0, 1.0]. Values outside of this range will be
    /// truncated.
    pub fn dim(self, factor: f64) -> Self {
        let factor = factor.max(0.0).min(1.0);
        let dim_component = |c| ((c as f64) * factor) as u8;
        Self {
            r: dim_component(self.r),
            g: dim_component(self.g),
            b: dim_component(self.b),
        }
    }

    /// Blends two colors with the default value of gamma equal to 2.
    pub fn blend(self, other: &Self) -> Self {
        self.blend_with_gamma(other, 2.0)
    }

    /// Blends two colors with the provided value of gamma.
    pub fn blend_with_gamma(self, other: &Self, gamma: f64) -> Self {
        let blend_component = |a, b| {
            let a = (a as f64) / 255.0;
            let b = (b as f64) / 255.0;
            (((a.powf(gamma) + b.powf(gamma)) / 2.0).powf(1.0 / gamma) * 255.0) as u8
        };

        Self {
            r: blend_component(self.r, other.r),
            g: blend_component(self.g, other.g),
            b: blend_component(self.b, other.b),
        }
    }

    /// Produces a gray of the given brightness, where 0 is black and 255 is white.
    pub fn gray(brightness: u8) -> Self {
        Self::rgb(brightness, brightness, brightness)
    }

    pub fn black() -> Self {
        Self::gray(0)
    }

    pub fn white() -> Self {
        Self::gray(255)
    }
}

#[derive(Clone)]
pub struct Frame {
    pixels: Vec<Color>,
}

impl Frame {
    pub fn new(number_of_lights: usize, color: Color) -> Self {
        Self {
            pixels: vec![color; number_of_lights],
        }
    }
    pub fn new_black(number_of_lights: usize) -> Self {
        Self::new(number_of_lights, Color::black())
    }

    pub fn set_pixel(&mut self, index: usize, color: Color) {
        self.pixels[index] = color;
    }

    pub fn with_pixel(mut self, index: usize, color: Color) -> Self {
        self.pixels[index] = color;
        self
    }

    pub fn pixels_iter(&self) -> impl Iterator<Item = &Color> {
        self.pixels.iter()
    }
}

impl<T> From<T> for Frame
where
    T: Iterator<Item = Color>,
{
    fn from(iter: T) -> Self {
        Self {
            pixels: iter.collect(),
        }
    }
}

#[derive(Debug)]
pub enum LightClientError {
    Unlikely,
    ConnectionLost,
}

impl fmt::Display for LightClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for LightClientError {}

#[async_trait]
pub trait LightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError>;
}

pub struct MockLightClient {
    frames: Mutex<Vec<Frame>>,
}

impl MockLightClient {
    pub fn new() -> Self {
        Self {
            frames: Mutex::new(Vec::new()),
        }
    }

    pub fn get_frames(&self) -> MutexGuard<Vec<Frame>> {
        self.frames.lock().unwrap()
    }
}

#[async_trait]
impl LightClient for MockLightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        self.frames.lock().unwrap().push(frame.clone());
        Ok(())
    }
}

#[derive(Clone)]
pub struct RemoteLightClient {
    url: String,
    http_client: reqwest::Client,
}

impl RemoteLightClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            http_client: reqwest::Client::builder()
                .http1_title_case_headers()
                .tcp_keepalive(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl LightClient for RemoteLightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        let pixels = frame
            .pixels_iter()
            .flat_map(|pixel| vec![&pixel.r, &pixel.g, &pixel.b]);
        let request: Vec<_> = "data=".as_bytes().iter().chain(pixels).cloned().collect();

        match self.http_client.post(&self.url).body(request).send().await {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!("{:?}", err);
                Err(LightClientError::ConnectionLost)
            }
        }
    }
}

pub struct VisualiserLightClient {
    _join_handle: tokio::task::JoinHandle<()>,
    light_tx: Mutex<mpsc::Sender<Vec<(f32, f32, f32)>>>,
}

impl VisualiserLightClient {
    pub fn new(points: Vec<(f64, f64, f64)>) -> Result<Self, Box<dyn Error>> {
        let points = points
            .into_iter()
            .map(|(x, y, z)| (x as f32, y as f32, z as f32))
            .collect();

        let (tx, rx) = mpsc::channel();
        Ok(Self {
            _join_handle: tokio::spawn(async move {
                match rustmas_visualiser::visualise(points, rx) {
                    Ok(_) => info!("Visualiser completed without errors"),
                    Err(e) => error!("Visualiser returned an error: {}", e),
                }
            }),
            light_tx: Mutex::new(tx),
        })
    }
}

#[async_trait]
impl LightClient for VisualiserLightClient {
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        let pixels = frame
            .pixels_iter()
            .map(|Color { r, g, b }| (*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0))
            .collect();

        self.light_tx
            .lock()
            .map_err(|_| LightClientError::Unlikely)?
            .send(pixels)
            .map_err(|_| LightClientError::ConnectionLost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_to_rgb() {
        assert_eq!(Color::hsv(0.0, 0.0, 0.0), Color::rgb(0, 0, 0), "black");
        assert_eq!(
            Color::hsv(0.0, 0.0, 1.0),
            Color::rgb(255, 255, 255),
            "white"
        );
        assert_eq!(Color::hsv(0.0, 1.0, 1.0), Color::rgb(255, 0, 0), "red");
        assert_eq!(
            Color::hsv(1.0 / 3.0, 1.0, 1.0),
            Color::rgb(0, 255, 0),
            "green"
        );
        assert_eq!(
            Color::hsv(2.0 / 3.0, 1.0, 1.0),
            Color::rgb(0, 0, 255),
            "blue"
        );
        assert_eq!(
            Color::hsv(1.0 / 6.0, 1.0, 1.0),
            Color::rgb(255, 255, 0),
            "yellow"
        );
        assert_eq!(
            Color::hsv(1.0 / 2.0, 1.0, 1.0),
            Color::rgb(0, 255, 255),
            "cyan"
        );
        assert_eq!(
            Color::hsv(5.0 / 6.0, 1.0, 1.0),
            Color::rgb(255, 0, 255),
            "magenta"
        );
    }

    #[test]
    fn kelvin_to_rgb() {
        assert_eq!(Color::kelvin(1000), Color::rgb(255, 67, 0));
        assert_eq!(Color::kelvin(1500), Color::rgb(255, 108, 0));
        assert_eq!(Color::kelvin(2500), Color::rgb(255, 159, 70));
        assert_eq!(Color::kelvin(5000), Color::rgb(255, 228, 205));
        assert_eq!(Color::kelvin(6600), Color::rgb(255, 255, 255));
        assert_eq!(Color::kelvin(10000), Color::rgb(201, 218, 255));
    }

    #[test]
    fn blend() {
        assert_eq!(
            Color::rgb(255, 0, 0).blend(&Color::rgb(0, 255, 0)),
            Color::rgb(180, 180, 0)
        );
    }
}
