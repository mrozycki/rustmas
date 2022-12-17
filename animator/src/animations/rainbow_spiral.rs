use std::f64::consts::PI;

use lightfx::parameter_schema::{Parameter, ParameterValue, ParametersSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    brightness_controlled::BrightnessControlled, direction_controlled::DirectionControlled,
    speed_controlled::SpeedControlled, utils, Animation, AnimationParameters,
};

#[derive(Serialize, Deserialize)]
struct Parameters {
    twistiness: f64,
}

pub struct RainbowSpiral {
    points_polar: Vec<(f64, f64, f64)>,
    parameters: Parameters,
}

impl RainbowSpiral {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        DirectionControlled::new(
            "Up",
            "Down",
            SpeedControlled::new(BrightnessControlled::new(Box::new(Self {
                points_polar: points.iter().map(utils::to_polar).collect(),
                parameters: Parameters { twistiness: 1.0 },
            }))),
        )
    }
}

impl Animation for RainbowSpiral {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        self.points_polar
            .iter()
            .map(|(_, a, h)| {
                lightfx::Color::hsv(
                    (a / PI + time + h * self.parameters.twistiness) / 2.0,
                    1.0,
                    1.0,
                )
            })
            .into()
    }
}

impl AnimationParameters for RainbowSpiral {
    fn animation_name(&self) -> &str {
        "Rainbow Spiral"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![Parameter {
                id: "twistiness".to_owned(),
                name: "Twistiness".to_owned(),
                description: None,
                value: ParameterValue::Number {
                    min: -5.0,
                    max: 5.0,
                    step: 0.02,
                },
            }],
        }
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
