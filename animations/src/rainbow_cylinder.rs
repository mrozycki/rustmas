use animation_api::Animation;
use animation_api::AnimationParameters;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use lightfx::parameter_schema::Parameter;
use lightfx::parameter_schema::ParameterValue;
use lightfx::parameter_schema::ParametersSchema;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    density: f64,
}

#[animation_utils::plugin]
pub struct RainbowCylinder {
    points_alpha: Vec<f64>,
    time: f64,
    parameters: Parameters,
}

impl RainbowCylinder {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points_alpha: points
                .into_iter()
                .map(animation_utils::to_polar)
                .map(|(_, a, _)| a)
                .collect(),
            time: 0.0,
            parameters: Parameters { density: 1.0 },
        }))
    }
}

impl Animation for RainbowCylinder {
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    fn render(&self) -> lightfx::Frame {
        self.points_alpha
            .iter()
            .map(|a| {
                lightfx::Color::hsv(
                    self.time + a / (2.0 * std::f64::consts::PI) * self.parameters.density,
                    1.0,
                    1.0,
                )
            })
            .into()
    }
}

impl AnimationParameters for RainbowCylinder {
    fn animation_name(&self) -> &str {
        "Rainbow Cylinder"
    }

    fn parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: vec![Parameter {
                id: "density".to_owned(),
                name: "Density".to_owned(),
                description: None,
                value: ParameterValue::Number {
                    min: 1.0,
                    max: 5.0,
                    step: 1.0,
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
