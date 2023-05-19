use animation_api::parameter_schema::{get_schema, ParametersSchema};
use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::ParameterSchema;
use nalgebra::{Rotation3, Vector3};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, ParameterSchema)]
struct Parameters {
    #[schema_field(name = "Wrap color", color)]
    color_wrap: lightfx::Color,

    #[schema_field(name = "Ribbon color", color)]
    color_ribbon: lightfx::Color,

    #[schema_field(
        name = "Ribbon height",
        description = "Position of the horizontal ribbon",
        number(min = "-1.0", max = 1.0, step = 0.05)
    )]
    height: f64,

    #[schema_field(
        name = "Ribbon thickness",
        description = "Thickness of the ribbon",
        number(min = 0.0, max = 2.0, step = 0.05)
    )]
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

    fn animation_name(&self) -> &str {
        "Present"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        get_schema::<Parameters>()
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
