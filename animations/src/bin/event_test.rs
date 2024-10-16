use animation_api::Animation;
use animation_utils::decorators::BrightnessControlled;
use lightfx::Color;
use serde_json::json;

#[animation_utils::plugin]
pub struct EventTest {
    points_count: usize,
    energy: f64,
    color: Color,
}

impl EventTest {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        BrightnessControlled::new(Self {
            points_count: points.len(),
            energy: 0.0,
            color: Color::black(),
        })
    }
}

impl Animation for EventTest {
    type Parameters = ();

    fn update(&mut self, delta: f64) {
        self.energy -= delta;
    }

    fn render(&self) -> lightfx::Frame {
        (0..self.points_count)
            .map(|_| self.color.dim(self.energy))
            .into()
    }

    fn on_event(&mut self, event: animation_api::event::Event) {
        if let animation_api::event::Event::ManualEvent { id } = event {
            self.energy = 1.0;
            self.color = match id {
                0 => Color::rgb(255, 0, 0),
                1 => Color::rgb(0, 255, 0),
                2 => Color::rgb(0, 0, 255),
                _ => Color::black(),
            }
        }
    }
}
