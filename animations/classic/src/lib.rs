use std::f64::consts::{FRAC_PI_2, PI};

use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::{EnumSchema, Schema};
use itertools::Itertools;
use lightfx::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, EnumSchema)]
enum Mode {
    #[schema_variant(name = "Flowing one-by-one")]
    FlowingSingles,

    #[schema_variant(name = "Flowing two-by-two")]
    FlowingPairs,

    #[schema_variant(name = "Static")]
    #[default]
    Static,
}

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Color A (Red)", color)]
    color_red: Color,

    #[schema_field(name = "Color B (Green)", color)]
    color_green: Color,

    #[schema_field(name = "Color C (Yellow)", color)]
    color_yellow: Color,

    #[schema_field(name = "Color D (Blue)", color)]
    color_blue: Color,

    #[schema_field(name = "Mode", enum_options)]
    mode: Mode,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            color_red: lightfx::Color::rgb(255, 0, 0),
            color_green: lightfx::Color::rgb(0, 255, 0),
            color_yellow: lightfx::Color::rgb(255, 160, 0),
            color_blue: lightfx::Color::rgb(0, 0, 255),
            mode: Default::default(),
        }
    }
}

#[animation_utils::wasm_plugin]
pub struct Classic {
    points_count: usize,
    time: f64,
    parameters: Parameters,
}

impl Animation for Classic {
    type Parameters = Parameters;
    type CustomTriggers = ();
    type Wrapped = SpeedControlled<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_count: points.len(),
            time: 0.0,
            parameters: Default::default(),
        }
    }

    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        let base = (0..self.points_count)
            .map(|i| match i % 4 {
                0 => self.parameters.color_red,
                1 => self.parameters.color_green,
                2 => self.parameters.color_yellow,
                _ => self.parameters.color_blue,
            })
            .collect_vec();

        let mask = (0..self.points_count).map(|i| match self.parameters.mode {
            Mode::FlowingSingles => ((self.time / 4.0).fract() * 2.0 * PI
                + (i % 4) as f64 * FRAC_PI_2)
                .sin()
                .clamp(0.0, 1.0),
            Mode::FlowingPairs => ((self.time / 4.0).fract() * 2.0 * PI
                + (i % 2) as f64 * FRAC_PI_2)
                .sin()
                .abs(),
            Mode::Static => 1.0,
        });

        base.into_iter()
            .zip(mask)
            .map(|(color, level)| color.dim(level))
            .collect()
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }
}
