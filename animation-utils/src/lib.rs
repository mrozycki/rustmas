pub mod decorators;

use std::f64::consts::TAU;

pub use animation_macros::{EnumSchema, Schema, plugin, wasm_plugin};
use nalgebra::{Rotation3, Unit, Vector3};
use rand::Rng;

pub fn to_polar((x, y, z): (f64, f64, f64)) -> (f64, f64, f64) {
    ((x.powi(2) + z.powi(2)).sqrt(), x.atan2(z), y)
}

pub fn random_component() -> f64 {
    rand::rng().random::<f64>().fract() * 2.0 - 1.0
}

pub fn random_rotation() -> Rotation3<f64> {
    let (mut x, mut y, mut z): (f64, f64, f64) = (-1.0, -1.0, -1.0);

    while (x.powi(2) + y.powi(2) + z.powi(2)).sqrt() > 1.0 {
        (x, y, z) = (random_component(), random_component(), random_component());
    }

    Rotation3::rotation_between(&Vector3::new(0.0, 1.0, 0.0), &Vector3::new(x, y, z)).unwrap()
}

pub fn random_rotation_around(axis: &Unit<Vector3<f64>>) -> Rotation3<f64> {
    Rotation3::from_axis_angle(axis, rand::rng().random_range(0.0..TAU))
}

pub fn random_hue(saturation: f64, value: f64) -> lightfx::Color {
    lightfx::Color::hsv(rand::rng().random::<f64>() % 1.0, saturation, value)
}
