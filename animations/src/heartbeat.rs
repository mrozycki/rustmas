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

    #[schema_field(name = "Radius", number(min = "-1.0", max = 1.0, step = 0.1))]
    radius: f64,

    #[schema_field(name = "BPM", number(min = 40.0, max = 240.0, step = 1.0))]
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
            color: Color::rgb(255, 0, 255),
        }
    }
}

#[animation_utils::plugin]
pub struct HeartBoom {
    points: Vec<(f64, f64, f64)>,
    time: f64,
    parameters: Parameters,
}

impl HeartBoom {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        BrightnessControlled::new(Self {
            points,
            time: 0.0,
            parameters: Default::default(),
        })
    }
}

impl Animation for HeartBoom {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        self.time += delta * self.parameters.bpm / 60.0;
    }

    fn render(&self) -> lightfx::Frame {
        let r = self.parameters.radius
            * (((self.time * std::f64::consts::PI).cos() + 0.5).abs() + 0.5)
            / 2.0;

        self.points
            .iter()
            .map(|(x, y, _z)| (x - self.parameters.center_x, y - self.parameters.center_y))
            .map(|(x, y)| (x / r, y / r))
            .map(|(x, y)| x.powi(2) + (1.25 * y - x.abs().sqrt() + 0.35).powi(2))
            .map(|r| {
                if r <= 1.0 {
                    self.parameters.color
                } else {
                    Color::black()
                }
            })
            .into()
    }

    fn animation_name(&self) -> &str {
        "Heartbeat"
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
