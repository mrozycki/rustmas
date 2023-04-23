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
pub struct RainbowWaterfall {
    points_height: Vec<f64>,
    time: f64,
    parameters: Parameters,
}

impl RainbowWaterfall {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            parameters: Parameters { density: 1.0 },
            time: 0.0,
            points_height: points
                .into_iter()
                .map(|(_, h, _)| (h + 1.0) / 2.0)
                .collect(),
        }))
    }
}

impl Animation for RainbowWaterfall {
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        self.points_height
            .iter()
            .map(|h| lightfx::Color::hsv(h * self.parameters.density + self.time, 1.0, 1.0))
            .into()
    }
}

impl AnimationParameters for RainbowWaterfall {
    fn animation_name(&self) -> &str {
        "Rainbow Waterfall"
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
