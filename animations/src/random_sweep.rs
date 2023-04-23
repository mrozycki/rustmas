use animation_api::parameter_schema::{Parameter, ParameterValue, ParametersSchema};
use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use itertools::Itertools;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    tail_length: f64,
}

#[animation_utils::plugin]
pub struct RandomSweep {
    points: Vec<Option<Vector3<f64>>>,
    heights: Vec<Option<f64>>,
    color: lightfx::Color,
    current_height: f64,
    max_height: f64,
    parameters: Parameters,
}

impl RandomSweep {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points: points
                .into_iter()
                .map(|(x, y, z)| {
                    if x.to_bits() == (-1.0_f64).to_bits()
                        && y.to_bits() == (-1.0_f64).to_bits()
                        && z.to_bits() == (-1.0_f64).to_bits()
                    {
                        None
                    } else {
                        Some(Vector3::new(x, y, z))
                    }
                })
                .collect(),
            heights: Vec::new(),
            color: lightfx::Color::black(),
            current_height: 0.0,
            max_height: 0.0,
            parameters: Parameters { tail_length: 0.5 },
        }))
    }
}

impl Animation for RandomSweep {
    fn update(&mut self, delta: f64) {
        if self.current_height > self.max_height + self.parameters.tail_length {
            let rotation = animation_utils::random_rotation();
            self.heights = self
                .points
                .iter()
                .map(|p| p.map(|p| rotation * p))
                .map(|p| p.map(|p| p.y))
                .collect();
            self.color = animation_utils::random_hue(1.0, 1.0);
            (self.current_height, self.max_height) =
                match self.heights.iter().filter_map(|x| x.as_ref()).minmax() {
                    itertools::MinMaxResult::MinMax(min, max) => (*min, *max),
                    _ => return,
                };
        }

        self.current_height += delta;
    }

    fn render(&self) -> lightfx::Frame {
        self.heights
            .iter()
            .map(|h| {
                if let Some(h) = h {
                    if *h < self.current_height
                        && *h > self.current_height - self.parameters.tail_length
                    {
                        self.color
                            .dim(1.0 - (self.current_height - h) / self.parameters.tail_length)
                    } else {
                        lightfx::Color::black()
                    }
                } else {
                    lightfx::Color::black()
                }
            })
            .into()
    }

    fn animation_name(&self) -> &str {
        "Rainbow Sweep"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![Parameter {
                id: "tail_length".to_owned(),
                name: "Tail length".to_owned(),
                description: Some("Length of the sweep tail".to_owned()),
                value: ParameterValue::Number {
                    min: 0.0,
                    max: 2.0,
                    step: 0.05,
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
