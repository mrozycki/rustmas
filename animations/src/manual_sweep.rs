use animation_api::parameter_schema::{get_schema, ParametersSchema};
use animation_api::Animation;
use animation_utils::decorators::BrightnessControlled;
use animation_utils::{EnumSchema, ParameterSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, EnumSchema)]
enum Axis {
    #[schema_variant(name = "X: Left-Right")]
    X,

    #[schema_variant(name = "Y: Bottom-Top")]
    Y,

    #[schema_variant(name = "Z: Front-Back")]
    Z,
}

#[derive(Serialize, Deserialize, EnumSchema)]
enum Alignment {
    #[schema_variant(name = "Before")]
    Before,

    #[schema_variant(name = "Center")]
    Center,

    #[schema_variant(name = "After")]
    After,
}

#[derive(Serialize, Deserialize, ParameterSchema)]
struct Parameters {
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

#[animation_utils::plugin]
pub struct ManualSweep {
    points: Vec<(f64, f64, f64)>,
    parameters: Parameters,
}

impl ManualSweep {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        BrightnessControlled::new(Self {
            points,
            parameters: Parameters {
                axis: Axis::Y,
                band_alignment: Alignment::Center,
                band_size: 0.1,
                band_position: 0.0,
                color: lightfx::Color::white(),
            },
        })
    }
}

impl Animation for ManualSweep {
    fn update(&mut self, _delta: f64) {}

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .map(|(x, y, z)| match self.parameters.axis {
                Axis::X => *x,
                Axis::Y => *y,
                Axis::Z => *z,
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

    fn animation_name(&self) -> &str {
        "Testing: Manual sweep"
    }

    fn get_fps(&self) -> f64 {
        0.0
    }

    fn parameter_schema(&self) -> ParametersSchema {
        get_schema::<Parameters>()
    }

    fn set_parameters(
        &mut self,
        parameters: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.parameters = serde_json::from_value(parameters)?;
        Ok(())
    }

    fn get_parameters(&self) -> serde_json::Value {
        json!(self.parameters)
    }
}
