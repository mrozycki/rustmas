use animation_api::Animation;
use animation_utils::{
    decorators::{BrightnessControlled, SpeedControlled},
    EnumSchema, ParameterSchema,
};
use lightfx::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, EnumSchema, PartialEq)]
pub enum Switch {
    #[schema_variant(name = "On")]
    #[default]
    On,

    #[schema_variant(name = "Off")]
    Off,
}

#[derive(Clone, Serialize, Deserialize, ParameterSchema)]
pub struct Parameters {
    #[schema_field(name = "Color", color)]
    color: Color,

    #[schema_field(name = "Switch delay", number(min = 0.0, max = 1.0, step = 0.05))]
    switch_delay: f64,

    #[schema_field(name = "Status", enum_options)]
    status: Switch,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            color: Color::white(),
            switch_delay: 1.0,
            status: Switch::On,
        }
    }
}

#[animation_utils::plugin]
pub struct SingleColor {
    points_count: usize,
    energy: f64,
    parameters: Parameters,
}

impl SingleColor {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points_count: points.len(),
            energy: 1.0,
            parameters: Default::default(),
        }))
    }
}

impl Animation for SingleColor {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        let delta = if self.parameters.switch_delay == 0.0 {
            1.0
        } else {
            delta / self.parameters.switch_delay
        };
        self.energy = match self.parameters.status {
            Switch::Off => self.energy - delta,
            Switch::On => self.energy + delta,
        }
        .clamp(0.0, 1.0);
    }

    fn render(&self) -> lightfx::Frame {
        lightfx::Frame::new(self.points_count, self.parameters.color.dim(self.energy))
    }

    fn animation_name(&self) -> &str {
        "Single Color"
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
