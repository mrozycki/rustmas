use animation_api::parameter_schema::{Parameter, ParameterValue, ParametersSchema};
use animation_api::Animation;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    speed_factor: f64,
}
pub struct SpeedControlled<T: Animation> {
    animation: T,
    parameters: Parameters,
}

impl<T: Animation> Animation for SpeedControlled<T> {
    fn update(&mut self, delta: f64) {
        self.animation.update(delta * self.parameters.speed_factor);
    }

    fn render(&self) -> lightfx::Frame {
        self.animation.render()
    }

    fn animation_name(&self) -> &str {
        self.animation.animation_name()
    }

    fn parameter_schema(&self) -> ParametersSchema {
        let mut parameters = vec![Parameter {
            id: "speed_factor".to_owned(),
            name: "Speed Factor".to_owned(),
            description: None,
            value: ParameterValue::Speed,
        }];
        parameters.extend(self.animation.parameter_schema().parameters);
        ParametersSchema { parameters }
    }

    fn set_parameters(
        &mut self,
        parameters: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.parameters = serde_json::from_value(parameters.clone())?;
        self.animation.set_parameters(parameters)?;

        Ok(())
    }

    fn get_parameters(&self) -> serde_json::Value {
        let mut parameters = self
            .animation
            .get_parameters()
            .as_object_mut()
            .unwrap()
            .to_owned();
        parameters.extend(
            json!(self.parameters)
                .as_object_mut()
                .cloned()
                .unwrap()
                .into_iter(),
        );
        json!(parameters)
    }

    fn get_fps(&self) -> f64 {
        (self.animation.get_fps() * self.parameters.speed_factor.abs()).clamp(0.0, 30.0)
    }
}

impl<T: Animation> SpeedControlled<T> {
    pub fn new(animation: T) -> Self {
        Self {
            animation,
            parameters: Parameters { speed_factor: 1.0 },
        }
    }
}
