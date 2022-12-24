use super::{
    animation::{StepAnimation, StepAnimationDecorator},
    brightness_controlled::BrightnessControlled,
    speed_controlled::SpeedControlled,
    utils, Animation, AnimationParameters,
};
use lightfx::{
    schema::{Parameter, ParameterValue, ParametersSchema},
    Color,
};
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    count: f64,
    size: f64,
    color: lightfx::Color,
}

pub struct Snow {
    points: Vec<Vector3<f64>>,
    centers: Vec<Vector3<f64>>,
    parameters: Parameters,
}

fn random_new_center(size: f64) -> Vector3<f64> {
    Vector3::new(
        utils::random_component(),
        utils::random_component() + 2.0 + size,
        utils::random_component(),
    )
}

impl Snow {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        let starting_size = 0.2;
        let mut result = Self {
            points: points
                .iter()
                .map(|(x, y, z)| Vector3::new(*x, *y, *z))
                .collect(),
            centers: vec![],
            parameters: Parameters {
                count: 20.0,
                size: starting_size,
                color: Color::white(),
            },
        };
        result
            .centers
            .resize_with(20, || random_new_center(starting_size));

        SpeedControlled::new(BrightnessControlled::new(StepAnimationDecorator::new(
            Box::new(result),
        )))
    }
}

impl StepAnimation for Snow {
    fn update(&mut self, delta: f64) {
        for center in self.centers.iter_mut() {
            center.y -= delta;
            if center.y < -1.0 - self.parameters.size {
                *center = random_new_center(self.parameters.size);
                center.y = 1.0 + self.parameters.size;
            }
        }
    }

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .map(|point| {
                if self
                    .centers
                    .iter()
                    .any(|center| (center - point).norm() < self.parameters.size)
                {
                    self.parameters.color
                } else {
                    Color::black()
                }
            })
            .into()
    }
}

impl AnimationParameters for Snow {
    fn animation_name(&self) -> &str {
        "Snow"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![
                Parameter {
                    id: "count".to_owned(),
                    name: "Count".to_owned(),
                    description: None,
                    value: ParameterValue::Number {
                        min: 50.0,
                        max: 150.0,
                        step: 10.0,
                    },
                },
                Parameter {
                    id: "size".to_owned(),
                    name: "Size".to_owned(),
                    description: None,
                    value: ParameterValue::Number {
                        min: 0.1,
                        max: 0.5,
                        step: 0.01,
                    },
                },
                Parameter {
                    id: "color".to_owned(),
                    name: "Color".to_owned(),
                    description: None,
                    value: ParameterValue::Color,
                },
            ],
        }
    }

    fn set_parameters(
        &mut self,
        parameters: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.parameters = serde_json::from_value(parameters)?;
        self.centers
            .resize_with(self.parameters.count as usize, || {
                random_new_center(self.parameters.size)
            });
        Ok(())
    }

    fn get_parameters(&self) -> serde_json::Value {
        json!(self.parameters)
    }
}
