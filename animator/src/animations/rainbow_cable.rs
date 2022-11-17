use super::Animation;

pub struct RainbowCable {
    points_count: usize,
}

impl RainbowCable {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_count: points.len(),
        }
    }
}

impl Animation for RainbowCable {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        (0..self.points_count)
            .into_iter()
            .map(|i| {
                lightfx::Color::hsv(i as f64 / self.points_count as f64 * 4.0 + time, 1.0, 0.5)
            })
            .into()
    }
}
