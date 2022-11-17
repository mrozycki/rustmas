use super::{utils, Animation};

pub struct RainbowSpiral {
    points_polar: Vec<(f64, f64, f64)>,
}

impl RainbowSpiral {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_polar: points.iter().map(utils::to_polar).collect(),
        }
    }
}

impl Animation for RainbowSpiral {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        self.points_polar
            .iter()
            .map(|(_, a, h)| {
                lightfx::Color::hsv(a / (std::f64::consts::PI * 2.0) + h + time / 2.0, 1.0, 0.5)
            })
            .into()
    }
}
