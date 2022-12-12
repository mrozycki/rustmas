use std::f64::consts::PI;

use super::{
    brightness_controlled::BrightnessControlled, direction_controlled::DirectionControlled,
    speed_controlled::SpeedControlled, utils, Animation, AnimationParameters,
};
use lightfx::schema::{Parameter, ParameterValue, ParametersSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    color_a: lightfx::Color,
    color_b: lightfx::Color,
    density: f64,
}

pub struct BarberPole {
    points_polar: Vec<(f64, f64, f64)>,
    parameters: Parameters,
}

impl BarberPole {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        DirectionControlled::new(
            "Up",
            "Down",
            SpeedControlled::new(BrightnessControlled::new(Box::new(Self {
                points_polar: points.iter().map(utils::to_polar).collect(),
                parameters: Parameters {
                    color_a: lightfx::Color::rgb(255, 0, 0),
                    color_b: lightfx::Color::rgb(255, 255, 255),
                    density: 1.0,
                },
            }))),
        )
    }
}

impl Animation for BarberPole {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        self.points_polar
            .iter()
            .map(|(_, a, h)| {
                if (a / PI + time + h * self.parameters.density) % 2.0 < 1.0 {
                    self.parameters.color_a
                } else {
                    self.parameters.color_b
                }
            })
            .into()
    }
}

impl AnimationParameters for BarberPole {
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
                    id: "density".to_owned(),
                    name: "Density".to_owned(),
                    description: None,
                    value: ParameterValue::Number {
                        min: Some(0.5),
                        max: Some(5.0),
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
