use animation_api::Animation;

#[animation_utils::plugin]
pub struct Blank {
    frame: lightfx::Frame,
}

impl Blank {
    pub fn create(points: Vec<(f64, f64, f64)>) -> Self {
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

    fn animation_name(&self) -> &str {
        "Off"
    }

    fn get_fps(&self) -> f64 {
        0.0
    }
}
