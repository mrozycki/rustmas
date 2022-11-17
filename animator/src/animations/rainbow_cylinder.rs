use super::utils;
use super::Animation;

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
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        self.points_alpha
            .iter()
            .map(|a| lightfx::Color::hsv(time + a / (2.0 * std::f64::consts::PI), 1.0, 1.0))
            .into()
    }
}
