use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::Schema;
use nalgebra::{Rotation3, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
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

impl Default for Parameters {
    fn default() -> Self {
        Self {
            color_wrap: lightfx::Color::rgb(255, 255, 255),
            color_ribbon: lightfx::Color::rgb(255, 0, 0),
            height: 0.0,
            width: 0.1,
        }
    }
}

#[animation_utils::wasm_plugin]
pub struct Present {
    points: Vec<Vector3<f64>>,
    time: f64,
    parameters: Parameters,
}

impl Animation for Present {
    type Parameters = Parameters;
    type Wrapped = SpeedControlled<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            points: points
                .into_iter()
                .map(|(x, y, z)| Vector3::new(x, y, z))
                .collect(),
            time: 0.0,
            parameters: Default::default(),
        }
    }

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

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
