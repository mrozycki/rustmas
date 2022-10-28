use rustmas_light_client as client;

pub trait Animation {
    fn frame(&self, time: f64) -> client::Frame;
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
        "sweep" => Box::new(super::sweep::Sweep::new(points)),
        "rgb" => Box::new(super::rgb::Rgb::new(points)),
        _ => panic!("Unknown animation pattern \"{}\"", name),
    }
}
