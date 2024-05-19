use animation_api::Animation;
use animation_utils::decorators::BrightnessControlled;
use animation_utils::Schema;
use lightfx::Color;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Color", color)]
    color: Color,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            color: Color::gray(255),
        }
    }
}

#[animation_utils::plugin]
pub struct EventTest {
    points_count: usize,
    energy: f64,
    decay: f64,
    parameters: Parameters,
}

impl EventTest {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        BrightnessControlled::new(Self {
            points_count: points.len(),
            energy: 0.0,
            decay: 1.0,
            parameters: Default::default(),
        })
    }
}

impl Animation for EventTest {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        self.energy -= delta * self.decay;
    }

    fn render(&self) -> lightfx::Frame {
        (0..self.points_count)
            .map(|_| self.parameters.color.dim(self.energy))
            .into()
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }

    fn on_event(&mut self, event: animation_api::event::Event) {
        if let animation_api::event::Event::BeatEvent { bpm } = event {
            self.energy = 1.0;
            self.decay = bpm / 60.0;
        }
    }
}
