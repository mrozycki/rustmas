use super::Animation;

pub struct Blank {
    frame: lightfx::Frame,
}

impl Blank {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            frame: lightfx::Frame::new_black(points.len()),
        }
    }
}

impl Animation for Blank {
    fn frame(&mut self, _: f64) -> lightfx::Frame {
        self.frame.clone()
    }
}
