use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::ParameterSchema;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use lightfx::Color;

use serde::{Deserialize, Serialize};
use serde_json::json;

const BUFFER_SIZE: usize = 1024;

#[derive(Clone, Serialize, Deserialize, ParameterSchema)]
pub struct Parameters {
    #[schema_field(name = "Color", color)]
    color: lightfx::Color,

    #[schema_field(name = "Band height", number(min = 0.0, max = 1.0, step = 0.05))]
    height: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            color: lightfx::Color::rgb(255, 0, 0),
            height: 0.5,
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

impl AudioVisualizer {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
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

        SpeedControlled::new(BrightnessControlled::new(Self {
            points,
            parameters: Default::default(),
            buffer,
            _stream: input_stream,
        }))
    }
}

impl Animation for AudioVisualizer {
    type Parameters = Parameters;

    fn update(&mut self, _: f64) {}

    fn render(&self) -> lightfx::Frame {
        let buf = self.buffer.lock().unwrap();

        self.points
            .iter()
            .map(|x| (x.0, x.1))
            .map(|(x, y)| {
                if (buf[((((x + 1.0) / 2.0) * BUFFER_SIZE as f64) as usize).clamp(0, buf.len() - 1)]
                    as f64
                    - y)
                    .abs()
                    < self.parameters.height / 2.0
                {
                    Color::gray(200)
                } else {
                    Color::black()
                }
            })
            .into()
    }

    fn animation_name(&self) -> &str {
        "Audio Wave"
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
    fn get_fps(&self) -> f64 {
        100.0
    }
}
