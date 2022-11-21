use crate::Animation;
use lightfx::{parameter_schema::Parameter, schema::ParametersSchema};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Parameters {
    speed_factor: f64,
}
pub struct SpeedControlled {
    animation: Box<dyn Animation + Sync + Send>,
    parameters: Parameters,
}

impl Animation for SpeedControlled {
    fn frame(&mut self, time: f64) -> lightfx::Frame {
        self.animation.frame(time * self.parameters.speed_factor)
    }

    fn parameter_schema(&self) -> ParametersSchema {
        let mut parameters = self.animation.parameter_schema().parameters;
        parameters.extend(vec![Parameter {
            id: "speed_factor".to_owned(),
            name: "Speed Factor".to_owned(),
            description: None,
            value: lightfx::parameter_schema::ParameterValue::Number {
                min: Some(0.0),
                max: Some(10.0),
            },
        }]);
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

    fn get_parameters(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut parameters = self
            .animation
            .get_parameters()?
            .as_object_mut()
            .unwrap()
            .to_owned();
        parameters.extend(
            serde_json::to_value(&self.parameters)?
                .as_object_mut()
                .cloned()
                .unwrap()
                .into_iter(),
        );
        Ok(serde_json::to_value(parameters)?)
    }

    fn get_fps(&self) -> f64 {
        (self.animation.get_fps() * self.parameters.speed_factor).clamp(0.0, 30.0)
    }
}

impl SpeedControlled {
    pub fn new(animation: Box<dyn Animation + Sync + Send>) -> Box<dyn Animation + Sync + Send> {
        Box::new(Self {
            animation,
            parameters: Parameters { speed_factor: 1.0 },
        })
    }
}
