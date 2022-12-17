use super::{
    brightness_controlled::BrightnessControlled, direction_controlled::DirectionControlled,
    speed_controlled::SpeedControlled, Animation, AnimationParameters,
};
use lightfx::schema::{Parameter, ParameterValue, ParametersSchema};
use nalgebra::{Rotation3, Vector3};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    color_wrap: lightfx::Color,
    color_ribbon: lightfx::Color,
    x: f64,
    y: f64,
    z: f64,
    width: f64,
}

pub struct Present {
    points: Vec<Vector3<f64>>,
    parameters: Parameters,
}

impl Present {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        DirectionControlled::new(
            "Clockwise",
            "Counterclockwise",
            SpeedControlled::new(BrightnessControlled::new(Box::new(Self {
                points: points
                    .iter()
                    .map(|(x, y, z)| Vector3::new(*x, *y, *z))
                    .collect(),
                parameters: Parameters {
                    color_wrap: lightfx::Color::rgb(255, 255, 255),
                    color_ribbon: lightfx::Color::rgb(255, 0, 0),
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    width: 0.1,
                },
            }))),
        )
    }
}

impl Animation for Present {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        let rotation = Rotation3::new(Vector3::y() * 2.0 * std::f64::consts::PI * time);

        self.points
            .iter()
            .map(|p| rotation * p)
            .map(|p| {
                let dist_x = (p.x - &self.parameters.x).abs();
                let dist_y = (p.y - &self.parameters.y).abs();
                let dist_z = (p.z - &self.parameters.z).abs();

                if dist_x < self.parameters.width / 2.0
                    || dist_y < self.parameters.width / 2.0
                    || dist_z < self.parameters.width / 2.0
                {
                    self.parameters.color_ribbon
                } else {
                    self.parameters.color_wrap
                }
            })
            .into()
    }
}

impl AnimationParameters for Present {
    fn animation_name(&self) -> &str {
        "Present"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![
                Parameter {
                    id: "color_wrap".to_owned(),
                    name: "Wrap color".to_owned(),
                    description: None,
                    value: ParameterValue::Color,
                },
                Parameter {
                    id: "color_ribbon".to_owned(),
                    name: "Ribbon color".to_owned(),
                    description: None,
                    value: ParameterValue::Color,
                },
                Parameter {
                    id: "x".to_owned(),
                    name: "Center X".to_owned(),
                    description: Some("Position of the center in the left-right axis".to_owned()),
                    value: ParameterValue::Number {
                        min: -1.0,
                        max: 1.0,
                        step: 0.05,
                    },
                },
                Parameter {
                    id: "y".to_owned(),
                    name: "Center Y".to_owned(),
                    description: Some("Position of the center in the bottom-top axis".to_owned()),
                    value: ParameterValue::Number {
                        min: -1.0,
                        max: 1.0,
                        step: 0.05,
                    },
                },
                Parameter {
                    id: "z".to_owned(),
                    name: "Center Z".to_owned(),
                    description: Some("Position of the center in the front-back axis".to_owned()),
                    value: ParameterValue::Number {
                        min: -1.0,
                        max: 1.0,
                        step: 0.05,
                    },
                },
                Parameter {
                    id: "width".to_owned(),
                    name: "Ribbon width".to_owned(),
                    description: Some("Width of the ribbon".to_owned()),
                    value: ParameterValue::Number {
                        min: 0.0,
                        max: 1.0,
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
