use std::f64::consts::TAU;

use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, OffSwitch};
use animation_utils::Schema;
use bracket_noise::prelude::FastNoise;
use lightfx::Color;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Radius", number(min = 0.0, max = 1.0, step = 0.05))]
    radius: f64,

    #[schema_field(name = "Light distance", number(min = 10.0, max = 100.0, step = 10.0))]
    light_distance: f64,

    #[schema_field(name = "Cycle length", number(min = 10.0, max = 120.0, step = 5.0))]
    cycle_length: f64,

    #[schema_field(name = "Halo brightness", number(min = 0.0, max = 1.0, step = 0.05))]
    halo_brightness: f64,

    #[schema_field(name = "Noise frequency", number(min = 1.0, max = 10.0, step = 0.5))]
    noise_frequency: f64,

    #[schema_field(name = "Noise amplitude", number(min = 0.0, max = 0.5, step = 0.01))]
    noise_amplitude: f64,

    #[schema_field(name = "Color", color)]
    color: Color,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            radius: 0.5,
            light_distance: 10.0,
            cycle_length: 10.0,
            halo_brightness: 0.7,
            noise_frequency: 3.0,
            noise_amplitude: 1.0,
            color: Color::white(),
        }
    }
}

#[animation_utils::plugin]
pub struct Moon {
    points: Vec<(f64, f64, f64)>,
    time: f64,
    noise: FastNoise,
    parameters: Parameters,
}

impl Moon {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        let mut noise = FastNoise::new();
        noise.set_frequency(2.0);
        OffSwitch::new(BrightnessControlled::new(Self {
            points,
            time: 0.25,
            noise,
            parameters: Default::default(),
        }))
    }
}

impl Animation for Moon {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        self.time += delta / self.parameters.cycle_length;
    }

    fn render(&self) -> lightfx::Frame {
        let t = self.time * TAU;
        let to_sun = Vector3::new(t.sin(), 0.0, t.cos());

        self.points
            .iter()
            .map(|(x, y, _z)| {
                let d = (x.powi(2) + y.powi(2)).sqrt();
                if d < self.parameters.radius {
                    let n = Vector3::new(
                        *x,
                        *y,
                        (-(x.powi(2) + y.powi(2) - self.parameters.radius)).sqrt(),
                    )
                    .normalize();
                    let noise = self.noise.get_noise(*x as f32, *y as f32) as f64 / 2.0
                        * self.parameters.noise_amplitude
                        + (1.0 - self.parameters.noise_amplitude / 2.0);
                    self.parameters.color.dim(n.dot(&to_sun) * noise)
                } else {
                    let sun_strength = (-t.cos() + 1.5) / 2.5;
                    let halo_strength = self.parameters.halo_brightness
                        * (((self.parameters.radius - d) * 4.0).exp() * 0.8
                            + (self.parameters.radius - d).exp() * 0.2);
                    self.parameters.color.dim(halo_strength * sun_strength)
                }
            })
            .into()
    }

    fn animation_name(&self) -> &str {
        "Moon"
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.noise.set_frequency(parameters.noise_frequency as f32);
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
