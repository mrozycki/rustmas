use super::utils;
use super::Animation;
use rustmas_light_client as client;

pub struct RainbowCylinder {
    points_alpha: Vec<f64>,
}

impl RainbowCylinder {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_alpha: points
                .iter()
                .map(utils::to_polar)
                .map(|(_, a, _)| a)
                .collect(),
        }
    }
}

impl Animation for RainbowCylinder {
    fn frame(&self, time: f64) -> client::Frame {
        self.points_alpha
            .iter()
            .map(|a| client::Color::hsv(time + a / (2.0 * std::f64::consts::PI), 1.0, 1.0))
            .into()
    }
}
