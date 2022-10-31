use super::Animation;
use rustmas_light_client as client;

pub struct Blank {
    frame: client::Frame,
}

impl Blank {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            frame: client::Frame::new_black(points.len()),
        }
    }
}

impl Animation for Blank {
    fn frame(&mut self, _: f64) -> client::Frame {
        self.frame.clone()
    }
}
