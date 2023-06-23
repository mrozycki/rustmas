use std::f64::consts::{PI, TAU};

use animation_api::Animation;
use animation_utils::{
    decorators::{BrightnessControlled, SpeedControlled},
    ParameterSchema,
};
use lightfx::Color;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::json;

struct Particle {
    angle: f64,
    angle_width: f64,
    speed: f64,
    position: f64,
    length: f64,
}

impl Default for Particle {
    fn default() -> Self {
        Self::new()
    }
}

impl Particle {
    fn new() -> Self {
        Self {
            angle: 0.0,
            angle_width: 0.0,
            speed: 1.0,
            position: 0.0,
            length: 0.5,
        }
    }

    fn new_random(parameters: &Parameters) -> Self {
        Self {
            angle: thread_rng().gen_range(0.0..TAU),
            angle_width: parameters.angle_width / 180.0 * PI,
            speed: thread_rng().gen_range(0.8..1.0) * parameters.max_speed,
            position: 0.0,
            length: parameters.tail_length,
        }
    }

    fn update(&mut self, delta: f64) -> bool {
        self.position += self.speed * delta;
        self.speed *= 1.0 + delta;

        self.position < 3.0
    }

    fn power_at(&self, x: f64, y: f64) -> f64 {
        let angle = y.atan2(x) + PI;
        let d = self.position - (x.powi(2) + y.powi(2)).sqrt();
        if (angle - self.angle).abs() < self.angle_width / 2.0 && d >= 0.0 && d < self.length {
            self.length - d
        } else {
            0.0
        }
    }
}

#[derive(Clone, Serialize, Deserialize, ParameterSchema)]
pub struct Parameters {
    #[schema_field(name = "Center X", number(min = "-1.0", max = 1.0, step = 0.05))]
    center_x: f64,

    #[schema_field(name = "Center Y", number(min = "-1.0", max = 1.0, step = 0.05))]
    center_y: f64,

    #[schema_field(name = "Angle width", number(min = 0.0, max = 45.0, step = 1.0))]
    angle_width: f64,

    #[schema_field(name = "Tail length", number(min = 0.0, max = 0.5, step = 0.01))]
    tail_length: f64,

    #[schema_field(name = "Max speed", number(min = 0.05, max = 1.0, step = 0.05))]
    max_speed: f64,

    #[schema_field(
        name = "Particle generation rate",
        number(min = 0.0, max = 100.0, step = 5.0)
    )]
    gen_rate: usize,

    #[schema_field(name = "Dim distance", number(min = 0.1, max = 1.0, step = 0.1))]
    dim_distance: f64,

    #[schema_field(name = "Ramp up time", number(min = 0.0, max = 10.0, step = 0.5))]
    ramp_up: f64,

    #[schema_field(name = "Color", color)]
    color: Color,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            angle_width: 10.0,
            tail_length: 0.5,
            max_speed: 1.0,
            gen_rate: 50,
            dim_distance: 0.5,
            ramp_up: 1.0,
            color: Color::white(),
        }
    }
}

#[animation_utils::plugin]
pub struct Lightspeed {
    points: Vec<(f64, f64, f64)>,
    time: f64,
    particles: Vec<Particle>,
    to_generate: f64,
    parameters: Parameters,
}

impl Lightspeed {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        let particles = Vec::new();
        SpeedControlled::new(BrightnessControlled::new(Self {
            points,
            time: 0.0,
            particles,
            to_generate: 0.0,
            parameters: Default::default(),
        }))
    }
}

impl Animation for Lightspeed {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        self.time += delta;
        self.particles.retain_mut(|p| p.update(delta));

        self.to_generate += self.parameters.gen_rate as f64
            * delta
            * (self.time / self.parameters.ramp_up).clamp(0.0, 1.0);
        if self.to_generate <= 0.0 {
            return;
        }

        let n = thread_rng().gen_range(0.0..2.0 * self.to_generate).floor();
        for _ in 0..n as usize {
            self.particles.push(Particle::new_random(&self.parameters));
        }
        self.to_generate -= n;
    }

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .map(|(x, y, _z)| {
                self.particles
                    .iter()
                    .map(|p| self.parameters.color.with_alpha(p.power_at(*x, *y)))
                    .fold(Color::black().with_alpha(1.0), |acc, c| c.blend(&acc))
                    .apply_alpha()
                    .dim((x.powi(2) + y.powi(2)).sqrt() / self.parameters.dim_distance)
            })
            .into()
    }

    fn animation_name(&self) -> &str {
        "Lightspeed"
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
