use animation_api::Animation;
use animation_utils::{
    decorators::{BrightnessControlled, SpeedControlled},
    Schema,
};
use lightfx::{Color, Gradient};
use nalgebra::{Rotation3, Vector3};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;

struct Surface<T> {
    values: Vec<T>,
    width: usize,
}

impl<T: Copy> Surface<T> {
    fn new(width: usize, height: usize, value: T) -> Self {
        Self {
            values: vec![value; width * height],
            width,
        }
    }
}

impl<T> Surface<T> {
    fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.values.get(y * self.width + x)
    }

    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.values.get_mut(y * self.width + x)
    }

    fn height(&self) -> usize {
        self.values.len() / self.width
    }
}

struct DoomFire {
    surface: Surface<f64>,
    gradient: Gradient,
}

impl DoomFire {
    fn new(width: usize, height: usize) -> Self {
        let mut surface = Surface::new(width, height, 0.0);
        for x in 0..width {
            if let Some(p) = surface.get_mut(x, height - 1) {
                *p = 1.0;
            }
        }
        Self {
            surface,
            gradient: Gradient::new(vec![
                Color::rgb_unit(0.0, 0.0, 0.0), // black
                Color::rgb_unit(1.0, 0.0, 0.0), // red
                Color::rgb_unit(1.0, 0.5, 0.0), // orange
                Color::rgb_unit(1.0, 1.0, 0.0), // yellow
                Color::rgb_unit(1.0, 1.0, 1.0), // white
            ]),
        }
    }

    fn tick(&mut self, parameters: &Parameters) {
        let mut rng = rand::thread_rng();
        for y in 1..self.surface.height() {
            for x in 0..self.surface.width {
                let side_spread = if parameters.side_spread == 0 {
                    rng.gen_range(0..parameters.side_spread) * parameters.side_spread.signum()
                } else {
                    0
                };
                let curr = self.surface.get(x, y).copied().unwrap_or(0.0);
                if let Some(p) = self.surface.get_mut(
                    (x as isize - side_spread as isize + 1).rem_euclid(self.surface.width as isize)
                        as usize,
                    y - 1,
                ) {
                    *p = (curr - rng.gen_range(0.0..1.0) * (1.0 - parameters.upward_spread) / 36.0)
                        .clamp(0.0, 1.0);
                }
            }
        }
    }

    fn sample(&self, x: f64, y: f64) -> Color {
        self.surface
            .get(
                ((x + 1.0) / 2.0 * self.surface.width as f64) as usize,
                ((-y + 1.0) / 2.0 * self.surface.height() as f64) as usize,
            )
            .map(|i| self.gradient.at(*i))
            .unwrap_or(Color::black())
    }
}

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Upward spread", number(min = 0.0, max = 1.0, step = 0.05))]
    upward_spread: f64,

    #[schema_field(name = "Side spread", number(min = "-5.0", max = 5.0, step = 1.0))]
    side_spread: i32,

    #[schema_field(name = "Angle", number(min = 0.0, max = 360.0, step = 5.0))]
    angle: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            upward_spread: 0.33,
            side_spread: 3,
            angle: 0.0,
        }
    }
}

#[animation_utils::plugin]
pub struct DoomFireAnimation {
    points: Vec<Vector3<f64>>,
    parameters: Parameters,
    time: f64,
    fire: DoomFire,
}

impl DoomFireAnimation {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points: points
                .into_iter()
                .map(|(x, y, z)| Vector3::new(x, y, z))
                .collect(),
            parameters: Default::default(),
            time: 0.0,
            fire: DoomFire::new(200, 200),
        }))
    }
}

impl Animation for DoomFireAnimation {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        let mut delta = delta * 30.0;
        while (self.time + delta).floor() > self.time.floor() {
            self.fire.tick(&self.parameters);
            self.time += 1.0;
            delta -= 1.0;
        }
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        let rotation =
            Rotation3::from_axis_angle(&Vector3::y_axis(), self.parameters.angle.to_radians());
        self.points
            .iter()
            .map(|v| rotation * v)
            .map(|v| self.fire.sample(v.x, v.y))
            .into()
    }

    fn get_fps(&self) -> f64 {
        30.0
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }
}
