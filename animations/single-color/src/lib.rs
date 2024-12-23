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

#[animation_utils::wasm_plugin]
pub struct SingleColor {
    points_count: usize,
    parameters: Parameters,
}

impl Animation for SingleColor {
    type Parameters = Parameters;
    type CustomTriggers = ();
    type Wrapped = OffSwitch<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_count: points.len(),
            parameters: Default::default(),
        }
    }

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
