use animation_api::{Animation, AnimationParameters};
use animation_utils::decorators::BrightnessControlled;
use lightfx::parameter_schema::{EnumOption, Parameter, ParameterValue, ParametersSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize)]
struct Parameters {
    bit: usize,
}

pub struct Indexing {
    points_count: usize,
    parameters: Parameters,
}

impl Indexing {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        BrightnessControlled::new(Box::new(Self {
            points_count: points.len(),
            parameters: Parameters { bit: 0 },
        }))
    }
}

impl Animation for Indexing {
    fn frame(&mut self, _time: f64) -> lightfx::Frame {
        (0..self.points_count)
            .into_iter()
            .map(|x| match (x >> self.parameters.bit) % 2 {
                0 => lightfx::Color::black(),
                _ => lightfx::Color::white(),
            })
            .into()
    }
}

impl AnimationParameters for Indexing {
    fn animation_name(&self) -> &str {
        "Testing: Indexing"
    }

    fn get_fps(&self) -> f64 {
        0.0
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![Parameter {
                id: "bit".to_owned(),
                name: "Bit".to_owned(),
                description: None,
                value: ParameterValue::Enum {
                    values: (0..10)
                        .into_iter()
                        .map(|i| EnumOption {
                            name: format!("{}s", 1 << i),
                            description: None,
                            value: i.to_string(),
                        })
                        .collect(),
                },
            }],
        }
    }

    fn set_parameters(
        &mut self,
        parameters: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.parameters.bit = parameters
            .as_object()
            .and_then(|obj| obj.get("bit"))
            .and_then(|value| value.as_str())
            .and_then(|s| s.parse::<usize>().ok())
            .ok_or_else(|| "Incorrect parameters")?;
        Ok(())
    }

    fn get_parameters(&self) -> serde_json::Value {
        json!(self.parameters)
    }
}
