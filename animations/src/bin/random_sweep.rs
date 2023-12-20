use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::{EnumSchema, ParameterSchema};
use itertools::Itertools;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Default, Serialize, Deserialize, EnumSchema)]
pub enum SweepType {
    #[schema_variant(name = "2D")]
    Sweep2D,

    #[schema_variant(name = "3D")]
    #[default]
    Sweep3D,
}

#[derive(Clone, Serialize, Deserialize, ParameterSchema)]
pub struct Parameters {
    #[schema_field(name = "Tail length", number(min = 0.0, max = 2.0, step = 0.05))]
    tail_length: f64,

    #[schema_field(name = "Sweep type", enum_options)]
    sweep_type: SweepType,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            tail_length: 0.5,
            sweep_type: Default::default(),
        }
    }
}

#[animation_utils::plugin]
pub struct RandomSweep {
    points: Vec<Option<Vector3<f64>>>,
    heights: Vec<Option<f64>>,
    color: lightfx::Color,
    current_height: f64,
    max_height: f64,
    parameters: Parameters,
}

impl RandomSweep {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points: points
                .into_iter()
                .map(|(x, y, z)| {
                    if x.to_bits() == (-1.0_f64).to_bits()
                        && y.to_bits() == (-1.0_f64).to_bits()
                        && z.to_bits() == (-1.0_f64).to_bits()
                    {
                        None
                    } else {
                        Some(Vector3::new(x, y, z))
                    }
                })
                .collect(),
            heights: Vec::new(),
            color: lightfx::Color::black(),
            current_height: 0.0,
            max_height: 0.0,
            parameters: Default::default(),
        }))
    }
}

impl Animation for RandomSweep {
    type Parameters = Parameters;

    fn update(&mut self, delta: f64) {
        if self.current_height > self.max_height + self.parameters.tail_length {
            let rotation = match self.parameters.sweep_type {
                SweepType::Sweep2D => animation_utils::random_rotation_around(&Vector3::z_axis()),
                SweepType::Sweep3D => animation_utils::random_rotation(),
            };
            self.heights = self
                .points
                .iter()
                .map(|p| p.map(|p| rotation * p))
                .map(|p| p.map(|p| p.y))
                .collect();
            self.color = animation_utils::random_hue(1.0, 1.0);
            (self.current_height, self.max_height) =
                match self.heights.iter().filter_map(|x| x.as_ref()).minmax() {
                    itertools::MinMaxResult::MinMax(min, max) => (*min, *max),
                    _ => return,
                };
        }

        self.current_height += delta;
    }

    fn render(&self) -> lightfx::Frame {
        self.heights
            .iter()
            .map(|h| {
                if let Some(h) = h {
                    if *h < self.current_height
                        && *h > self.current_height - self.parameters.tail_length
                    {
                        self.color
                            .dim(1.0 - (self.current_height - h) / self.parameters.tail_length)
                    } else {
                        lightfx::Color::black()
                    }
                } else {
                    lightfx::Color::black()
                }
            })
            .into()
    }

    fn animation_name(&self) -> &str {
        "Random Sweep"
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
