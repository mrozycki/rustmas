use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

use animation_api::event::Event;
use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::{to_polar, EnumSchema, Schema};
use lightfx::Color;

use serde::{Deserialize, Serialize};

const BUFFER_SIZE: usize = 1024;

#[derive(Clone, Serialize, Deserialize, EnumSchema)]
pub enum ColorScheme {
    #[schema_variant(name = "Selected color")]
    Selected,
    #[schema_variant(name = "Rainbow")]
    Rainbow,
}

#[derive(Clone, Serialize, Deserialize, EnumSchema)]
pub enum Orientation {
    #[schema_variant(name = "XY")]
    XY,
    #[schema_variant(name = "Around")]
    Around,
}

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Color scheme", enum_options)]
    color_scheme: ColorScheme,

    #[schema_field(name = "Selected color", color)]
    selected_color: lightfx::Color,

    #[schema_field(name = "Band height", number(min = 0.0, max = 1.0, step = 0.05))]
    height: f64,

    #[schema_field(name = "Orientation", enum_options)]
    orientation: Orientation,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Selected,
            selected_color: lightfx::Color::kelvin(2700),
            height: 0.1,
            orientation: Orientation::Around,
        }
    }
}

#[animation_utils::wasm_plugin]
pub struct AudioWave {
    points: Vec<(f64, f64, f64)>,
    parameters: Parameters,
    buffer: Arc<Mutex<Vec<f32>>>,
}

impl Animation for AudioWave {
    type Parameters = Parameters;
    type CustomTriggers = ();
    type Wrapped = SpeedControlled<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        let buffer = Arc::new(Mutex::new(vec![0.0; BUFFER_SIZE]));

        Self {
            points,
            parameters: Default::default(),
            buffer,
        }
    }

    fn render(&self) -> lightfx::Frame {
        let buf = self.buffer.lock().unwrap();
        let buffer_size = buf.len();

        self.points
            .iter()
            .copied()
            .map(match self.parameters.orientation {
                Orientation::XY => |(x, y, _)| (x, y),
                Orientation::Around => |p| {
                    let p = to_polar(p);
                    ((p.1 / PI).rem_euclid(2.0) - 1.0, p.2)
                },
            })
            .map(|(x, y)| ((x + 1.0) / 2.0, y))
            .map(|(x, y)| {
                if (buf[((x * buffer_size as f64) as usize).clamp(0, buf.len() - 1)] as f64 - y)
                    .abs()
                    < self.parameters.height / 2.0
                {
                    match self.parameters.color_scheme {
                        ColorScheme::Selected => self.parameters.selected_color,
                        ColorScheme::Rainbow => Color::hsv(x, 1.0, 1.0),
                    }
                } else {
                    Color::black()
                }
            })
            .into()
    }

    fn on_event(&mut self, _event: animation_api::event::Event) {
        if let Event::FftEvent { wave, .. } = _event {
            *self.buffer.lock().unwrap() = wave;
        }
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
