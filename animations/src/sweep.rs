use animation_api::{Animation, AnimationParameters};
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use lightfx::schema::{EnumOption, Parameter, ParameterValue, ParametersSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
enum Direction {
    BottomToTop,
    TopToBottom,
    BackToFront,
    FrontToBack,
    LeftToRight,
    RightToLeft,
}

#[derive(Serialize, Deserialize)]
struct Parameters {
    direction: Direction,
    band_size: f64,
    color: lightfx::Color,
}

#[animation_utils::plugin]
pub struct Sweep {
    points: Vec<(f64, f64, f64)>,
    time: f64,
    parameters: Parameters,
}

impl Sweep {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points,
            time: 0.0,
            parameters: Parameters {
                direction: Direction::BottomToTop,
                band_size: 0.2,
                color: lightfx::Color::white(),
            },
        }))
    }
}

impl Animation for Sweep {
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        let time =
            self.time % (2.0 + self.parameters.band_size) - (1.0 + self.parameters.band_size / 2.0);
        self.points
            .iter()
            .map(|(x, y, z)| match self.parameters.direction {
                Direction::BottomToTop => *y,
                Direction::TopToBottom => -*y,
                Direction::BackToFront => -*z,
                Direction::FrontToBack => *z,
                Direction::LeftToRight => *x,
                Direction::RightToLeft => -*x,
            })
            .map(|h| {
                if h > time && h < time + self.parameters.band_size {
                    self.parameters.color
                } else {
                    lightfx::Color::black()
                }
            })
            .into()
    }
}

impl AnimationParameters for Sweep {
    fn animation_name(&self) -> &str {
        "Sweep"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![
                Parameter {
                    id: "direction".to_owned(),
                    name: "Direction".to_owned(),
                    description: Some("Direction of the sweep".to_owned()),
                    value: ParameterValue::Enum {
                        values: vec![
                            EnumOption {
                                name: "Bottom to top".to_owned(),
                                description: None,
                                value: "BottomToTop".to_owned(),
                            },
                            EnumOption {
                                name: "Top to bottom".to_owned(),
                                description: None,
                                value: "TopToBottom".to_owned(),
                            },
                            EnumOption {
                                name: "Back to front".to_owned(),
                                description: None,
                                value: "BackToFront".to_owned(),
                            },
                            EnumOption {
                                name: "Front to back".to_owned(),
                                description: None,
                                value: "FrontToBack".to_owned(),
                            },
                            EnumOption {
                                name: "Left to right".to_owned(),
                                description: None,
                                value: "LeftToRight".to_owned(),
                            },
                            EnumOption {
                                name: "Right to left".to_owned(),
                                description: None,
                                value: "RightToLeft".to_owned(),
                            },
                        ],
                    },
                },
                Parameter {
                    id: "band_size".to_owned(),
                    name: "Band size".to_owned(),
                    description: Some("Thickness of the sweep band".to_owned()),
                    value: ParameterValue::Number {
                        min: 0.0,
                        max: 2.0,
                        step: 0.05,
                    },
                },
                Parameter {
                    id: "color".to_owned(),
                    name: "Color".to_owned(),
                    description: Some("Color of the sweep band".to_owned()),
                    value: ParameterValue::Color,
                },
            ],
        }
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
