use animation_api::Animation;
use animation_utils::{
    decorators::{BrightnessControlled, OffSwitch},
    Schema,
};
use lightfx::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Color", color)]
    color: Color,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            color: Color::white(),
        }
    }
}

#[animation_utils::plugin]
pub struct SingleColor {
    points_count: usize,
    parameters: Parameters,
}

impl SingleColor {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        OffSwitch::new(BrightnessControlled::new(Self {
            points_count: points.len(),
            parameters: Default::default(),
        }))
    }
}

impl Animation for SingleColor {
    type Parameters = Parameters;

    fn render(&self) -> lightfx::Frame {
        lightfx::Frame::new(self.points_count, self.parameters.color)
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
