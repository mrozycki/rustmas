use animation_api::{Animation, event::Event};
use animation_utils::{Schema, decorators::SpeedControlled};
use itertools::Itertools;
use midi_msg::{ChannelVoiceMsg, MidiMsg};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Radius", number(min = 0.0, max = 2.0, step = 0.1))]
    radius: f64,

    #[schema_field(name = "Dropoff", number(min = 0.1, max = 1.0, step = 0.1))]
    dropoff: f64,

    #[schema_field(name = "Center X", number(min = "-1.0", max = 1.0, step = 0.1))]
    center_x: f64,

    #[schema_field(name = "Center Y", number(min = "-1.0", max = 1.0, step = 0.1))]
    center_y: f64,

    #[schema_field(name = "Color", color)]
    color: lightfx::Color,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            radius: 1.0,
            dropoff: 1.0,
            center_x: 0.0,
            center_y: 0.0,
            color: lightfx::Color::white(),
        }
    }
}

#[animation_utils::wasm_plugin]
pub struct MidiWave {
    points: Vec<(usize, f64)>,
    control_points: Vec<f64>,
    parameters: Parameters,
}

impl Animation for MidiWave {
    type Parameters = Parameters;
    type CustomTriggers = ();
    type Wrapped = SpeedControlled<Self>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        let points = points
            .into_iter()
            .map(|(x, y, _)| (x.powi(2) + y.powi(2)).sqrt())
            .enumerate()
            .sorted_by(|(_, r0), (_, r1)| r0.total_cmp(r1))
            .collect();

        Self {
            points,
            control_points: Vec::new(),
            parameters: Default::default(),
        }
    }

    fn update(&mut self, time_delta: f64) {
        self.control_points.retain_mut(|p| {
            *p += time_delta;
            *p < 5.0
        });
    }

    fn on_event(&mut self, event: Event) {
        if matches!(
            event,
            Event::MidiEvent(MidiMsg::ChannelVoice {
                msg: ChannelVoiceMsg::NoteOn { .. },
                ..
            })
        ) {
            self.control_points.push(0.0)
        }
    }

    fn render(&self) -> lightfx::Frame {
        let mut control_points_iter = self.control_points.iter().rev();
        let mut control_point = control_points_iter.next();
        self.points
            .iter()
            .map(|(i, r)| {
                while control_point.is_some_and(|x| x < r) {
                    control_point = control_points_iter.next();
                }
                (
                    i,
                    control_point
                        .map(|x| {
                            self.parameters
                                .color
                                .dim(1.0 - (x - r) / self.parameters.dropoff)
                        })
                        .unwrap_or(lightfx::Color::black()),
                )
            })
            .sorted_by(|(i0, _), (i1, _)| i0.cmp(i1))
            .map(|(_, c)| c)
            .into()
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }
}
