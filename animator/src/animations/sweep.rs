use super::Animation;
use rustmas_light_client as client;

pub struct Sweep {
    points_height: Vec<f64>,
}

impl Sweep {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_height: points.iter().map(|(_, h, _)| *h).collect(),
        }
    }
}

impl Animation for Sweep {
    fn frame(&mut self, time: f64) -> client::Frame {
        self.points_height
            .iter()
            .map(|h| {
                if *h > time && *h < time + 0.2 {
                    client::Color::white()
                } else {
                    client::Color::black()
                }
            })
            .into()
    }
}
