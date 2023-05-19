use std::f64::consts::PI;

use animation_api::parameter_schema::{get_schema, ParametersSchema};
use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::{EnumSchema, ParameterSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, EnumSchema)]
enum Axis {
    #[schema_variant(name = "X: Left-Right")]
    X,

    #[schema_variant(name = "Y: Bottom-Top")]
    Y,

    #[schema_variant(name = "Z: Front-Back")]
    Z,
}

#[derive(Serialize, Deserialize, ParameterSchema)]
struct Parameters {
    #[schema_field(name = "First color", color)]
    color_a: lightfx::Color,

    #[schema_field(name = "Second color", color)]
    color_b: lightfx::Color,

    #[schema_field(name = "Axis", enum_options)]
    axis: Axis,
}

#[animation_utils::plugin]
pub struct SpinningHalves {
    points: Vec<(f64, f64, f64)>,
    time: f64,
    parameters: Parameters,
}

impl SpinningHalves {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points,
            time: 0.0,
            parameters: Parameters {
                color_a: lightfx::Color::rgb(255, 0, 0),
                color_b: lightfx::Color::rgb(0, 255, 0),
                axis: Axis::Z,
            },
        }))
    }
}

impl Animation for SpinningHalves {
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
                if animation_utils::cycle(y.atan2(x) / PI + self.time, 2.0) < 1.0 {
                    self.parameters.color_a
                } else {
                    self.parameters.color_b
                }
            })
            .into()
    }

    fn animation_name(&self) -> &str {
        "Spinning Halves"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        get_schema::<Parameters>()
    }

    fn set_parameters(
        &mut self,
        parameters: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.parameters = serde_json::from_value(parameters)?;
        Ok(())
    }

    fn get_parameters(&self) -> serde_json::Value {
        json!(self.parameters)
    }
}
