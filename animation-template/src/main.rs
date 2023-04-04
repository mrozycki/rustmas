use animation_api::{AnimationParameters, StepAnimation};
use lightfx::parameter_schema::{Parameter, ParameterValue, ParametersSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Parameters {
    // TODO: Define your animation's parameters
    tail_length: usize,
}

#[animation_utils::plugin]
pub struct MyAnimation {
    // TODO: Define your animation's state
    points_count: usize,
    time: f64,
    parameters: Parameters,
}

impl MyAnimation {
    pub fn new(points: Vec<(f64, f64, f64)>) -> Box<dyn StepAnimation> {
        // TODO: Initialize animation state from a set of light locations
        Box::new(Self {
            points_count: points.len(),
            time: 0.0,
            parameters: Parameters { tail_length: 3 },
        })
    }
}

impl StepAnimation for MyAnimation {
    fn update(&mut self, time_delta: f64) {
        // TODO: Update your animation state by time_delta seconds
        self.time += time_delta;
    }

    fn render(&self) -> lightfx::Frame {
        // TODO: Render a frame of your animation
        let index = ((self.time * 8.0) % self.points_count as f64) as usize;

        (0..self.points_count)
            .into_iter()
            .map(|i| {
                if i + self.parameters.tail_length > index && i <= index {
                    lightfx::Color::white()
                } else {
                    lightfx::Color::black()
                }
            })
            .into()
    }
}

impl AnimationParameters for MyAnimation {
    fn animation_name(&self) -> &str {
        // TODO: Return the name of your animation
        "Animation Plugin Template"
    }

    fn get_fps(&self) -> f64 {
        // TODO: Return the FPS of your animation
        8.0
    }

    fn parameter_schema(&self) -> lightfx::parameter_schema::ParametersSchema {
        // TODO: Describe the schema of your animation
        ParametersSchema {
            parameters: vec![Parameter {
                id: "tail_length".to_owned(),
                name: "Tail length".to_owned(),
                description: Some("Number of lights lit at once".into()),
                value: ParameterValue::Number {
                    min: 1.0,
                    max: 20.0,
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
        // You might need to reset the state of your animation in some cases.
        // Otherwise there's nothing to do here.
        self.parameters = serde_json::from_value(parameters)?;
        Ok(())
    }
}
