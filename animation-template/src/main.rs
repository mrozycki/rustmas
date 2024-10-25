use animation_api::{event::Event, Animation};
use animation_utils::Schema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    // TODO: Define your animation's parameters
    #[schema_field(name = "Tail length", number(min = 1.0, max = 10.0, step = 1.0))]
    tail_length: usize,
}

impl Default for Parameters {
    fn default() -> Self {
        Self { tail_length: 3 }
    }
}

#[animation_utils::plugin]
pub struct MyAnimation {
    // TODO: Define your animation's state
    points_count: usize,
    time: f64,
    parameters: Parameters,
}

impl MyAnimation {
    pub fn create(points: Vec<(f64, f64, f64)>) -> Self {
        // TODO: Initialize animation state from a set of light locations
        Self {
            points_count: points.len(),
            time: 0.0,
            parameters: Default::default(),
        }
    }
}

impl Animation for MyAnimation {
    type Parameters = Parameters;

    fn update(&mut self, time_delta: f64) {
        // TODO: Update your animation state by time_delta seconds
        self.time += time_delta;
    }

    fn on_event(&mut self, event: Event) {
        // TODO: React to selected types of events by matching on `event` parameters
        // Other event types are available.
        match event {
            Event::MidiEvent(_midi_msg) => (),
            Event::MouseMove {
                ray_origin: _,
                ray_direction: _,
            } => (),
            _ => (),
        }
    }

    fn render(&self) -> lightfx::Frame {
        // TODO: Render a frame of your animation
        let index = ((self.time * 8.0) % self.points_count as f64) as usize;

        (0..self.points_count)
            .map(|i| {
                if i + self.parameters.tail_length > index && i <= index {
                    lightfx::Color::white()
                } else {
                    lightfx::Color::black()
                }
            })
            .into()
    }

    fn get_fps(&self) -> f64 {
        // TODO: Return the FPS of your animation
        8.0
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        // You might need to reset the state of your animation in some cases.
        // Otherwise there's nothing to do here.
        self.parameters = parameters;
    }
}
