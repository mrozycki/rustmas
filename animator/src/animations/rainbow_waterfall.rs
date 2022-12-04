use super::Animation;
use lightfx::schema::{Parameter, ParameterValue, ParametersSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    cycles: f64,
}

pub struct RainbowWaterfall {
    points_height: Vec<f64>,
    parameters: Parameters,
}

impl RainbowWaterfall {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        Self {
            parameters: Parameters { cycles: 2.0 },
            points_height: points.iter().map(|(_, h, _)| (h + 1.0) / 2.0).collect(),
        }
    }
}

impl Animation for RainbowWaterfall {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        self.points_height
            .iter()
            .map(|h| lightfx::Color::hsv(h * self.parameters.cycles + time, 1.0, 0.5))
            .into()
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![
                Parameter {
                id: "cycles".to_owned(),
                    name: "Number of cycles".to_owned(),
                    description: Some("Number of color cycles that will be present on the tree at the same time. Does not have to be a whole number".to_owned()),
                    value: ParameterValue::Number { min: Some(0.0), max: Some(10.0) },
                },
            ]
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
