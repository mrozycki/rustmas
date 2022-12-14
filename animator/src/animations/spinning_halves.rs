use std::f64::consts::PI;

use super::{
    brightness_controlled::BrightnessControlled, speed_controlled::SpeedControlled, utils,
    Animation, AnimationParameters,
};
use lightfx::{
    parameter_schema::EnumOption,
    schema::{Parameter, ParameterValue, ParametersSchema},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
enum Axis {
    X,
    Y,
    Z,
}

#[derive(Serialize, Deserialize)]
struct Parameters {
    color_a: lightfx::Color,
    color_b: lightfx::Color,
    axis: Axis,
}

pub struct SpinningHalves {
    points: Vec<(f64, f64, f64)>,
    parameters: Parameters,
}

impl SpinningHalves {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        SpeedControlled::new(BrightnessControlled::new(Box::new(Self {
            points: points.clone(),
            parameters: Parameters {
                color_a: lightfx::Color::rgb(255, 0, 0),
                color_b: lightfx::Color::rgb(0, 255, 0),
                axis: Axis::Z,
            },
        })))
    }
}

impl Animation for SpinningHalves {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        self.points
            .iter()
            .map(|(x, y, z)| match self.parameters.axis {
                Axis::X => (*y, *z),
                Axis::Y => (*x, *z),
                Axis::Z => (*x, *y),
            })
            .map(|(x, y)| {
                if utils::cycle(y.atan2(x) / PI + time, 2.0) < 1.0 {
                    self.parameters.color_a
                } else {
                    self.parameters.color_b
                }
            })
            .into()
    }
}

impl AnimationParameters for SpinningHalves {
    fn animation_name(&self) -> &str {
        "Spinning Halves"
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
                    id: "axis".to_owned(),
                    name: "Rotation".to_owned(),
                    description: Some("Axis of rotation".to_owned()),
                    value: ParameterValue::Enum {
                        values: vec![
                            EnumOption {
                                name: "X: Left-Right".to_owned(),
                                description: None,
                                value: "X".to_owned(),
                            },
                            EnumOption {
                                name: "Y: Bottom-Top".to_owned(),
                                description: None,
                                value: "Y".to_owned(),
                            },
                            EnumOption {
                                name: "Z: Front-Back".to_owned(),
                                description: None,
                                value: "Z".to_owned(),
                            },
                        ],
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
