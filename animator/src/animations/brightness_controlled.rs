use super::{Animation, AnimationParameters};
use lightfx::{parameter_schema::Parameter, schema::ParametersSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    brightness_factor: f64,
}
pub struct BrightnessControlled {
    animation: Box<dyn Animation>,
    parameters: Parameters,
}

impl Animation for BrightnessControlled {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        self.animation
            .frame(time)
            .pixels_iter()
            .map(|x| x.dim(self.parameters.brightness_factor))
            .into()
    }
}

impl AnimationParameters for BrightnessControlled {
    fn parameter_schema(&self) -> ParametersSchema {
        let mut parameters = vec![Parameter {
            id: "brightness_factor".to_owned(),
            name: "Brightness".to_owned(),
            description: None,
            value: lightfx::parameter_schema::ParameterValue::Percentage,
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
            json!(&self.parameters)
                .as_object_mut()
                .cloned()
                .unwrap()
                .into_iter(),
        );
        json!(parameters)
    }

    fn get_fps(&self) -> f64 {
        self.animation.get_fps()
    }
}

impl BrightnessControlled {
    pub fn new(animation: Box<dyn Animation>) -> Box<dyn Animation> {
        Box::new(Self {
            animation,
            parameters: Parameters {
                brightness_factor: 1.0,
            },
        })
    }
}
