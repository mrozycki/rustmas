use std::f64::consts::PI;

use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::{EnumSchema, Schema};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, EnumSchema)]
pub enum Axis {
    #[schema_variant(name = "X: Left-Right")]
    X,

    #[schema_variant(name = "Y: Bottom-Top")]
    Y,

    #[schema_variant(name = "Z: Front-Back")]
    #[default]
    Z,
}

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "First color", color)]
    color_a: lightfx::Color,

    #[schema_field(name = "Second color", color)]
    color_b: lightfx::Color,

    #[schema_field(name = "Axis", enum_options)]
    axis: Axis,

    #[schema_field(name = "Transition width", number(min = 0.0, max = 1.0, step = 0.1))]
    transition: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            color_a: lightfx::Color::rgb(255, 0, 0),
            color_b: lightfx::Color::rgb(0, 255, 0),
            axis: Default::default(),
            transition: 0.0,
        }
    }
}

#[animation_utils::wasm_plugin]
pub struct SpinningHalves {
    points: Vec<(f64, f64, f64)>,
    time: f64,
    parameters: Parameters,
}

impl SpinningHalves {}

impl Animation for SpinningHalves {
    type Parameters = Parameters;
    type CustomTriggers = ();
    type Wrapped = SpeedControlled<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            points,
            time: 0.0,
            parameters: Default::default(),
        }
    }

    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .map(|(x, y, z)| match self.parameters.axis {
                Axis::X => (*y, *z),
                Axis::Y => (*x, *z),
                Axis::Z => (*x, *y),
            })
            .map(|(x, y)| {
                let (ny, nx) = (PI * self.time).sin_cos();
                let p = nx * x + ny * y;
                self.parameters.color_a.lerp(
                    &self.parameters.color_b,
                    ((p + self.parameters.transition / 2.0) / self.parameters.transition)
                        .clamp(0.0, 1.0),
                )
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
