use std::f64::consts::FRAC_PI_2;

use animation_api::Animation;
use animation_utils::Schema;
use animation_utils::decorators::BrightnessControlled;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Center X", number(min = "-1.0", max = 1.0, step = 0.1))]
    center_x: f64,

    #[schema_field(name = "Center Y", number(min = "-1.0", max = 1.0, step = 0.1))]
    center_y: f64,

    #[schema_field(name = "Radius", number(min = "-1.0", max = 1.0, step = 0.1))]
    radius: f64,

    #[schema_field(name = "BPM", number(min = 40.0, max = 240.0, step = 1.0))]
    bpm: f64,

    #[schema_field(name = "Color cycle", number(min = 5.0, max = 60.0, step = 5.0))]
    color_cycle: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            radius: 1.0,
            bpm: 60.0,
            color_cycle: 10.0,
        }
    }
}

#[animation_utils::wasm_plugin]
pub struct CircleBoom {
    points: Vec<(f64, f64, f64)>,
    time: f64,
    parameters: Parameters,
}

impl Animation for CircleBoom {
    type Parameters = Parameters;
    type CustomTriggers = ();
    type Wrapped = BrightnessControlled<Self>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            points,
            time: 0.0,
            parameters: Default::default(),
        }
    }

    fn update(&mut self, delta: f64) {
        self.time += delta * self.parameters.bpm / 60.0;
    }

    fn render(&self) -> lightfx::Frame {
        let r = self.parameters.radius
            * (((self.time * std::f64::consts::PI).cos() + 0.5).abs() + 0.5)
            / 2.0;
        self.points
            .iter()
            .map(|(x, y, _z)| {
                let d = ((x - self.parameters.center_x).powi(2)
                    + (y - self.parameters.center_y).powi(2))
                    / r.powi(2);
                if d <= 1.0 {
                    lightfx::Color::hsv(
                        (self.time / self.parameters.color_cycle).rem_euclid(1.0),
                        1.0,
                        (d * FRAC_PI_2 * 3.0).cos().powi(2),
                    )
                } else {
                    lightfx::Color::black()
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
