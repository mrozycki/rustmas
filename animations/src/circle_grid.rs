use std::f64::consts::TAU;

use animation_api::Animation;
use animation_utils::{decorators::BrightnessControlled, ParameterSchema};
use lightfx::Color;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, ParameterSchema)]
pub struct Parameters {
    #[schema_field(name = "Center X", number(min = "-1.0", max = 1.0, step = 0.1))]
    center_x: f64,

    #[schema_field(name = "Center Y", number(min = "-1.0", max = 1.0, step = 0.1))]
    center_y: f64,

    #[schema_field(name = "Grid size", number(min = 0.0, max = 2.0, step = 0.01))]
    grid_size: f64,

    #[schema_field(name = "Radius", number(min = 0.0, max = 1.0, step = 0.01))]
    radius: f64,

    #[schema_field(name = "Halo size", number(min = 0.0, max = 1.0, step = 0.01))]
    halo_size: f64,

    #[schema_field(name = "Color", color)]
    color: Color,

    #[schema_field(
        name = "Size cycle time (seconds)",
        number(min = 5.0, max = 60.0, step = 5.0)
    )]
    size_cycle_time: f64,

    #[schema_field(
        name = "Size cycle deviation (percent)",
        number(min = 0.0, max = 100.0, step = 5.0)
    )]
    size_cycle_deviation: f64,

    #[schema_field(
        name = "Rotation cycle time (seconds)",
        number(min = 5.0, max = 60.0, step = 5.0)
    )]
    rotation_time: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            grid_size: 0.4,
            radius: 0.1,
            halo_size: 0.02,
            color: Color::white(),
            size_cycle_time: 10.0,
            size_cycle_deviation: 10.0,
            rotation_time: 10.0,
        }
    }
}

#[animation_utils::plugin]
pub struct CircleGrid {
    points: Vec<(f64, f64, f64)>,
    time: f64,
    parameters: Parameters,
}

impl CircleGrid {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        BrightnessControlled::new(Self {
            points,
            time: 0.0,
            parameters: Default::default(),
        })
    }
}

impl Animation for CircleGrid {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        let factor = (self.time * TAU / self.parameters.size_cycle_time).sin()
            * self.parameters.size_cycle_deviation
            / 100.0;
        let grid_size = self.parameters.grid_size * (1.0 + factor);
        let (angle_sin, angle_cos) = (self.time * TAU / self.parameters.rotation_time).sin_cos();

        self.points
            .iter()
            .map(|(x, y, _z)| {
                (
                    (x - self.parameters.center_x),
                    (y - self.parameters.center_y),
                )
            })
            .map(|(x, y)| (x * angle_cos - y * angle_sin, x * angle_sin + y * angle_cos))
            .map(|(x, y)| (x.rem_euclid(grid_size), y.rem_euclid(grid_size)))
            .map(|(x, y)| {
                ((x - self.parameters.grid_size / 2.0).powi(2)
                    + (y - self.parameters.grid_size / 2.0).powi(2))
                .sqrt()
            })
            .map(|r| {
                if r < self.parameters.radius {
                    self.parameters.color
                } else if r < self.parameters.halo_size + self.parameters.radius {
                    self.parameters.color.dim(
                        (self.parameters.halo_size + self.parameters.radius - r)
                            / self.parameters.halo_size,
                    )
                } else {
                    Color::black()
                }
            })
            .into()
    }

    fn animation_name(&self) -> &str {
        "Circle grid"
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
