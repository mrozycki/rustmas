use animation_api::Animation;
use animation_utils::{
    decorators::{BrightnessControlled, SpeedControlled},
    ParameterSchema,
};
use lightfx::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, ParameterSchema)]
pub struct Parameters {
    #[schema_field(name = "Color", color)]
    color: Color,
}

#[animation_utils::plugin]
pub struct ColorPicker {
    points_count: usize,
    parameters: Parameters,
}

impl ColorPicker {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points_count: points.len(),
            parameters: Default::default(),
        }))
    }
}

impl Animation for ColorPicker {
    type Parameters = ();

    fn render(&self) -> lightfx::Frame {
        lightfx::Frame::new(self.points_count, self.parameters.color)
    }

    fn animation_name(&self) -> &str {
        "Testing: Color picker"
    }

    fn get_fps(&self) -> f64 {
        0.0
    }
}
