use animation_api::{Animation, AnimationParameters};
use lightfx::{parameter_schema::Parameter, schema::ParametersSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    speed_factor: f64,
}
pub struct SpeedControlled {
    animation: Box<dyn Animation>,
    parameters: Parameters,
    reference_real_time: f64,
    reference_fake_time: f64,
}

impl Animation for SpeedControlled {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        self.reference_fake_time +=
            (time - self.reference_real_time) * self.parameters.speed_factor;
        self.reference_real_time = time;
        self.animation.frame(self.reference_fake_time)
    }
}

impl AnimationParameters for SpeedControlled {
    fn animation_name(&self) -> &str {
        self.animation.animation_name()
    }

    fn parameter_schema(&self) -> ParametersSchema {
        let mut parameters = vec![Parameter {
            id: "speed_factor".to_owned(),
            name: "Speed Factor".to_owned(),
            description: None,
            value: lightfx::parameter_schema::ParameterValue::Speed,
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

impl SpeedControlled {
    pub fn new(animation: Box<dyn Animation>) -> Box<dyn Animation> {
        Box::new(Self {
            animation,
            parameters: Parameters { speed_factor: 1.0 },
            reference_fake_time: 0.0,
            reference_real_time: 0.0,
        })
    }
}
