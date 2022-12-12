use crate::Animation;
use lightfx::{
    parameter_schema::{EnumOption, Parameter, ParameterValue},
    schema::ParametersSchema,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::AnimationParameters;

#[derive(Serialize, Deserialize)]
enum Direction {
    Forward,
    Backward,
}

#[derive(Serialize, Deserialize)]
struct Parameters {
    direction: Direction,
}
pub struct DirectionControlled {
    animation: Box<dyn Animation>,
    parameters: Parameters,
    forward_name: String,
    backward_name: String,
    reference_real_time: f64,
    reference_fake_time: f64,
}

impl Animation for DirectionControlled {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        let time_diff = time - self.reference_real_time;
        self.reference_real_time = time;

        self.reference_fake_time += match &self.parameters.direction {
            Direction::Forward => time_diff,
            Direction::Backward => -time_diff,
        };

        self.animation.frame(self.reference_fake_time)
    }
}

impl AnimationParameters for DirectionControlled {
    fn animation_name(&self) -> &str {
        self.animation.animation_name()
    }

    fn parameter_schema(&self) -> ParametersSchema {
        let mut parameters = vec![Parameter {
            id: "direction".to_owned(),
            name: "Direction".to_owned(),
            description: None,
            value: ParameterValue::Enum {
                values: vec![
                    EnumOption {
                        name: self.forward_name.clone(),
                        description: None,
                        value: "Forward".to_owned(),
                    },
                    EnumOption {
                        name: self.backward_name.clone(),
                        description: None,
                        value: "Backward".to_owned(),
                    },
                ],
            },
        }];
        parameters.extend(self.animation.parameter_schema().parameters);
        ParametersSchema { parameters }
    }

    fn set_parameters(
        &mut self,
        parameters: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.parameters = serde_json::from_value(parameters.clone())?;
        self.animation.set_parameters(parameters)?;

        Ok(())
    }

    fn get_parameters(&self) -> serde_json::Value {
        let mut parameters = self
            .animation
            .get_parameters()
            .as_object_mut()
            .unwrap()
            .to_owned();
        parameters.extend(
            json!(self.parameters)
                .as_object_mut()
                .cloned()
                .unwrap()
                .into_iter(),
        );
        json!(parameters)
    }
}

impl DirectionControlled {
    pub fn new(
        forward_name: &str,
        backward_name: &str,
        animation: Box<dyn Animation>,
    ) -> Box<dyn Animation> {
        Box::new(Self {
            animation,
            parameters: Parameters {
                direction: Direction::Forward,
            },
            forward_name: forward_name.to_owned(),
            backward_name: backward_name.to_owned(),
            reference_fake_time: 0.0,
            reference_real_time: 0.0,
        })
    }
}
