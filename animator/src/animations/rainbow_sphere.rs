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
    height: f64,
}

pub struct RainbowSphere {
    points: Vec<(f64, f64, f64)>,
    points_radius: Vec<f64>,
    parameters: Parameters,
}

impl RainbowSphere {
    fn reset(&mut self) {
        self.points_radius = self
            .points
            .iter()
            .map(|(x, y, z)| (x.powi(2) + (y - self.parameters.height).powi(2) + z.powi(2)).sqrt())
            .collect();
    }

    pub fn new(points: &Vec<(f64, f64, f64)>) -> Box<dyn Animation> {
        let mut result = Self {
            points_radius: vec![],
            points: points.clone(),
            parameters: Parameters {
                density: 1.0,
                height: 0.0,
            },
        };
        result.reset();
        SpeedControlled::new(BrightnessControlled::new(Box::new(result)))
    }
}

impl Animation for RainbowSphere {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        self.points_radius
            .iter()
            .map(|r| lightfx::Color::hsv(r / 2.0 * self.parameters.density + time, 1.0, 1.0))
            .into()
    }
}

impl AnimationParameters for RainbowSphere {
    fn animation_name(&self) -> &str {
        "Rainbow Sphere"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![
                Parameter {
                    id: "density".to_owned(),
                    name: "Density".to_owned(),
                    description: None,
                    value: ParameterValue::Number {
                        min: 0.5,
                        max: 5.0,
                        step: 0.05,
                    },
                },
                Parameter {
                    id: "height".to_owned(),
                    name: "Height".to_owned(),
                    description: Some("Height of the center of the sphere".to_owned()),
                    value: ParameterValue::Number {
                        min: -1.0,
                        max: 1.0,
                        step: 0.05,
                    },
                },
            ],
        }
    }

    fn set_parameters(
        &mut self,
        parameters: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.parameters = serde_json::from_value(parameters)?;
        self.reset();
        Ok(())
    }

    fn get_parameters(&self) -> serde_json::Value {
        json!(self.parameters)
    }
}
