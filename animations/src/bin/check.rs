use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use log::debug;

#[animation_utils::plugin]
pub struct Check {
    points_count: usize,
    time: f64,
}

impl Animation for Check {
    type Parameters = ();
    type Wrapped = SpeedControlled<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_count: points.len(),
            time: 0.0,
        }
    }

    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        let index = ((self.time * 8.0) % self.points_count as f64) as usize;
        debug!("Checking light #{}", index);
        lightfx::Frame::new_black(self.points_count).with_pixel(index, lightfx::Color::white())
    }

    fn get_fps(&self) -> f64 {
        8.0
    }
}
