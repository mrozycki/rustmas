use std::f64::consts::PI;

use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::Schema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Color A", color)]
    color_a: lightfx::Color,

    #[schema_field(name = "Color B", color)]
    color_b: lightfx::Color,

    #[schema_field(name = "Density", number(min = 0.1, max = 8.0, step = 0.1))]
    density: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            density: 1.0,
            color_a: lightfx::Color::white(),
            color_b: lightfx::Color::rgb(0, 0, 255),
        }
    }
}

#[animation_utils::plugin]
pub struct RainbowWaterfall {
    points_height: Vec<f64>,
    time: f64,
    parameters: Parameters,
}

impl RainbowWaterfall {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            parameters: Default::default(),
            time: 0.0,
            points_height: points.into_iter().map(|(_, h, _)| h).collect(),
        }))
    }
}

impl Animation for RainbowWaterfall {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        self.points_height
            .iter()
            .map(|h| {
                (PI * (h * self.parameters.density / 2.0 + self.time))
                    .sin()
                    .powi(2)
            })
            .map(|p| {
                self.parameters
                    .color_a
                    .with_alpha(p)
                    .blend(&self.parameters.color_b.with_alpha(1.0))
                    .apply_alpha()
            })
            .into()
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
