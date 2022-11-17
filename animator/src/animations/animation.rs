use std::error::Error;

use lightfx::schema::ParametersSchema;
use serde_json::json;

pub trait Animation {
    fn frame(&mut self, time: f64) -> lightfx::Frame;

    fn parameter_schema(&self) -> ParametersSchema {
        Default::default()
    }

    fn set_parameters(&mut self, _parameters: serde_json::Value) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn get_parameters(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        Ok(json!({}))
    }
}

pub fn make_animation(
    name: &str,
    points: &Vec<(f64, f64, f64)>,
) -> Box<dyn Animation + Sync + Send> {
    match name {
        "barber_pole" => Box::new(super::barber_pole::BarberPole::new(points)),
        "blank" => Box::new(super::blank::Blank::new(points)),
        "check" => Box::new(super::check::Check::new(points)),
        "rainbow_cable" => Box::new(super::rainbow_cable::RainbowCable::new(points)),
        "rainbow_cylinder" => Box::new(super::rainbow_cylinder::RainbowCylinder::new(points)),
        "rainbow_sphere" => Box::new(super::rainbow_sphere::RainbowSphere::new(points)),
        "rainbow_spiral" => Box::new(super::rainbow_spiral::RainbowSpiral::new(points)),
        "rainbow_waterfall" => Box::new(super::rainbow_waterfall::RainbowWaterfall::new(points)),
        "random_sweep" => Box::new(super::random_sweep::RandomSweep::new(points)),
        "sweep" => Box::new(super::sweep::Sweep::new(points)),
        "rgb" => Box::new(super::rgb::Rgb::new(points)),
        _ => panic!("Unknown animation pattern \"{}\"", name),
    }
}
