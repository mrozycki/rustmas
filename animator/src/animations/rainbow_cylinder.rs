use lightfx::parameter_schema::Parameter;
use lightfx::parameter_schema::ParameterValue;
use lightfx::parameter_schema::ParametersSchema;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

use super::brightness_controlled::BrightnessControlled;
use super::direction_controlled::DirectionControlled;
use super::speed_controlled::SpeedControlled;
use super::utils;
use super::Animation;
use super::AnimationParameters;

#[derive(Serialize, Deserialize)]
struct Parameters {
    density: f64,
}

pub struct RainbowCylinder {
    points_alpha: Vec<f64>,
    parameters: Parameters,
}

impl RainbowCylinder {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        DirectionControlled::new(
            "Clockwise",
            "Counterclockwise",
            SpeedControlled::new(BrightnessControlled::new(Box::new(Self {
                points_alpha: points
                    .iter()
                    .map(utils::to_polar)
                    .map(|(_, a, _)| a)
                    .collect(),
                parameters: Parameters { density: 1.0 },
            }))),
        )
    }
}

impl Animation for RainbowCylinder {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        self.points_alpha
            .iter()
            .map(|a| {
                lightfx::Color::hsv(
                    time + a / (2.0 * std::f64::consts::PI) * self.parameters.density,
                    1.0,
                    1.0,
                )
            })
            .into()
    }
}

impl AnimationParameters for RainbowCylinder {
    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![Parameter {
                id: "density".to_owned(),
                name: "Density".to_owned(),
                description: None,
                value: ParameterValue::Number {
                    min: Some(0.5),
                    max: Some(5.0),
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
