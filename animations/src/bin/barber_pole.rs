use std::f64::consts::PI;

use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::Schema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "First color", color)]
    color_a: lightfx::Color,

    #[schema_field(name = "Second color", color)]
    color_b: lightfx::Color,

    #[schema_field(name = "Transition width", number(min = 0.0, max = 1.0, step = 0.1))]
    transition: f64,

    #[schema_field(name = "Twistiness", number(min = "-5.0", max = 5.0, step = 0.02))]
    twistiness: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            color_a: lightfx::Color::rgb(255, 0, 0),
            color_b: lightfx::Color::rgb(255, 255, 255),
            transition: 0.5,
            twistiness: 1.0,
        }
    }
}

#[animation_utils::plugin]
pub struct BarberPole {
    points_polar: Vec<(f64, f64, f64)>,
    time: f64,
    parameters: Parameters,
}

impl Animation for BarberPole {
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
                let p = (a / PI + self.time + h * self.parameters.twistiness).rem_euclid(2.0);
                if p < self.parameters.transition / 2.0 {
                    self.parameters.color_b.lerp(
                        &self.parameters.color_a,
                        p / (self.parameters.transition / 2.0) + 0.5,
                    )
                } else if p < 1.0 - self.parameters.transition / 2.0 {
                    self.parameters.color_a
                } else if p < 1.0 + self.parameters.transition / 2.0 {
                    self.parameters.color_a.lerp(
                        &self.parameters.color_b,
                        (p - 1.0 + self.parameters.transition / 2.0) / self.parameters.transition,
                    )
                } else if p < 2.0 - self.parameters.transition / 2.0 {
                    self.parameters.color_b
                } else {
                    self.parameters.color_b.lerp(
                        &self.parameters.color_a,
                        (p - 2.0 + self.parameters.transition / 2.0) / self.parameters.transition,
                    )
                }
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
