use super::Animation;
use log::debug;
use rustmas_light_client as client;

pub struct Check {
    points_count: usize,
}

impl Check {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_count: points.len(),
        }
    }
}

impl Animation for Check {
    fn frame(&mut self, time: f64) -> client::Frame {
        let index = ((time * 8.0) % self.points_count as f64) as usize;
        debug!("Checking light #{}", index);
        client::Frame::new_black(self.points_count).with_pixel(index, client::Color::white())
    }
}
