use animation_api::Animation;
use animation_utils::{
    decorators::{BrightnessControlled, SpeedControlled},
    Schema,
};
use lightfx::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Color", color)]
    color: Color,
}

#[animation_utils::plugin]
pub struct ColorPicker {
    points_count: usize,
    parameters: Parameters,
}

impl Animation for ColorPicker {
    type Parameters = ();
    type Wrapped = SpeedControlled<BrightnessControlled<Self>>;

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
        0.0
    }
}
