use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::Schema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Density", number(min = 1.0, max = 5.0, step = 1.0))]
    density: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self { density: 1.0 }
    }
}

#[animation_utils::wasm_plugin]
pub struct RainbowCylinder {
    points_alpha: Vec<f64>,
    time: f64,
    parameters: Parameters,
}

impl Animation for RainbowCylinder {
    type Parameters = Parameters;
    type Wrapped = SpeedControlled<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_alpha: points
                .into_iter()
                .map(animation_utils::to_polar)
                .map(|(_, a, _)| a)
                .collect(),
            time: 0.0,
            parameters: Default::default(),
        }
    }

    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        self.points_alpha
            .iter()
            .map(|a| {
                lightfx::Color::hsv(
                    self.time + a / (2.0 * std::f64::consts::PI) * self.parameters.density,
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
