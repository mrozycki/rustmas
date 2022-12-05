use super::{Animation, AnimationParameters};
use lightfx::schema::{Parameter, ParameterValue, ParametersSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    x: f64,
    y: f64,
    z: f64,
}

pub struct RainbowSphere {
    points: Vec<(f64, f64, f64)>,
    points_radius: Vec<f64>,
    parameters: Parameters,
}

impl RainbowSphere {
    fn reset(&mut self) {
        self.points_radius = self
            .points
            .iter()
            .map(|(x, y, z)| {
                ((x - self.parameters.x).powi(2)
                    + (y - self.parameters.y).powi(2)
                    + (z - self.parameters.z).powi(2))
                .sqrt()
            })
            .collect();
    }

    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        let mut result = Self {
            points_radius: vec![],
            points: points.clone(),
            parameters: Parameters {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        };
        result.reset();
        Box::new(result)
    }
}

impl Animation for RainbowSphere {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        self.points_radius
            .iter()
            .map(|r| lightfx::Color::hsv(r - time, 1.0, 1.0))
            .into()
    }
}

impl AnimationParameters for RainbowSphere {
    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![
                Parameter {
                    id: "x".to_owned(),
                    name: "Center X".to_owned(),
                    description: Some("Position of the center in the left-right axis".to_owned()),
                    value: ParameterValue::Number {
                        min: Some(-1.0),
                        max: Some(1.0),
                    },
                },
                Parameter {
                    id: "y".to_owned(),
                    name: "Center Y".to_owned(),
                    description: Some("Position of the center in the bottom-top axis".to_owned()),
                    value: ParameterValue::Number {
                        min: Some(-1.0),
                        max: Some(1.0),
                    },
                },
                Parameter {
                    id: "z".to_owned(),
                    name: "Center Z".to_owned(),
                    description: Some("Position of the center in the front-back axis".to_owned()),
                    value: ParameterValue::Number {
                        min: Some(-1.0),
                        max: Some(1.0),
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
        self.reset();
        Ok(())
    }

    fn get_parameters(&self) -> serde_json::Value {
        json!(self.parameters)
    }
}
