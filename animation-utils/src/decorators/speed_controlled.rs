use animation_api::schema::{ConfigurationSchema, GetSchema, ParameterSchema, ValueSchema};
use animation_api::Animation;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Parameters<P: GetSchema> {
    speed_factor: f64,

    #[serde(flatten)]
    inner: P,
}

impl<P: GetSchema> GetSchema for Parameters<P> {
    fn schema() -> ConfigurationSchema {
        let mut parameters = vec![ParameterSchema {
            id: "speed_factor".to_owned(),
            name: "Speed Factor".to_owned(),
            description: None,
            value: ValueSchema::Speed,
        }];
        parameters.extend(P::schema().parameters);
        ConfigurationSchema { parameters }
    }
}

pub struct SpeedControlled<P: GetSchema, A: Animation<Parameters = P>> {
    animation: A,
    parameters: Parameters<P>,
}

impl<A, P> Animation for SpeedControlled<P, A>
where
    A: Animation<Parameters = P>,
    P: GetSchema + Default + Clone + Serialize + DeserializeOwned,
{
    type Parameters = Parameters<P>;

    fn update(&mut self, delta: f64) {
        self.animation.update(delta * self.parameters.speed_factor);
    }

    fn on_event(&mut self, event: animation_api::event::Event) {
        self.animation.on_event(event)
    }

    fn render(&self) -> lightfx::Frame {
        self.animation.render()
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
        (self.animation.get_fps() * self.parameters.speed_factor.abs()).clamp(0.0, 30.0)
    }
}

impl<P: GetSchema + Default, A: Animation<Parameters = P>> SpeedControlled<P, A> {
    pub fn new(animation: A) -> Self {
        Self {
            animation,
            parameters: Parameters {
                speed_factor: 1.0,
                inner: Default::default(),
            },
        }
    }
}
