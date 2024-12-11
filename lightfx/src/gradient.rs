use crate::Color;

pub struct Gradient {
    lut: Vec<Color>,
}

impl Gradient {
    pub fn new(colors: &[Color]) -> Self {
        let lut = (0..=255)
            .map(|d| Self::compute(colors, d as f64 / 255.0))
            .collect();
        Self { lut }
    }

    fn compute(colors: &[Color], d: f64) -> Color {
        let d = d.clamp(0.0, 1.0) * (colors.len() as f64 - 1.0);

        let left = d.floor() as usize;
        let right = d.ceil() as usize;
        let d = d.fract();
        colors[left].lerp(&colors[right], d)
    }

    pub fn at(&self, d: f64) -> Color {
        let d = (d.clamp(0.0, 1.0) * 255.0) as usize;
        self.lut[d]
    }
}

impl<const N: usize> From<[Color; N]> for Gradient {
    fn from(colors: [Color; N]) -> Self {
        Self::new(&colors)
    }
}
