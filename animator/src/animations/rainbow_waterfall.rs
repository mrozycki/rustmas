use super::{
    brightness_controlled::BrightnessControlled, speed_controlled::SpeedControlled, Animation,
    AnimationParameters,
};
use lightfx::schema::{Parameter, ParameterValue, ParametersSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    density: f64,
}

pub struct RainbowWaterfall {
    points_height: Vec<f64>,
    parameters: Parameters,
}

impl RainbowWaterfall {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        SpeedControlled::new(BrightnessControlled::new(Box::new(Self {
            parameters: Parameters { density: 1.0 },
            points_height: points.iter().map(|(_, h, _)| (h + 1.0) / 2.0).collect(),
        })))
    }
}

impl Animation for RainbowWaterfall {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        self.points_height
            .iter()
            .map(|h| lightfx::Color::hsv(h * self.parameters.density + time, 1.0, 1.0))
            .into()
    }
}

impl AnimationParameters for RainbowWaterfall {
    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![Parameter {
                id: "density".to_owned(),
                name: "Density".to_owned(),
                description: None,
                value: ParameterValue::Number {
                    min: Some(0.5),
                    max: Some(5.0),
                },
            }],
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
