use super::{utils, Animation};
use nalgebra::Vector3;
use rustmas_animation_model::schema::{Parameter, ParameterValue, ParametersSchema};
use rustmas_light_client as client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Parameters {
    tail_length: f64,
}

pub struct RandomSweep {
    points: Vec<Vector3<f64>>,
    heights: Vec<f64>,
    color: client::Color,
    last_time: f64,
    parameters: Parameters,
}

impl RandomSweep {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points: points
                .iter()
                .map(|(x, y, z)| Vector3::new(*x, *y, *z))
                .collect(),
            heights: Vec::new(),
            color: client::Color::black(),
            last_time: 1.0,
            parameters: Parameters { tail_length: 0.5 },
        }
    }

    fn generate_new_sweep(&mut self) {
        let rotation = utils::random_rotation();
        self.heights = self
            .points
            .iter()
            .map(|p| rotation * p)
            .map(|p| p.y)
            .collect();
        self.color = utils::random_hue(1.0, 0.5);
    }
}

impl Animation for RandomSweep {
    fn frame(&mut self, time: f64) -> client::Frame {
        let time = time % 2.0 - 1.0;
        if self.heights.len() == 0 || (self.last_time > 0.0 && time < 0.0) {
            self.generate_new_sweep();
        }

        self.last_time = time;
        self.heights
            .iter()
            .map(|h| {
                if *h > time && *h < time + self.parameters.tail_length {
                    self.color.dim((h - time) / self.parameters.tail_length)
                } else {
                    client::Color::black()
                }
            })
            .into()
    }

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

    fn get_parameters(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        Ok(serde_json::to_value(&self.parameters)?)
    }
}
