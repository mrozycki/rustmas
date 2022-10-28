use super::Animation;
use rustmas_light_client as client;

pub struct RainbowCable {
    points_count: usize,
}

impl RainbowCable {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_count: points.len(),
        }
    }
}

impl Animation for RainbowCable {
    fn frame(&self, time: f64) -> client::Frame {
        (0..self.points_count)
            .into_iter()
            .map(|i| client::Color::hsv(i as f64 / 100.0 + time, 1.0, 0.5))
            .into()
    }
}
