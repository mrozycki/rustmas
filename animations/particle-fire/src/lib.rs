use animation_api::Animation;
use animation_utils::{
    EnumSchema, Schema,
    decorators::{BrightnessControlled, SpeedControlled},
};
use lightfx::{Color, Gradient};
use rand::Rng;
use serde::{Deserialize, Serialize};

struct Particle {
    position: (f64, f64, f64),
    speed: (f64, f64, f64),
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
            position: (-1.0, -1.0, -1.0),
            speed: (0.0, 0.0, 0.01),
            decay_rate: 1.0,
            power: 0.0,
        }
    }

    fn new_random(parameters: &Parameters) -> Self {
        let decay_rate = if parameters.decay_rate_spread.abs() < 0.0001 {
            parameters.decay_rate
        } else {
            rand::rng().random_range(
                parameters.decay_rate * (1.0 - parameters.decay_rate_spread)
                    ..parameters.decay_rate * (1.0 + parameters.decay_rate_spread),
            )
        }
        .clamp(0.0, 1.0);

        let z = match parameters.dimension {
            Dimension::Dim2D => 0.0,
            Dimension::Dim3D => rand::rng().random_range(-1.0..1.0),
        };

        Self {
            position: (
                rand::rng().random_range(-1.0..1.0),
                parameters.bottom_line - parameters.particle_range,
                z,
            ),
            speed: (0.0, rand::rng().random_range(0.5..1.0), 0.0),
            decay_rate,
            power: 1.0,
        }
    }

    fn update(&mut self, delta: f64, wind: f64, wind_direction: f64) -> bool {
        let (x, y, z) = self.position;
        let (vx, vy, vz) = self.speed;
        self.position = (
            (1.0 + x + vx * delta).rem_euclid(2.0) - 1.0,
            y + vy * delta,
            (1.0 + z + vz * delta).rem_euclid(2.0) - 1.0,
        );

        let wind = if wind == 0.0 {
            0.0
        } else if wind < 0.0 {
            rand::rng().random_range(wind * 1.1..wind / 1.1)
        } else {
            rand::rng().random_range(wind / 1.1..wind * 1.1)
        };
        let (wz, wx) = wind_direction.to_radians().sin_cos();
        self.speed = (vx + wx * wind, vy, vz + wz * wind);
        self.power = (self.power - self.decay_rate * delta).clamp(0.0, 1.0);

        self.power > 0.0
    }

    fn distance_to(&self, x: f64, y: f64, z: f64) -> f64 {
        ((x - self.position.0).powi(2)
            + (y - self.position.1).powi(2)
            + (z - self.position.2).powi(2))
        .sqrt()
    }
}

#[derive(Clone, Serialize, Deserialize, EnumSchema)]
pub enum Dimension {
    #[schema_variant(name = "2D")]
    Dim2D,

    #[schema_variant(name = "3D")]
    Dim3D,
}

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Dimension", enum_options)]
    dimension: Dimension,

    #[schema_field(name = "Generation rate", number(min = 0.0, max = 500.0, step = 10.0))]
    gen_rate: usize,

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

    #[schema_field(name = "Wind direction", number(min = 0.0, max = 360.0, step = 10.0))]
    wind_direction: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            dimension: Dimension::Dim3D,
            gen_rate: 250,
            decay_rate: 0.3,
            decay_rate_spread: 0.1,
            bottom_line: -1.0,
            particle_range: 0.3,
            wind: 0.0,
            wind_direction: 0.0,
        }
    }
}

#[animation_utils::wasm_plugin]
pub struct ParticleFire {
    points: Vec<(f64, f64, f64)>,
    parameters: Parameters,
    particles: Vec<Particle>,
    to_generate: f64,
    gradient: Gradient,
}

impl Animation for ParticleFire {
    type Parameters = Parameters;
    type CustomTriggers = ();
    type Wrapped = SpeedControlled<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            points,
            parameters: Default::default(),
            particles: Vec::new(),
            to_generate: 0.0,
            gradient: Gradient::from([
                Color::rgb_unit(0.0, 0.0, 0.0), // black
                Color::rgb_unit(1.0, 0.0, 0.0), // red
                Color::rgb_unit(1.0, 0.3, 0.0), // orange
                Color::rgb_unit(1.0, 0.6, 0.0), // yellow
                Color::rgb_unit(1.0, 0.6, 0.0), // yellow
            ]),
        }
    }

    fn update(&mut self, delta: f64) {
        let wind_direction = match self.parameters.dimension {
            Dimension::Dim2D => 0.0,
            Dimension::Dim3D => self.parameters.wind_direction,
        };

        self.particles
            .retain_mut(|p| p.update(delta, self.parameters.wind, wind_direction));

        self.to_generate += self.parameters.gen_rate as f64 * delta;
        if self.to_generate <= 0.0 {
            return;
        }

        let n = rand::rng()
            .random_range(0.0..2.0 * self.to_generate)
            .floor();
        if n == 0.0 {
            return;
        }

        self.to_generate -= n;
        for _ in 0..n as usize {
            self.particles.push(Particle::new_random(&self.parameters))
        }
    }

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .copied()
            .map(match self.parameters.dimension {
                Dimension::Dim2D => |(x, y, _)| (x, y, 0.0),
                Dimension::Dim3D => |p| p,
            })
            .map(|(x, y, z)| {
                if y < self.parameters.bottom_line {
                    self.gradient.at(1.0)
                } else {
                    self.particles
                        .iter()
                        .map(|p| (p.power, p.distance_to(x, y, z)))
                        .map(|(power, distance)| {
                            (
                                power,
                                1.0 - (distance / (self.parameters.particle_range * power)).powi(2),
                            )
                        })
                        .filter(|(_, alpha)| *alpha > 0.0)
                        .fold(Color::black().with_alpha(0.0), |color, (power, alpha)| {
                            self.gradient.at(power).with_alpha(alpha).blend(&color)
                        })
                        .apply_alpha()
                }
            })
            .into()
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }
}
