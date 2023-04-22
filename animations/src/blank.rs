use animation_api::{Animation, AnimationParameters};

#[animation_utils::plugin]
pub struct Blank {
    frame: lightfx::Frame,
}

impl Blank {
    pub fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            frame: lightfx::Frame::new_black(points.len()),
        }
    }
}

impl Animation for Blank {
    fn update(&mut self, _delta: f64) {}

    fn render(&self) -> lightfx::Frame {
        self.frame.clone()
    }
}

impl AnimationParameters for Blank {
    fn animation_name(&self) -> &str {
        "Off"
    }

    fn get_fps(&self) -> f64 {
        0.0
    }
}
