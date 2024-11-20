use std::f64::consts::PI;

use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, OffSwitch};
use animation_utils::Schema;
use lightfx::Color;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Boundary left", number(min = "-1.0", max = 1.0, step = 0.05))]
    boundary_left: f64,

    #[schema_field(name = "Boundary right", number(min = "-1.0", max = 1.0, step = 0.05))]
    boundary_right: f64,

    #[schema_field(name = "Border width", number(min = 0.0, max = 0.5, step = 0.01))]
    border_width: f64,

    #[schema_field(name = "Color A", color)]
    color_a: Color,

    #[schema_field(name = "Color B", color)]
    color_b: Color,

    #[schema_field(name = "Color C", color)]
    color_c: Color,

    #[schema_field(name = "Color D", color)]
    color_d: Color,

    #[schema_field(name = "Base dim", number(min = 0.0, max = 1.0, step = 0.05))]
    base_dim: f64,

    #[schema_field(name = "BPM", number(min = 20.0, max = 180.0, step = 0.1))]
    bpm: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            boundary_left: -1.0,
            boundary_right: 1.0,
            border_width: 0.05,
            color_a: Color::rgb(255, 0, 0),
            color_b: Color::rgb(0, 255, 0),
            color_c: Color::rgb(255, 255, 0),
            color_d: Color::rgb(0, 0, 255),
            base_dim: 0.5,
            bpm: 60.0,
        }
    }
}

#[animation_utils::plugin]
pub struct CircleBoom {
    points: Vec<(f64, f64, f64)>,
    beat: f64,
    parameters: Parameters,
}

impl Animation for CircleBoom {
    type Parameters = Parameters;
    type Wrapped = OffSwitch<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            points,
            beat: 0.0,
            parameters: Default::default(),
        }
    }

    fn update(&mut self, delta: f64) {
        self.beat += delta * self.parameters.bpm / 60.0;
    }

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .map(|(x, _y, _z)| {
                (x - self.parameters.boundary_left)
                    / (self.parameters.boundary_right - self.parameters.boundary_left)
            })
            .map(|x| {
                let c = match (x * 4.0).floor() as i32 {
                    0 => self.parameters.color_a,
                    1 => self.parameters.color_b,
                    2 => self.parameters.color_c,
                    3 => self.parameters.color_d,
                    _ => Color::black(),
                };

                let band_pos = (x * 4.0).fract();
                if band_pos < self.parameters.border_width
                    || band_pos > 1.0 - self.parameters.border_width
                {
                    Color::black()
                } else if self.beat.floor() % 4.0 == (x * 4.0).floor() {
                    c.dim(
                        (self.beat.fract() * PI).sin() * (1.0 - self.parameters.base_dim)
                            + self.parameters.base_dim,
                    )
                } else {
                    c.dim(self.parameters.base_dim)
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
