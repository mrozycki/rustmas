use std::f64::consts::PI;

use animation_api::parameter_schema::{get_schema, ParametersSchema};
use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::ParameterSchema;
use lightfx::Color;
use nalgebra::Vector3;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, ParameterSchema)]
struct Parameters {
    #[schema_field(name = "Count", number(min = 50.0, max = 150.0, step = 10.0))]
    count: f64,

    #[schema_field(name = "Size", number(min = 0.1, max = 0.5, step = 0.01))]
    size: f64,

    #[schema_field(name = "Color", color)]
    color: lightfx::Color,
}

struct Star {
    position: Vector3<f64>,
    age: f64,
}

#[animation_utils::plugin]
pub struct Stars {
    points: Vec<Vector3<f64>>,
    stars: Vec<Star>,
    parameters: Parameters,
}

fn random_star() -> Star {
    Star {
        position: Vector3::new(
            animation_utils::random_component(),
            animation_utils::random_component(),
            animation_utils::random_component(),
        ),
        age: rand::thread_rng().gen::<f64>().fract(),
    }
}

impl Stars {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        let mut result = Self {
            points: points
                .into_iter()
                .map(|(x, y, z)| Vector3::new(x, y, z))
                .collect(),
            stars: vec![],
            parameters: Parameters {
                count: 20.0,
                size: 0.2,
                color: Color::white(),
            },
        };
        result.stars.resize_with(20, random_star);
        SpeedControlled::new(BrightnessControlled::new(result))
    }
}

impl Animation for Stars {
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

    fn animation_name(&self) -> &str {
        "Stars"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        get_schema::<Parameters>()
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
