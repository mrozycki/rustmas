use super::Animation;
use rustmas_light_client as client;

pub struct Rgb {
    points_count: usize,
}

impl Rgb {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_count: points.len(),
        }
    }
}

impl Animation for Rgb {
    fn frame(&self, time: f64) -> client::Frame {
        (0..self.points_count)
            .into_iter()
            .map(|x| match (x + ((time * 3.0).abs() as usize)) % 3 {
                0 => client::Color::rgb(255, 0, 0),
                1 => client::Color::rgb(0, 255, 0),
                _ => client::Color::rgb(0, 0, 255),
            })
            .into()
    }
}
