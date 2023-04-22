use animation_api::{Animation, AnimationParameters};
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use lightfx::schema::{Parameter, ParameterValue, ParametersSchema};
use nalgebra::{Rotation3, Vector3};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    color_wrap: lightfx::Color,
    color_ribbon: lightfx::Color,
    height: f64,
    width: f64,
}

#[animation_utils::plugin]
pub struct Present {
    points: Vec<Vector3<f64>>,
    time: f64,
    parameters: Parameters,
}

impl Present {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points: points
                .into_iter()
                .map(|(x, y, z)| Vector3::new(x, y, z))
                .collect(),
            time: 0.0,
            parameters: Parameters {
                color_wrap: lightfx::Color::rgb(255, 255, 255),
                color_ribbon: lightfx::Color::rgb(255, 0, 0),
                height: 0.0,
                width: 0.1,
            },
        }))
    }
}

impl Animation for Present {
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        let rotation = Rotation3::new(Vector3::y() * 2.0 * std::f64::consts::PI * self.time);

        self.points
            .iter()
            .map(|p| rotation * p)
            .map(|p| {
                let dist_x = p.x.abs();
                let dist_y = (p.y - self.parameters.height).abs();
                let dist_z = p.z.abs();

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
                    id: "height".to_owned(),
                    name: "Height".to_owned(),
                    description: Some("Position of the horizontal ribbon".to_owned()),
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
