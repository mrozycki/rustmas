use super::Animation;
use rustmas_light_client as client;

pub struct RainbowSphere {
    points_radius: Vec<f64>,
}

impl RainbowSphere {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_radius: points
                .iter()
                .map(|(x, y, z)| (x.powi(2) + y.powi(2) + z.powi(2)).sqrt())
                .collect(),
        }
    }
}

impl Animation for RainbowSphere {
    fn frame(&mut self, time: f64) -> client::Frame {
        self.points_radius
            .iter()
            .map(|r| client::Color::hsv(r - time, 1.0, 1.0))
            .into()
    }
}
