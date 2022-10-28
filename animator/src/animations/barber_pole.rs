use super::{utils, Animation};
use rustmas_light_client as client;

pub struct BarberPole {
    points_polar: Vec<(f64, f64, f64)>,
}

impl BarberPole {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_polar: points.iter().map(utils::to_polar).collect(),
        }
    }
}

impl Animation for BarberPole {
    fn frame(&self, time: f64) -> client::Frame {
        self.points_polar
            .iter()
            .map(|(_, a, h)| {
                if (a / (std::f64::consts::PI * 2.0) + h + time / 2.0) % 0.5 - 0.25 < 0.0 {
                    client::Color::rgb(128, 0, 0)
                } else {
                    client::Color::gray(128)
                }
            })
            .into()
    }
}
