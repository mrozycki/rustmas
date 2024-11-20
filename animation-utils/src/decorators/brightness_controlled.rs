use animation_api::schema::{ConfigurationSchema, GetSchema, ParameterSchema, ValueSchema};
use animation_api::Animation;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Parameters<P: GetSchema> {
    brightness_factor: f64,
    #[serde(flatten)]
    inner: P,
}

impl<P: GetSchema> GetSchema for Parameters<P> {
    fn schema() -> ConfigurationSchema {
        let mut parameters = vec![ParameterSchema {
            id: "brightness_factor".to_owned(),
            name: "Brightness".to_owned(),
            description: None,
            value: ValueSchema::Percentage,
        }];
        parameters.extend(P::schema().parameters);
        ConfigurationSchema { parameters }
    }
}

pub struct BrightnessControlled<A: Animation<Parameters: GetSchema>> {
    animation: A,
    parameters: Parameters<A::Parameters>,
}

impl<A> Animation for BrightnessControlled<A>
where
    A: Animation,
    A::Parameters: GetSchema + Default + Clone + Serialize + DeserializeOwned,
{
    type Parameters = Parameters<A::Parameters>;
    type Wrapped = Self;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            animation: A::new(points),
            parameters: Parameters {
                brightness_factor: 1.0,
                inner: Default::default(),
            },
        }
    }

    fn update(&mut self, delta: f64) {
        self.animation.update(delta)
    }

    fn on_event(&mut self, event: animation_api::event::Event) {
        self.animation.on_event(event)
    }

    fn render(&self) -> lightfx::Frame {
        self.animation
            .render()
            .pixels_iter()
            .map(|x| x.dim(self.parameters.brightness_factor))
            .into()
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
