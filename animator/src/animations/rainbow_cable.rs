use lightfx::parameter_schema::{Parameter, ParameterValue, ParametersSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    brightness_controlled::BrightnessControlled, speed_controlled::SpeedControlled, Animation,
    AnimationParameters,
};

#[derive(Serialize, Deserialize)]
struct Parameters {
    density: f64,
}

pub struct RainbowCable {
    points_count: usize,
    parameters: Parameters,
}

impl RainbowCable {
    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        SpeedControlled::new(BrightnessControlled::new(Box::new(Self {
            points_count: points.len(),
            parameters: Parameters { density: 1.0 },
        })))
    }
}

impl Animation for RainbowCable {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        (0..self.points_count)
            .into_iter()
            .map(|i| {
                lightfx::Color::hsv(
                    i as f64 / self.points_count as f64 * self.parameters.density + time,
                    1.0,
                    1.0,
                )
            })
            .into()
    }
}

impl AnimationParameters for RainbowCable {
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
