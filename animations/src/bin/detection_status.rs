use animation_api::Animation;
use animation_utils::decorators::BrightnessControlled;

#[animation_utils::plugin]
pub struct DetectionStatus {
    points: Vec<bool>,
}

impl Animation for DetectionStatus {
    type Parameters = ();
    type Wrapped = BrightnessControlled<Self>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            points: points
                .iter()
                .map(|(a, b, c)| {
                    a.to_bits() == (-1.0_f64).to_bits()
                        || b.to_bits() == (-1.0_f64).to_bits()
                        || c.to_bits() == (-1.0_f64).to_bits()
                })
                .collect(),
        }
    }

    fn update(&mut self, _delta: f64) {}

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .map(|x| {
                if *x {
                    lightfx::Color::rgb(255, 0, 0)
                } else {
                    lightfx::Color::rgb(0, 255, 0)
                }
            })
            .into()
    }

    fn get_fps(&self) -> f64 {
        0.0
    }
}
