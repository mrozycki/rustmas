use std::f64::consts::PI;

use animation_api::Animation;
use animation_utils::{
    decorators::{BrightnessControlled, SpeedControlled},
    ParameterSchema,
};
use lightfx::Color;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug)]
struct Pillar {
    position: f64,
    speed: f64,
    width: f64,
    alpha: f64,
    gradient: f64,
}

impl Default for Pillar {
    fn default() -> Self {
        Self::new()
    }
}

impl Pillar {
    fn new() -> Self {
        Self {
            position: -1.0,
            speed: 1.0,
            width: 0.2,
            alpha: 0.5,
            gradient: 0.15_f64.tan(),
        }
    }

    fn new_random(parameters: &Parameters) -> Self {
        let left_to_right = thread_rng().gen_bool(0.5);
        let width = thread_rng().gen_range(0.5..1.0) * parameters.max_width;
        let max_angle = parameters.max_angle / 180.0 * PI;
        let gradient = thread_rng().gen_range(-max_angle..max_angle).tan();
        Self {
            position: if left_to_right {
                parameters.left_boundary - width / 2.0 - gradient.abs()
            } else {
                parameters.right_boundary + gradient.abs()
            },
            speed: thread_rng().gen_range(0.25..1.0)
                * parameters.max_speed
                * if left_to_right { 1.0 } else { -1.0 },
            width,
            alpha: thread_rng().gen_range(0.5..1.0) * parameters.max_alpha,
            gradient,
        }
    }

    fn update(&mut self, delta: f64) -> bool {
        self.position += self.speed * delta;

        if self.speed > 0.0 {
            self.position.min(self.position + 2.0 * self.gradient) < 1.0 + self.width / 2.0
        } else {
            self.position.max(self.position + 2.0 * self.gradient) > -1.0 - self.width / 2.0
        }
    }

    fn power_at(&self, x: f64, y: f64) -> f64 {
        let x = x + y * self.gradient;
        if x > self.position - self.width / 2.0 && x < self.position + self.width / 2.0 {
            (1.0 - (self.position - x).abs() / (self.width / 2.0)) * self.alpha
        } else {
            0.0
        }
    }
}

#[derive(Clone, Serialize, Deserialize, ParameterSchema)]
pub struct Parameters {
    #[schema_field(name = "Gen rate", number(min = 0.0, max = 10.0, step = 0.1))]
    gen_rate: f64,

    #[schema_field(name = "Left boundary", number(min = "-1.0", max = 1.0, step = 0.05))]
    left_boundary: f64,

    #[schema_field(name = "Right boundary", number(min = "-1.0", max = 1.0, step = 0.05))]
    right_boundary: f64,

    #[schema_field(name = "Max width", number(min = 0.1, max = 1.0, step = 0.01))]
    max_width: f64,

    #[schema_field(name = "Max speed", number(min = 0.0, max = 1.0, step = 0.01))]
    max_speed: f64,

    #[schema_field(name = "Max angle", number(min = 0.0, max = 45.0, step = 5.0))]
    max_angle: f64,

    #[schema_field(name = "Max alpha", number(min = 0.0, max = 1.0, step = 0.05))]
    max_alpha: f64,

    #[schema_field(name = "Color", color)]
    color: Color,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            gen_rate: 1.0,
            left_boundary: -1.0,
            right_boundary: 1.0,
            max_width: 0.5,
            max_speed: 0.5,
            max_angle: 15.0,
            max_alpha: 0.7,
            color: Color::white(),
        }
    }
}

#[animation_utils::plugin]
pub struct Pillars {
    points: Vec<(f64, f64, f64)>,
    parameters: Parameters,
    fractional_gen: f64,
    pillars: Vec<Pillar>,
}

impl Pillars {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points,
            parameters: Default::default(),
            fractional_gen: 0.0,
            pillars: Vec::new(),
        }))
    }
}

impl Animation for Pillars {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        self.pillars.retain_mut(|p| p.update(delta));
        self.fractional_gen += self.parameters.gen_rate * delta;

        if self.fractional_gen <= 0.0 {
            return;
        }
        let n = thread_rng()
            .gen_range(0.0..2.0 * self.fractional_gen)
            .floor();
        if n == 0.0 {
            return;
        }

        self.fractional_gen -= n;
        for _ in 0..n as usize {
            self.pillars.push(Pillar::new_random(&self.parameters))
        }
    }

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .map(|(x, y, _z)| {
                self.pillars
                    .iter()
                    .map(|p| self.parameters.color.with_alpha(p.power_at(*x, *y)))
                    .fold(Color::black().with_alpha(1.0), |acc, c| c.blend(&acc))
                    .apply_alpha()
            })
            .into()
    }

    fn animation_name(&self) -> &str {
        "Random pillar"
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
