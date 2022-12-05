use super::{animation::StepAnimation, utils, AnimationParameters};
use itertools::Itertools;
use lightfx::schema::{Parameter, ParameterValue, ParametersSchema};
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    tail_length: f64,
}

pub struct RandomSweep {
    points: Vec<Vector3<f64>>,
    heights: Vec<f64>,
    color: lightfx::Color,
    current_height: f64,
    max_height: f64,
    parameters: Parameters,
}

impl RandomSweep {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn StepAnimation> {
        Box::new(Self {
            points: points
                .iter()
                .map(|(x, y, z)| Vector3::new(*x, *y, *z))
                .collect(),
            heights: Vec::new(),
            color: lightfx::Color::black(),
            current_height: 0.0,
            max_height: 0.0,
            parameters: Parameters { tail_length: 0.5 },
        })
    }
}

impl StepAnimation for RandomSweep {
    fn update(&mut self, delta: f64) {
        if self.current_height > self.max_height + self.parameters.tail_length {
            let rotation = utils::random_rotation();
            self.heights = self
                .points
                .iter()
                .map(|p| rotation * p)
                .map(|p| p.y)
                .collect();
            self.color = utils::random_hue(1.0, 0.5);
            (self.current_height, self.max_height) = match self.heights.iter().minmax() {
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
                if *h < self.current_height
                    && *h > self.current_height - self.parameters.tail_length
                {
                    self.color
                        .dim(1.0 - (self.current_height - h) / self.parameters.tail_length)
                } else {
                    lightfx::Color::black()
                }
            })
            .into()
    }
}

impl AnimationParameters for RandomSweep {
    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![Parameter {
                id: "tail_length".to_owned(),
                name: "Tail length".to_owned(),
                description: Some("Length of the sweep tail".to_owned()),
                value: ParameterValue::Number {
                    min: Some(0.0),
                    max: Some(1.0),
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
