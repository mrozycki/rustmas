use super::Animation;
use rustmas_animation_model::schema::{EnumOption, Parameter, ParameterValue, ParametersSchema};
use rustmas_light_client as client;
use serde::{Deserialize, Serialize};

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
    color: client::Color,
}

pub struct Sweep {
    points: Vec<(f64, f64, f64)>,
    parameters: Parameters,
}

impl Sweep {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            points: points.clone(),
            parameters: Parameters {
                direction: Direction::BottomToTop,
                band_size: 0.2,
                color: client::Color::white(),
            },
        }
    }
}

impl Animation for Sweep {
    fn frame(&mut self, time: f64) -> client::Frame {
        let time =
            time % (2.0 + self.parameters.band_size) - (1.0 + self.parameters.band_size / 2.0);
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
                    client::Color::black()
                }
            })
            .into()
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
                        min: Some(0.0),
                        max: Some(2.0),
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

    fn get_parameters(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        Ok(serde_json::to_value(&self.parameters)?)
    }
}
