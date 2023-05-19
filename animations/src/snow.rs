use animation_api::parameter_schema::{get_schema, ParametersSchema};
use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::ParameterSchema;
use lightfx::Color;
use nalgebra::Vector3;
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

#[animation_utils::plugin]
pub struct Snow {
    points: Vec<Vector3<f64>>,
    centers: Vec<Vector3<f64>>,
    parameters: Parameters,
}

fn random_new_center(size: f64) -> Vector3<f64> {
    Vector3::new(
        animation_utils::random_component(),
        animation_utils::random_component() + 2.0 + size,
        animation_utils::random_component(),
    )
}

impl Snow {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        let starting_size = 0.2;
        let mut result = Self {
            points: points
                .into_iter()
                .map(|(x, y, z)| Vector3::new(x, y, z))
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

        SpeedControlled::new(BrightnessControlled::new(result))
    }
}

impl Animation for Snow {
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

    fn animation_name(&self) -> &str {
        "Snow"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        get_schema::<Parameters>()
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
