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
pub struct RainbowWaterfall {
    points_height: Vec<f64>,
    time: f64,
    parameters: Parameters,
}

impl Animation for RainbowWaterfall {
    type Parameters = Parameters;
    type Wrapped = SpeedControlled<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            parameters: Default::default(),
            time: 0.0,
            points_height: points
                .into_iter()
                .map(|(_, h, _)| (h + 1.0) / 2.0)
                .collect(),
        }
    }

    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        self.points_height
            .iter()
            .map(|h| lightfx::Color::hsv(h * self.parameters.density + self.time, 1.0, 1.0))
            .into()
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
