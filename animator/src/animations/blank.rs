use super::{Animation, AnimationParameters};

pub struct Blank {
    frame: lightfx::Frame,
}

impl Blank {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        Box::new(Self {
            frame: lightfx::Frame::new_black(points.len()),
        })
    }
}

impl Animation for Blank {
    fn frame(&mut self, _: f64) -> lightfx::Frame {
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
