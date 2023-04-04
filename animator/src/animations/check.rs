use animation_api::{Animation, AnimationParameters};
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use log::debug;

pub struct Check {
    points_count: usize,
}

impl Check {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        SpeedControlled::new(BrightnessControlled::new(Box::new(Self {
            points_count: points.len(),
        })))
    }
}

impl Animation for Check {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        let index = ((time * 8.0) % self.points_count as f64) as usize;
        debug!("Checking light #{}", index);
        lightfx::Frame::new_black(self.points_count).with_pixel(index, lightfx::Color::white())
    }
}

impl AnimationParameters for Check {
    fn animation_name(&self) -> &str {
        "Testing: Check Lights"
    }

    fn get_fps(&self) -> f64 {
        8.0
    }
}
