use std::f64::consts::PI;

use animation_api::parameter_schema::{get_schema, ParametersSchema};
use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::ParameterSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, ParameterSchema)]
struct Parameters {
    #[schema_field(name = "Twistiness", number(min = "-5.0", max = 5.0, step = 0.02))]
    twistiness: f64,
}

#[animation_utils::plugin]
pub struct RainbowSpiral {
    points_polar: Vec<(f64, f64, f64)>,
    time: f64,
    parameters: Parameters,
}

impl RainbowSpiral {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points_polar: points.into_iter().map(animation_utils::to_polar).collect(),
            time: 0.0,
            parameters: Parameters { twistiness: 1.0 },
        }))
    }
}

impl Animation for RainbowSpiral {
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

    fn animation_name(&self) -> &str {
        "Rainbow Spiral"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        get_schema::<Parameters>()
    }

    fn get_parameters(&self) -> serde_json::Value {
        json!(self.parameters)
    }

    fn set_parameters(
        &mut self,
        parameters: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.parameters = serde_json::from_value(parameters)?;
        Ok(())
    }
}
