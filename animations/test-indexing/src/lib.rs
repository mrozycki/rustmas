use animation_api::Animation;
use animation_api::schema::{EnumOption, GetSchema, ParameterSchema, ValueSchema};
use animation_utils::decorators::BrightnessControlled;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Parameters {
    bit: usize,
}

impl GetSchema for Parameters {
    fn schema() -> Vec<ParameterSchema> {
        vec![ParameterSchema {
            id: "bit".to_owned(),
            name: "Bit".to_owned(),
            description: None,
            value: ValueSchema::Enum {
                values: (0..10)
                    .map(|i| EnumOption {
                        name: format!("{}s", 1 << i),
                        description: None,
                        value: i.to_string(),
                    })
                    .collect(),
            },
        }]
    }
}

#[animation_utils::wasm_plugin]
pub struct Indexing {
    points_count: usize,
    parameters: Parameters,
}

impl Animation for Indexing {
    type Parameters = Parameters;
    type CustomTriggers = ();
    type Wrapped = BrightnessControlled<Self>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            points_count: points.len(),
            parameters: Default::default(),
        }
    }

    fn render(&self) -> lightfx::Frame {
        (0..self.points_count)
            .map(|x| match (x >> self.parameters.bit) % 2 {
                0 => lightfx::Color::black(),
                _ => lightfx::Color::white(),
            })
            .collect()
    }

    fn get_fps(&self) -> f64 {
        0.0
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
