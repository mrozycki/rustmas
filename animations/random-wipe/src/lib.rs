use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use animation_utils::{EnumSchema, Schema};
use itertools::Itertools;
use nalgebra::Vector3;
use rand::Rng;
use serde::{Deserialize, Serialize};

// This is constant, because the tail is not visible, and adjusting it
// does not make sense. It just adds a little bit of delay between wipes.
const TAIL_LENGTH: f64 = 0.1;

#[derive(Clone, Default, Serialize, Deserialize, EnumSchema)]
pub enum SweepType {
    #[schema_variant(name = "2D")]
    Sweep2D,

    #[schema_variant(name = "3D")]
    #[default]
    Sweep3D,
}

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Lead length", number(min = 0.0, max = 2.0, step = 0.05))]
    lead_length: f64,

    #[schema_field(name = "Sweep type", enum_options)]
    sweep_type: SweepType,

    #[schema_field(name = "Hue step", number(min = "-1.0", max = 1.0, step = 0.05))]
    hue_step: f64,

    #[schema_field(name = "Hue step variance", number(min = 0.0, max = 1.0, step = 0.05))]
    hue_step_variance: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            lead_length: 0.3,
            sweep_type: Default::default(),
            hue_step: 0.5,
            hue_step_variance: 0.5,
        }
    }
}

#[animation_utils::wasm_plugin]
pub struct RandomSweep {
    points: Vec<Option<Vector3<f64>>>,
    heights: Vec<Option<f64>>,
    hue: f64,
    next_hue: f64,
    current_height: f64,
    max_height: f64,
    parameters: Parameters,
}

impl RandomSweep {
    fn next_hue(&self, curr: f64) -> f64 {
        (curr
            + self.parameters.hue_step
            + (rand::rng().random::<f64>().fract() - self.parameters.hue_step_variance) / 2.0)
            .fract()
    }
}

impl Animation for RandomSweep {
    type Parameters = Parameters;
    type CustomTriggers = ();
    type Wrapped = SpeedControlled<BrightnessControlled<Self>>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        let initial_hue = rand::rng().random::<f64>().fract();

        Self {
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
            hue: initial_hue,
            next_hue: initial_hue,
            current_height: 0.0,
            max_height: 0.0,
            parameters: Default::default(),
        }
    }

    fn update(&mut self, delta: f64) {
        if self.heights.is_empty() || self.current_height > self.max_height + TAIL_LENGTH {
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
            self.hue = self.next_hue;
            self.next_hue = self.next_hue(self.hue);
            (self.current_height, self.max_height) =
                match self.heights.iter().filter_map(|x| x.as_ref()).minmax() {
                    itertools::MinMaxResult::MinMax(min, max) => {
                        (*min - self.parameters.lead_length, *max)
                    }
                    _ => return,
                };
        }

        self.current_height += delta;
    }

    fn render(&self) -> lightfx::Frame {
        let color = lightfx::Color::hsv(self.hue, 1.0, 1.0);
        let next_color = lightfx::Color::hsv(self.next_hue, 1.0, 1.0);

        self.heights
            .iter()
            .map(|h| {
                if let Some(h) = h {
                    let h = h - self.current_height;

                    if h < 0.0 {
                        next_color
                    } else if h < self.parameters.lead_length {
                        let alpha = h / self.parameters.lead_length;
                        color
                            .with_alpha(alpha)
                            .blend(&next_color.with_alpha(1.0))
                            .apply_alpha()
                    } else {
                        color
                    }
                } else {
                    lightfx::Color::black()
                }
            })
            .collect()
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        self.parameters.clone()
    }
}
