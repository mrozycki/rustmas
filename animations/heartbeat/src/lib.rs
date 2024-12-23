use animation_api::Animation;
use animation_utils::Schema;
use animation_utils::{decorators::BrightnessControlled, EnumSchema};
use lightfx::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, EnumSchema, PartialEq)]
pub enum Switch {
    #[schema_variant(name = "On")]
    #[default]
    On,

    #[schema_variant(name = "Off")]
    Off,
}

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Center X", number(min = "-1.0", max = 1.0, step = 0.1))]
    center_x: f64,

    #[schema_field(name = "Center Y", number(min = "-1.0", max = 1.0, step = 0.1))]
    center_y: f64,

    #[schema_field(name = "Radius", number(min = 0.0, max = 1.0, step = 0.05))]
    radius: f64,

    #[schema_field(name = "BPM", number(min = 20.0, max = 240.0, step = 0.1))]
    bpm: f64,

    #[schema_field(name = "Color", color)]
    color: Color,

    #[schema_field(name = "State", enum_options)]
    state: Switch,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            radius: 1.0,
            bpm: 60.0,
            color: Color::rgb(255, 0, 255),
            state: Switch::On,
        }
    }
}

#[animation_utils::wasm_plugin]
pub struct Heartbeat {
    points: Vec<(f64, f64, f64)>,
    time: f64,
    off_after: Option<f64>,
    parameters: Parameters,
}

impl Animation for Heartbeat {
    type Parameters = Parameters;
    type CustomTriggers = ();
    type Wrapped = BrightnessControlled<Self>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            points,
            time: 0.0,
            off_after: None,
            parameters: Default::default(),
        }
    }

    fn update(&mut self, delta: f64) {
        self.time += delta * self.parameters.bpm / 60.0;
    }

    fn render(&self) -> lightfx::Frame {
        let r = self.parameters.radius
            * if let Some(off_after) = self.off_after.filter(|t| self.time > *t) {
                1.0 - (self.time - off_after).clamp(0.0, 1.0)
            } else {
                (((self.time * std::f64::consts::TAU).cos() + 0.5).abs() + 0.5) / 2.0
            };

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

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        if parameters.state == Switch::Off {
            self.off_after = Some(self.time.ceil());
        }
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
