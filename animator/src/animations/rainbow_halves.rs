use std::f64::consts::PI;

use animation_api::{Animation, AnimationParameters};
use lightfx::{
    parameter_schema::EnumOption,
    schema::{Parameter, ParameterValue, ParametersSchema},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    brightness_controlled::BrightnessControlled, speed_controlled::SpeedControlled, utils,
};

#[derive(Serialize, Deserialize)]
enum Axis {
    X,
    Y,
    Z,
}

#[derive(Serialize, Deserialize)]
struct Parameters {
    axis: Axis,
}

pub struct RainbowHalves {
    points: Vec<(f64, f64, f64)>,
    parameters: Parameters,
}

impl RainbowHalves {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        SpeedControlled::new(BrightnessControlled::new(Box::new(Self {
            points: points.clone(),
            parameters: Parameters { axis: Axis::Z },
        })))
    }
}

impl Animation for RainbowHalves {
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
                    lightfx::Color::hsv(time / (2.0 * std::f64::consts::PI), 1.0, 1.0)
                } else {
                    lightfx::Color::hsv(time / (2.0 * std::f64::consts::PI) + 0.5, 1.0, 1.0)
                }
            })
            .into()
    }
}

impl AnimationParameters for RainbowHalves {
    fn animation_name(&self) -> &str {
        "Rainbow Halves"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![Parameter {
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
            }],
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
