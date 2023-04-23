use animation_api::parameter_schema::{Parameter, ParameterValue, ParametersSchema};
use animation_api::{Animation, AnimationParameters};
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    density: f64,
}

#[animation_utils::plugin]
pub struct RainbowCable {
    points_count: usize,
    time: f64,
    parameters: Parameters,
}

impl RainbowCable {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points_count: points.len(),
            time: 0.0,
            parameters: Parameters { density: 1.0 },
        }))
    }
}

impl Animation for RainbowCable {
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        (0..self.points_count)
            .map(|i| {
                lightfx::Color::hsv(
                    i as f64 / self.points_count as f64 * self.parameters.density + self.time,
                    1.0,
                    1.0,
                )
            })
            .into()
    }
}

impl AnimationParameters for RainbowCable {
    fn animation_name(&self) -> &str {
        "Rainbow Cable"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![Parameter {
                id: "density".to_owned(),
                name: "Density".to_owned(),
                description: None,
                value: ParameterValue::Number {
                    min: 0.5,
                    max: 5.0,
                    step: 0.05,
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
