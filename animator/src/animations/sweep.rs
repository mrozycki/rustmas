use super::Animation;
use rustmas_light_client as client;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
enum Direction {
    BottomToTop,
    TopToBottom,
    BackToFront,
    FrontToBack,
    LeftToRight,
    RightToLeft,
}

#[derive(Deserialize)]
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

    fn parameter_schema(&self) -> serde_json::Value {
        json!({
            "direction": {
                "type": "enum",
                "values": [
                    "BottomToTop",
                    "TopToBottom",
                    "BackToFront",
                    "FrontToBack",
                    "LeftToRight",
                    "RightToLeft",
                ]
            },
            "band_size": {
                "type": "number"
            },
            "color": {
                "type": "color"
            }
        })
    }

    fn set_parameters(
        &mut self,
        parameters: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.parameters = serde_json::from_value(parameters)?;
        Ok(())
    }
}
