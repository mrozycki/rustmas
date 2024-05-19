use std::f64::consts::PI;

use animation_api::Animation;
use animation_utils::decorators::BrightnessControlled;
use animation_utils::{to_polar, EnumSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Default, Serialize, Deserialize, EnumSchema)]
pub enum Axis {
    #[schema_variant(name = "X: Left-Right")]
    #[default]
    X,

    #[schema_variant(name = "Y: Bottom-Top")]
    Y,

    #[schema_variant(name = "Z: Front-Back")]
    Z,

    #[schema_variant(name = "R: Inside-Outside")]
    R,

    #[schema_variant(name = "Theta: Around")]
    Theta,
}

#[derive(Clone, Default, Serialize, Deserialize, EnumSchema)]
pub enum Alignment {
    #[schema_variant(name = "Before")]
    Before,

    #[schema_variant(name = "Center")]
    #[default]
    Center,

    #[schema_variant(name = "After")]
    After,
}

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Axis", enum_options)]
    axis: Axis,

    #[schema_field(name = "Band alignment", enum_options)]
    band_alignment: Alignment,

    #[schema_field(
        name = "Band size",
        description = "Thickness of the sweep band",
        number(min = 0.0, max = 2.0, step = 0.01)
    )]
    band_size: f64,

    #[schema_field(
        name = "Band position",
        description = "Position of the start (left, bottom, front) of the band",
        number(min = "-1.0", max = 1.0, step = 0.01)
    )]
    band_position: f64,

    #[schema_field(name = "Color", color)]
    color: lightfx::Color,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            axis: Default::default(),
            band_alignment: Default::default(),
            band_size: 0.1,
            band_position: 0.0,
            color: lightfx::Color::white(),
        }
    }
}

#[animation_utils::plugin]
pub struct ManualSweep {
    points: Vec<(f64, f64, f64)>,
    parameters: Parameters,
}

impl ManualSweep {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        BrightnessControlled::new(Self {
            points,
            parameters: Default::default(),
        })
    }
}

impl Animation for ManualSweep {
    type Parameters = Parameters;

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .map(|(x, y, z)| match self.parameters.axis {
                Axis::X => *x,
                Axis::Y => *y,
                Axis::Z => *z,
                Axis::R => to_polar((*x, *y, *z)).0,
                Axis::Theta => to_polar((*x, *y, *z)).1 / PI,
            })
            .map(|h| {
                let (start, end) = match self.parameters.band_alignment {
                    Alignment::Before => (
                        self.parameters.band_position - self.parameters.band_size,
                        self.parameters.band_position,
                    ),
                    Alignment::Center => (
                        self.parameters.band_position - self.parameters.band_size / 2.0,
                        self.parameters.band_position + self.parameters.band_size / 2.0,
                    ),
                    Alignment::After => (
                        self.parameters.band_position,
                        self.parameters.band_position + self.parameters.band_size,
                    ),
                };
                if h > start && h < end {
                    self.parameters.color
                } else {
                    lightfx::Color::black()
                }
            })
            .into()
    }

    fn get_fps(&self) -> f64 {
        0.0
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
