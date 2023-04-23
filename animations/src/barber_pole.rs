use std::f64::consts::PI;

use animation_api::parameter_schema::{Parameter, ParameterValue, ParametersSchema};
use animation_api::{Animation, AnimationParameters};
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    color_a: lightfx::Color,
    color_b: lightfx::Color,
    twistiness: f64,
}

#[animation_utils::plugin]
pub struct BarberPole {
    points_polar: Vec<(f64, f64, f64)>,
    time: f64,
    parameters: Parameters,
}

impl BarberPole {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points_polar: points.into_iter().map(animation_utils::to_polar).collect(),
            time: 0.0,
            parameters: Parameters {
                color_a: lightfx::Color::rgb(255, 0, 0),
                color_b: lightfx::Color::rgb(255, 255, 255),
                twistiness: 1.0,
            },
        }))
    }
}

impl Animation for BarberPole {
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        self.points_polar
            .iter()
            .map(|(_, a, h)| {
                if animation_utils::cycle(a / PI + self.time + h * self.parameters.twistiness, 2.0)
                    < 1.0
                {
                    self.parameters.color_a
                } else {
                    self.parameters.color_b
                }
            })
            .into()
    }
}

impl AnimationParameters for BarberPole {
    fn animation_name(&self) -> &str {
        "Barber Pole"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![
                Parameter {
                    id: "color_a".to_owned(),
                    name: "First color".to_owned(),
                    description: None,
                    value: ParameterValue::Color,
                },
                Parameter {
                    id: "color_b".to_owned(),
                    name: "Second color".to_owned(),
                    description: None,
                    value: ParameterValue::Color,
                },
                Parameter {
                    id: "twistiness".to_owned(),
                    name: "Twistiness".to_owned(),
                    description: None,
                    value: ParameterValue::Number {
                        min: -5.0,
                        max: 5.0,
                        step: 0.02,
                    },
                },
            ],
        }
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
