use crate::Color;

pub struct Gradient {
    colors: Vec<Color>,
}

impl Gradient {
    pub fn new(colors: Vec<Color>) -> Self {
        Self { colors }
    }

    pub fn at(&self, d: f64) -> Color {
        let d = d.clamp(0.0, 1.0) * (self.colors.len() as f64 - 1.0);

        let left = d.floor() as usize;
        let right = d.ceil() as usize;
        let d = d.fract();
        self.colors[left].lerp(&self.colors[right], d)
    }
}
