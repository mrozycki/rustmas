use animation_api::schema::{
    ConfigurationSchema, GetEnumOptions, GetSchema, ParameterSchema, ValueSchema,
};
use animation_api::Animation;
use animation_plugin_macro::EnumSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Serialize, Deserialize, EnumSchema, PartialEq)]
pub enum Switch {
    #[schema_variant(name = "On")]
    #[default]
    On,

    #[schema_variant(name = "Off")]
    Off,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Parameters<P: GetSchema> {
    off_switch_delay: f64,
    off_switch_state: Switch,

    #[serde(flatten)]
    inner: P,
}

impl<P: GetSchema> GetSchema for Parameters<P> {
    fn schema() -> ConfigurationSchema {
        let mut parameters = vec![
            ParameterSchema {
                id: "off_switch_state".to_owned(),
                name: "State".to_owned(),
                description: None,
                value: ValueSchema::Enum {
                    values: Switch::enum_options(),
                },
            },
            ParameterSchema {
                id: "off_switch_delay".to_owned(),
                name: "Switch delay".to_owned(),
                description: None,
                value: ValueSchema::Number {
                    min: 0.0,
                    max: 5.0,
                    step: 0.1,
                },
            },
        ];
        parameters.extend(P::schema().parameters);
        ConfigurationSchema { parameters }
    }
}

pub struct OffSwitch<P: GetSchema, A: Animation<Parameters = P>> {
    animation: A,
    parameters: Parameters<P>,
    energy: f64,
}

impl<A, P> Animation for OffSwitch<P, A>
where
    A: Animation<Parameters = P>,
    P: GetSchema + Default + Clone + Serialize + DeserializeOwned,
{
    type Parameters = Parameters<P>;

    fn update(&mut self, time_delta: f64) {
        let energy_delta = if self.parameters.off_switch_delay == 0.0 {
            1.0
        } else {
            time_delta / self.parameters.off_switch_delay
        };
        self.energy = match self.parameters.off_switch_state {
            Switch::Off => self.energy - energy_delta,
            Switch::On => self.energy + energy_delta,
        }
        .clamp(0.0, 1.0);

        self.animation.update(time_delta);
    }

    fn on_event(&mut self, event: animation_api::event::Event) {
        self.animation.on_event(event)
    }

    fn render(&self) -> lightfx::Frame {
        self.animation
            .render()
            .pixels_iter()
            .map(|x| x.dim(self.energy))
            .into()
    }

    fn get_schema(&self) -> ConfigurationSchema {
        Self::Parameters::schema()
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.animation.set_parameters(parameters.inner.clone());
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters {
        let inner = self.animation.get_parameters();
        Self::Parameters {
            inner,
            ..self.parameters
        }
    }

    fn get_fps(&self) -> f64 {
        self.animation.get_fps()
    }
}

impl<P: GetSchema + Default, A: Animation<Parameters = P>> OffSwitch<P, A> {
    pub fn new(animation: A) -> Self {
        Self {
            animation,
            parameters: Parameters {
                off_switch_delay: 1.0,
                off_switch_state: Switch::On,
                inner: Default::default(),
            },
            energy: 0.0,
        }
    }
}
