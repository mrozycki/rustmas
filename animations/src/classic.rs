use std::f64::consts::{FRAC_PI_2, PI};

use animation_api::parameter_schema::{EnumOption, Parameter, ParameterValue, ParametersSchema};
use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use itertools::Itertools;
use lightfx::Color;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
enum Mode {
    FlowingSingles,
    FlowingPairs,
    Static,
}

#[derive(Serialize, Deserialize)]
struct Parameters {
    color_red: Color,
    color_green: Color,
    color_yellow: Color,
    color_blue: Color,
    mode: Mode,
}

#[animation_utils::plugin]
pub struct Classic {
    points_count: usize,
    time: f64,
    parameters: Parameters,
}

impl Classic {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points_count: points.len(),
            time: 0.0,
            parameters: Parameters {
                color_red: lightfx::Color::rgb(255, 0, 0),
                color_green: lightfx::Color::rgb(0, 255, 0),
                color_yellow: lightfx::Color::rgb(255, 160, 0),
                color_blue: lightfx::Color::rgb(0, 0, 255),
                mode: Mode::Static,
            },
        }))
    }
}

impl Animation for Classic {
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        let base = (0..self.points_count)
            .map(|i| match i % 4 {
                0 => self.parameters.color_red,
                1 => self.parameters.color_green,
                2 => self.parameters.color_yellow,
                _ => self.parameters.color_blue,
            })
            .collect_vec();

        let mask = (0..self.points_count).map(|i| match self.parameters.mode {
            Mode::FlowingSingles => ((self.time / 4.0).fract() * 2.0 * PI
                + (i % 4) as f64 * FRAC_PI_2)
                .sin()
                .clamp(0.0, 1.0),
            Mode::FlowingPairs => ((self.time / 4.0).fract() * 2.0 * PI
                + (i % 2) as f64 * FRAC_PI_2)
                .sin()
                .abs(),
            Mode::Static => 1.0,
        });

        base.into_iter()
            .zip(mask)
            .map(|(color, level)| color.dim(level))
            .into()
    }

    fn animation_name(&self) -> &str {
        "Classic"
    }

    fn get_fps(&self) -> f64 {
        30.0
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![
                Parameter {
                    id: "color_red".to_owned(),
                    name: "Color A (Red)".to_owned(),
                    description: None,
                    value: ParameterValue::Color,
                },
                Parameter {
                    id: "color_green".to_owned(),
                    name: "Color B (Green)".to_owned(),
                    description: None,
                    value: ParameterValue::Color,
                },
                Parameter {
                    id: "color_yellow".to_owned(),
                    name: "Color C (Yellow)".to_owned(),
                    description: None,
                    value: ParameterValue::Color,
                },
                Parameter {
                    id: "color_blue".to_owned(),
                    name: "Color D (Blue)".to_owned(),
                    description: None,
                    value: ParameterValue::Color,
                },
                Parameter {
                    id: "mode".to_owned(),
                    name: "Mode".to_owned(),
                    description: None,
                    value: ParameterValue::Enum {
                        values: vec![
                            EnumOption {
                                name: "Flowing one-by-one".into(),
                                description: None,
                                value: "FlowingSingles".into(),
                            },
                            EnumOption {
                                name: "Flowing two-by-two".into(),
                                description: None,
                                value: "FlowingPairs".into(),
                            },
                            EnumOption {
                                name: "Static".into(),
                                description: None,
                                value: "Static".into(),
                            },
                        ],
                    },
                },
            ],
        }
    }

    fn get_parameters(&self) -> serde_json::Value {
        json!(self.parameters)
    }

    fn set_parameters(
        &mut self,
        parameters: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.parameters = serde_json::from_value(parameters)?;
        Ok(())
    }
}
