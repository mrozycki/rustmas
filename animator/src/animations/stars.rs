use std::f64::consts::PI;

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
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    count: f64,
    size: f64,
    color: lightfx::Color,
}

struct Star {
    position: Vector3<f64>,
    age: f64,
}

pub struct Stars {
    points: Vec<Vector3<f64>>,
    stars: Vec<Star>,
    parameters: Parameters,
}

fn random_star() -> Star {
    Star {
        position: Vector3::new(
            utils::random_component(),
            utils::random_component(),
            utils::random_component(),
        ),
        age: rand::thread_rng().gen::<f64>().fract(),
    }
}

impl Stars {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        let mut result = Self {
            points: points
                .iter()
                .map(|(x, y, z)| Vector3::new(*x, *y, *z))
                .collect(),
            stars: vec![],
            parameters: Parameters {
                count: 20.0,
                size: 0.2,
                color: Color::white(),
            },
        };
        result.stars.resize_with(20, random_star);
        SpeedControlled::new(BrightnessControlled::new(StepAnimationDecorator::new(
            Box::new(result),
        )))
    }
}

impl StepAnimation for Stars {
    fn update(&mut self, delta: f64) {
        for star in self.stars.iter_mut() {
            star.age += delta;
            if star.age > 1.0 {
                *star = random_star();
                star.age = 0.0;
            }
        }
    }

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .map(|point| {
                if self.stars.iter().any(|star| {
                    (star.position - point).norm() < self.parameters.size * (star.age * PI).sin()
                }) {
                    self.parameters.color
                } else {
                    Color::black()
                }
            })
            .into()
    }
}

impl AnimationParameters for Stars {
    fn animation_name(&self) -> &str {
        "Stars"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![
                Parameter {
                    id: "count".to_owned(),
                    name: "Count".to_owned(),
                    description: None,
                    value: ParameterValue::Number {
                        min: 10.0,
                        max: 120.0,
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
        self.stars
            .resize_with(self.parameters.count as usize, random_star);
        Ok(())
    }

    fn get_parameters(&self) -> serde_json::Value {
        json!(self.parameters)
    }
}
