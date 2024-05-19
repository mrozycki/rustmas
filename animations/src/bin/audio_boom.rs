use std::f64::consts::PI;

use animation_api::event::Event;
use animation_api::Animation;
use animation_utils::{
    decorators::{BrightnessControlled, SpeedControlled},
    to_polar, EnumSchema, Schema,
};
use itertools::Itertools;
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

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Color scheme", enum_options)]
    color_scheme: ColorScheme,

    #[schema_field(name = "Selected color", color)]
    selected_color: lightfx::Color,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Rainbow,
            selected_color: Color::kelvin(2700),
        }
    }
}

#[animation_utils::plugin]
pub struct AudioVisualizer {
    points: Vec<(usize, (f64, f64))>,
    time: f64,
    control_points: Vec<(f64, f32, f32)>,
    parameters: Parameters,
}

impl AudioVisualizer {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points: points
                .into_iter()
                .map(|p| {
                    let (_, a, h) = to_polar(p);
                    ((a / PI + 1.0) / 2.0, h.abs())
                })
                .enumerate()
                .sorted_by(|(_, (_, r0)), (_, (_, r1))| r0.total_cmp(r1))
                .collect(),
            time: 0.0,
            control_points: Vec::new(),
            parameters: Default::default(),
        }))
    }
}

impl Animation for AudioVisualizer {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        self.time += delta;

        self.control_points.retain_mut(|(r, _, _)| {
            *r += delta;
            *r < 1.0
        });
    }

    fn on_event(&mut self, event: Event) {
        if let Event::FftEvent { bands } = event {
            let new_point = bands.iter().sum();
            let new_max = self
                .control_points
                .iter()
                .map(|(_, level, _)| *level)
                .chain([new_point])
                .fold(0.0, |acc, next| if acc > next { acc } else { next })
                .clamp(0.5, 100.0);

            self.control_points.push((0.0, new_point, new_max));
        }
    }

    fn render(&self) -> lightfx::Frame {
        let mut control_points_iter = self.control_points.iter().rev();
        let mut control_point = control_points_iter.next();

        self.points
            .iter()
            .map(|(i, (a, h))| {
                while control_point.is_some_and(|(x, _, _)| x < h) {
                    control_point = control_points_iter.next();
                }
                (
                    i,
                    control_point
                        .map(|(_, level, max_level)| {
                            match self.parameters.color_scheme {
                                ColorScheme::Selected => self.parameters.selected_color,
                                ColorScheme::Rainbow => lightfx::Color::hsv(*a, 1.0, 1.0),
                            }
                            .dim((*level / max_level).clamp(0.0, 1.0).powi(2) as f64)
                        })
                        .unwrap_or(lightfx::Color::black()),
                )
            })
            .sorted_by(|(i0, _), (i1, _)| i0.cmp(i1))
            .map(|(_, c)| c)
            .into()
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
