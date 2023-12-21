use std::f64::consts::PI;

use animation_api::event::Event;
use animation_api::Animation;
use animation_utils::{
    decorators::{BrightnessControlled, SpeedControlled},
    to_polar, EnumSchema, ParameterSchema,
};
use lightfx::Color;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, EnumSchema)]
pub enum ColorScheme {
    #[schema_variant(name = "Selected color")]
    Selected,
    #[schema_variant(name = "Rainbow")]
    Rainbow,
}

#[derive(Clone, Serialize, Deserialize, EnumSchema)]
pub enum Orientation {
    #[schema_variant(name = "XY")]
    XY,
    #[schema_variant(name = "Around")]
    Around,
}

#[derive(Clone, Serialize, Deserialize, ParameterSchema)]
pub struct Parameters {
    #[schema_field(name = "Color scheme", enum_options)]
    color_scheme: ColorScheme,

    #[schema_field(name = "Selected color", color)]
    selected_color: lightfx::Color,

    #[schema_field(name = "Orientation", enum_options)]
    orientation: Orientation,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Rainbow,
            selected_color: Color::kelvin(2700),
            orientation: Orientation::Around,
        }
    }
}

#[animation_utils::plugin]
pub struct AudioVisualizer {
    points: Vec<(f64, f64, f64)>,
    bands: Vec<f32>,
    parameters: Parameters,
}

impl AudioVisualizer {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points,
            bands: vec![0.0],
            parameters: Default::default(),
        }))
    }
}

impl Animation for AudioVisualizer {
    type Parameters = Parameters;

    fn on_event(&mut self, event: Event) {
        if let Event::FftEvent { bands } = event {
            self.bands = bands;
        }
    }

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .copied()
            .map(match self.parameters.orientation {
                Orientation::XY => |(x, y, _)| (x, y),
                Orientation::Around => |p| {
                    let p = to_polar(p);
                    ((p.1 / PI).rem_euclid(2.0) - 1.0, p.2)
                },
            })
            .map(|(x, y)| {
                let band_width = 2.0 / self.bands.len() as f64;
                let band_index =
                    ((x + 1.0).div_euclid(band_width) as usize).clamp(0, self.bands.len() - 1);
                let band_value = self.bands[band_index].clamp(0.0, 1.0);
                let y = (y as f32 + 1.0) / 2.0;
                if y < band_value {
                    match self.parameters.color_scheme {
                        ColorScheme::Selected => self.parameters.selected_color,
                        ColorScheme::Rainbow => Color::hsv((x + 1.0) / 2.0, 1.0, 1.0),
                    }
                } else {
                    Color::black()
                }
            })
            .into()
    }

    fn animation_name(&self) -> &str {
        "Audio Visualizer"
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters
    where
        Self::Parameters: Default,
    {
        self.parameters.clone()
    }
}
