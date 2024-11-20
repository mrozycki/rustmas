use std::f64::consts::PI;

use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::Schema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Twistiness", number(min = "-5.0", max = 5.0, step = 0.02))]
    twistiness: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self { twistiness: 1.0 }
    }
}

#[animation_utils::plugin]
pub struct RainbowSpiral {
    points_polar: Vec<(f64, f64, f64)>,
    time: f64,
    parameters: Parameters,
}

impl Animation for RainbowSpiral {
    type Parameters = Parameters;
    type Wrapped = SpeedControlled<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_polar: points.into_iter().map(animation_utils::to_polar).collect(),
            time: 0.0,
            parameters: Default::default(),
        }
    }

    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        self.points_polar
            .iter()
            .map(|(_, a, h)| {
                lightfx::Color::hsv(
                    (a / PI + self.time + h * self.parameters.twistiness) / 2.0,
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
