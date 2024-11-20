use std::collections::VecDeque;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::{to_polar, EnumSchema, Schema};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use lightfx::Color;

use serde::{Deserialize, Serialize};
use serde_json::json;

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

#[animation_utils::plugin]
pub struct AudioVisualizer {
    points: Vec<(f64, f64, f64)>,
    parameters: Parameters,
    buffer: Arc<Mutex<VecDeque<f32>>>,
    _stream: Stream,
}

impl Animation for AudioVisualizer {
    type Parameters = Parameters;
    type Wrapped = SpeedControlled<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        let buffer: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(vec![0.0; BUFFER_SIZE].into()));
        let input_data_fn = {
            let buffer = buffer.clone();
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buf = buffer.lock().unwrap();
                for &sample in data {
                    buf.push_front(sample);
                    buf.pop_back();
                }
            }
        };

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };
        let host = cpal::default_host();

        let device = host.default_input_device().unwrap();
        let config: cpal::StreamConfig = device.default_input_config().unwrap().into();

        let input_stream = device
            .build_input_stream(&config, input_data_fn, err_fn, None)
            .unwrap();
        input_stream.play().unwrap();

        Self {
            points,
            parameters: Default::default(),
            buffer,
            _stream: input_stream,
        }
    }

    fn update(&mut self, _: f64) {}

    fn render(&self) -> lightfx::Frame {
        let buf = self.buffer.lock().unwrap();

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
                if (buf[((x * BUFFER_SIZE as f64) as usize).clamp(0, buf.len() - 1)] as f64 - y)
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

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
    fn get_fps(&self) -> f64 {
        60.0
    }
}
