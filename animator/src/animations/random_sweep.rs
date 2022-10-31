use super::{utils, Animation};
use nalgebra::Vector3;
use rustmas_light_client as client;

pub struct RandomSweep {
    points: Vec<Vector3<f64>>,
    heights: Vec<f64>,
    color: client::Color,
    last_time: f64,
}

impl RandomSweep {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points: points
                .iter()
                .map(|(x, y, z)| Vector3::new(*x, *y, *z))
                .collect(),
            heights: Vec::new(),
            color: client::Color::black(),
            last_time: 1.0,
        }
    }

    fn generate_new_sweep(&mut self) {
        let rotation = utils::random_rotation();
        self.heights = self
            .points
            .iter()
            .map(|p| rotation * p)
            .map(|p| p.y)
            .collect();
        self.color = utils::random_hue(1.0, 0.5);
    }
}

impl Animation for RandomSweep {
    fn frame(&mut self, time: f64) -> client::Frame {
        let time = time % 2.0 - 1.0;
        if self.last_time > 0.0 && time < 0.0 {
            self.generate_new_sweep();
        }

        self.last_time = time;
        self.heights
            .iter()
            .map(|h| {
                if *h > time && *h < time + 0.5 {
                    self.color.dim((h - time) / 2.0)
                } else {
                    client::Color::black()
                }
            })
            .into()
    }
}
