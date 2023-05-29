use std::f64::consts::TAU;

use animation_api::Animation;
use animation_utils::decorators::BrightnessControlled;
use animation_utils::ParameterSchema;
use lightfx::Color;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, ParameterSchema)]
pub struct Parameters {
    #[schema_field(name = "Center X", number(min = "-1.0", max = 1.0, step = 0.1))]
    center_x: f64,

    #[schema_field(name = "Center Y", number(min = "-1.0", max = 1.0, step = 0.1))]
    center_y: f64,

    #[schema_field(name = "Radius", number(min = 0.0, max = 2.0, step = 0.1))]
    radius: f64,

    #[schema_field(name = "BPM", number(min = 20.0, max = 240.0, step = 0.1))]
    bpm: f64,

    #[schema_field(name = "Color", color)]
    color: Color,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            radius: 1.0,
            bpm: 60.0,
            color: Color::rgb(0, 138, 106),
        }
    }
}

#[animation_utils::plugin]
pub struct CircleWave {
    points: Vec<(f64, f64, f64)>,
    time: f64,
    parameters: Parameters,
}

impl CircleWave {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        BrightnessControlled::new(Self {
            points,
            time: 0.0,
            parameters: Default::default(),
        })
    }
}

impl Animation for CircleWave {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        self.time += delta * self.parameters.bpm / 60.0;
    }

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .map(|(x, y, _z)| {
                let d = ((x - self.parameters.center_x).powi(2)
                    + (y - self.parameters.center_y).powi(2))
                .sqrt();
                if d > self.time {
                    Color::black()
                } else {
                    self.parameters
                        .color
                        .dim((((self.time - d) * TAU / self.parameters.radius).sin() + 1.0) / 2.0)
                }
            })
            .into()
    }

    fn animation_name(&self) -> &str {
        "Circle Wave"
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
