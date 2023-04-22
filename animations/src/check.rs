use animation_api::{Animation, AnimationParameters};
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use log::debug;

#[animation_utils::plugin]
pub struct Check {
    points_count: usize,
    time: f64,
}

impl Check {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points_count: points.len(),
            time: 0.0,
        }))
    }
}

impl Animation for Check {
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        let index = ((self.time * 8.0) % self.points_count as f64) as usize;
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
