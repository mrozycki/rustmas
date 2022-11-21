use super::Animation;

pub struct Rgb {
    points_count: usize,
}

impl Rgb {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_count: points.len(),
        }
    }
}

impl Animation for Rgb {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        (0..self.points_count)
            .into_iter()
            .map(|x| match (x + ((time * 3.0).abs() as usize)) % 3 {
                0 => lightfx::Color::rgb(255, 0, 0),
                1 => lightfx::Color::rgb(0, 255, 0),
                _ => lightfx::Color::rgb(0, 0, 255),
            })
            .into()
    }

    fn get_fps(&self) -> f64 {
        3.0
    }
}
