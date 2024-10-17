use animation_api::Animation;
use animation_utils::decorators::BrightnessControlled;
use lightfx::Color;
use serde_json::json;

#[animation_utils::plugin]
pub struct EventTest {
    points: Vec<(f64, f64, f64)>,
    active_points: Vec<bool>,
}

impl EventTest {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        let len = points.len();
        BrightnessControlled::new(Self {
            points,
            active_points: vec![false; len],
        })
    }
}

impl Animation for EventTest {
    type Parameters = ();

    fn update(&mut self, _delta: f64) {}

    fn render(&self) -> lightfx::Frame {
        self.active_points
            .iter()
            .map(|active| {
                if *active {
                    Color::white()
                } else {
                    Color::black()
                }
            })
            .into()
    }

    fn on_event(&mut self, event: animation_api::event::Event) {
        if let animation_api::event::Event::MouseEvent {
            ray_origin,
            ray_direction,
            ..
        } = event
        {
            self.points
                .iter()
                .zip(self.active_points.iter_mut())
                .for_each(|((x, y, z), active)| {
                    let p = nalgebra::Vector3::new(*x as f32, *y as f32, *z as f32);
                    let a = nalgebra::Vector3::new(ray_origin.0, ray_origin.1, ray_origin.2);
                    let n =
                        nalgebra::Vector3::new(ray_direction.0, ray_direction.1, ray_direction.2)
                            .normalize();
                    let distance = (a - p - (a - p).dot(&n) * n).norm();
                    *active |= distance < 0.1;
                });
        }
    }
}
