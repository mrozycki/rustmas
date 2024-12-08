use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::Schema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Density", number(min = 0.5, max = 5.0, step = 0.05))]
    density: f64,

    #[schema_field(
        name = "Height",
        description = "Height of the center of the sphere",
        number(min = "-1.0", max = 1.0, step = 0.05)
    )]
    height: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            density: 1.0,
            height: 0.0,
        }
    }
}

#[animation_utils::wasm_plugin]
pub struct RainbowSphere {
    points: Vec<(f64, f64, f64)>,
    points_radius: Vec<f64>,
    time: f64,
    parameters: Parameters,
}

impl RainbowSphere {
    fn reset(&mut self) {
        self.points_radius = self
            .points
            .iter()
            .map(|(x, y, z)| (x.powi(2) + (y - self.parameters.height).powi(2) + z.powi(2)).sqrt())
            .collect();
    }
}

impl Animation for RainbowSphere {
    type Parameters = Parameters;
    type Wrapped = SpeedControlled<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        let mut result = Self {
            points,
            points_radius: vec![],
            time: 0.0,
            parameters: Default::default(),
        };
        result.reset();
        result
    }

    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        self.points_radius
            .iter()
            .map(|r| lightfx::Color::hsv(r / 2.0 * self.parameters.density + self.time, 1.0, 1.0))
            .into()
    }

    fn set_parameters(&mut self, parameters: Parameters) {
        self.parameters = parameters;
        self.reset();
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
