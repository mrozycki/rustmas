use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::Schema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Density", number(min = 0.5, max = 5.0, step = 0.05))]
    density: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self { density: 1.0 }
    }
}

#[animation_utils::plugin]
pub struct RainbowCable {
    points_count: usize,
    time: f64,
    parameters: Parameters,
}

impl RainbowCable {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points_count: points.len(),
            time: 0.0,
            parameters: Default::default(),
        }))
    }
}

impl Animation for RainbowCable {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        (0..self.points_count)
            .map(|i| {
                lightfx::Color::hsv(
                    i as f64 / self.points_count as f64 * self.parameters.density + self.time,
                    1.0,
                    1.0,
                )
            })
            .into()
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }
}
