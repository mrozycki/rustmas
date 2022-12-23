use std::error::Error;

use lightfx::schema::ParametersSchema;
use serde_json::json;

pub trait AnimationParameters {
    fn animation_name(&self) -> &str;

    fn parameter_schema(&self) -> ParametersSchema {
        Default::default()
    }

    fn set_parameters(&mut self, _parameters: serde_json::Value) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn get_parameters(&self) -> serde_json::Value {
        json!({})
    }

    fn get_fps(&self) -> f64 {
        30.0
    }
}

pub trait Animation: AnimationParameters + Sync + Send {
    fn frame(&mut self, time: f64) -> lightfx::Frame;
}

pub trait StepAnimation: AnimationParameters + Sync + Send {
    fn update(&mut self, delta: f64);
    fn render(&self) -> lightfx::Frame;
}

pub struct StepAnimationDecorator {
    last_time: f64,
    step_animation: Box<dyn StepAnimation>,
}

impl StepAnimationDecorator {
    pub fn new(step_animation: Box<dyn StepAnimation>) -> Box<dyn Animation> {
        Box::new(Self {
            last_time: 0.0,
            step_animation,
        })
    }
}

impl Animation for StepAnimationDecorator {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        let delta = time - self.last_time;
        self.last_time = time;
        self.step_animation.update(delta);
        self.step_animation.render()
    }
}

impl AnimationParameters for StepAnimationDecorator {
    fn animation_name(&self) -> &str {
        self.step_animation.animation_name()
    }

    fn parameter_schema(&self) -> ParametersSchema {
        self.step_animation.parameter_schema()
    }

    fn set_parameters(&mut self, parameters: serde_json::Value) -> Result<(), Box<dyn Error>> {
        self.step_animation.set_parameters(parameters)
    }

    fn get_parameters(&self) -> serde_json::Value {
        self.step_animation.get_parameters()
    }

    fn get_fps(&self) -> f64 {
        self.step_animation.get_fps()
    }
}

pub fn make_animation(name: &str, points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
    match name {
        "blank" => super::blank::Blank::new(points),
        "barber_pole" => super::barber_pole::BarberPole::new(points),
        "classic" => super::classic::Classic::new(points),
        "rainbow_cable" => super::rainbow_cable::RainbowCable::new(points),
        "rainbow_cylinder" => super::rainbow_cylinder::RainbowCylinder::new(points),
        "rainbow_halves" => super::rainbow_halves::RainbowHalves::new(points),
        "rainbow_sphere" => super::rainbow_sphere::RainbowSphere::new(points),
        "rainbow_spiral" => super::rainbow_spiral::RainbowSpiral::new(points),
        "rainbow_waterfall" => super::rainbow_waterfall::RainbowWaterfall::new(points),
        "random_sweep" => super::random_sweep::RandomSweep::new(points),
        "present" => super::present::Present::new(points),
        "spinning_halves" => super::spinning_halves::SpinningHalves::new(points),
        "test_check" => super::check::Check::new(points),
        "test_sweep" => super::sweep::Sweep::new(points),
        "test_manual_sweep" => super::manual_sweep::ManualSweep::new(points),
        "test_indexing" => super::indexing::Indexing::new(points),
        "test_detection_status" => super::detection_status::DetectionStatus::new(points),
        _ => panic!("Unknown animation pattern \"{}\"", name),
    }
}
