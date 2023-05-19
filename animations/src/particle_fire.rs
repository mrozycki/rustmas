use animation_api::Animation;
use animation_utils::{
    decorators::{BrightnessControlled, SpeedControlled},
    ParameterSchema,
};
use lightfx::{Color, ColorWithAlpha, Gradient};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::json;

struct Particle {
    position: (f64, f64),
    speed: (f64, f64),
    decay_rate: f64,
    power: f64,
}

impl Default for Particle {
    fn default() -> Self {
        Self::new()
    }
}

impl Particle {
    fn new() -> Self {
        Self {
            position: (-1.0, -1.0),
            speed: (0.0, 0.0),
            decay_rate: 1.0,
            power: 0.0,
        }
    }

    fn new_random(parameters: &Parameters) -> Self {
        let decay_rate = if parameters.decay_rate_spread.abs() < 0.0001 {
            parameters.decay_rate
        } else {
            thread_rng().gen_range(
                parameters.decay_rate * (1.0 - parameters.decay_rate_spread)
                    ..parameters.decay_rate * (1.0 + parameters.decay_rate_spread),
            )
        }
        .clamp(0.0, 1.0);

        Self {
            position: (
                thread_rng().gen_range(-1.0..1.0),
                parameters.bottom_line - parameters.particle_range,
            ),
            speed: (0.0, thread_rng().gen_range(0.5..1.0)),
            decay_rate,
            power: 1.0,
        }
    }

    fn update(&mut self, delta: f64, wind: f64) -> bool {
        let (x, y) = self.position;
        let (vx, vy) = self.speed;
        self.position = ((1.0 + x + vx * delta).rem_euclid(2.0) - 1.0, y + vy * delta);

        let wind = if wind == 0.0 {
            0.0
        } else if wind < 0.0 {
            thread_rng().gen_range(wind * 1.1..wind / 1.1)
        } else {
            thread_rng().gen_range(wind / 1.1..wind * 1.1)
        };
        self.speed = (vx + wind, vy);

        self.power -= self.decay_rate * delta;

        if self.power > 0.0 {
            true
        } else {
            self.power = 0.0;
            false
        }
    }

    fn distance_to(&self, x: f64, y: f64) -> f64 {
        ((x - self.position.0).powi(2) + (y - self.position.1).powi(2)).sqrt()
    }
}

#[derive(Clone, Serialize, Deserialize, ParameterSchema)]
pub struct Parameters {
    #[schema_field(name = "Particle count", number(min = 100.0, max = 500.0, step = 20.0))]
    particle_count: usize,

    #[schema_field(name = "Decay rate", number(min = 0.0, max = 1.0, step = 0.01))]
    decay_rate: f64,

    #[schema_field(name = "Decay rate spread", number(min = 0.0, max = 0.2, step = 0.01))]
    decay_rate_spread: f64,

    #[schema_field(name = "Bottom line", number(min = "-1.0", max = 1.0, step = 0.01))]
    bottom_line: f64,

    #[schema_field(name = "Particle range", number(min = 0.0, max = 1.0, step = 0.01))]
    particle_range: f64,

    #[schema_field(name = "Wind", number(min = "-0.1", max = 0.1, step = 0.001))]
    wind: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            particle_count: 200,
            decay_rate: 0.5,
            decay_rate_spread: 0.1,
            bottom_line: -1.0,
            particle_range: 0.2,
            wind: 0.0,
        }
    }
}

#[animation_utils::plugin]
pub struct DoomFireAnimation {
    points: Vec<(f64, f64, f64)>,
    parameters: Parameters,
    particles: Vec<Option<Particle>>,
    gradient: Gradient,
}

impl DoomFireAnimation {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        let particles = Vec::new();
        SpeedControlled::new(BrightnessControlled::new(Self {
            points,
            parameters: Default::default(),
            particles,
            gradient: Gradient::new(vec![
                Color::rgb_unit(0.0, 0.0, 0.0), // black
                Color::rgb_unit(1.0, 0.0, 0.0), // red
                Color::rgb_unit(1.0, 0.5, 0.0), // orange
                Color::rgb_unit(1.0, 1.0, 0.0), // yellow
                Color::rgb_unit(1.0, 1.0, 1.0), // white
            ]),
        }))
    }
}

impl Animation for DoomFireAnimation {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        for particle in &mut self.particles {
            if let Some(p) = particle {
                if !p.update(delta, self.parameters.wind) {
                    *p = Particle::new_random(&self.parameters);
                }
            } else if thread_rng().gen_bool((self.parameters.decay_rate * delta).clamp(0.0, 1.0)) {
                *particle = Some(Particle::new_random(&self.parameters));
            }
        }
    }

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .map(|(x, y, _z)| {
                if *y < self.parameters.bottom_line {
                    self.gradient.at(1.0)
                } else {
                    self.particles
                        .iter()
                        .flatten()
                        .map(|p| (p.power, p.distance_to(*x, *y)))
                        .fold(
                            ColorWithAlpha::new(Color::black(), 0.0),
                            |color, (power, distance)| {
                                ColorWithAlpha::new(
                                    self.gradient.at(power),
                                    (1.0 - (distance / self.parameters.particle_range).powi(2))
                                        .clamp(0.0, 1.0),
                                )
                                .blend(&color)
                            },
                        )
                        .apply_alpha()
                }
            })
            .into()
    }

    fn animation_name(&self) -> &str {
        "Particle fire"
    }

    fn get_fps(&self) -> f64 {
        30.0
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
        self.particles
            .resize_with(self.parameters.particle_count, || None);
    }
}
