use super::Animation;
use rustmas_light_client as client;

pub struct RainbowWaterfall {
    points_height: Vec<f64>,
}

impl RainbowWaterfall {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_height: points.iter().map(|(_, h, _)| (h + 1.0) / 2.0).collect(),
        }
    }
}

impl Animation for RainbowWaterfall {
    fn frame(&self, time: f64) -> client::Frame {
        self.points_height
            .iter()
            .map(|h| client::Color::hsv(h + time, 1.0, 0.5))
            .into()
    }
}
