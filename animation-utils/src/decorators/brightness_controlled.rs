use animation_api::parameter_schema::{
    GetParametersSchema, Parameter, ParameterValue, ParametersSchema,
};
use animation_api::Animation;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Parameters<P: GetParametersSchema> {
    brightness_factor: f64,
    #[serde(flatten)]
    inner: P,
}

impl<P: GetParametersSchema> GetParametersSchema for Parameters<P> {
    fn schema() -> ParametersSchema {
        let mut parameters = vec![Parameter {
            id: "brightness_factor".to_owned(),
            name: "Brightness".to_owned(),
            description: None,
            value: ParameterValue::Percentage,
        }];
        parameters.extend(P::schema().parameters);
        ParametersSchema { parameters }
    }
}

pub struct BrightnessControlled<P: GetParametersSchema, A: Animation<Parameters = P>> {
    animation: A,
    parameters: Parameters<P>,
}

impl<A, P> Animation for BrightnessControlled<P, A>
where
    A: Animation<Parameters = P>,
    P: GetParametersSchema + Default + Clone + Serialize + DeserializeOwned,
{
    type Parameters = Parameters<P>;

    fn update(&mut self, delta: f64) {
        self.animation.update(delta)
    }

    fn render(&self) -> lightfx::Frame {
        self.animation
            .render()
            .pixels_iter()
            .map(|x| x.dim(self.parameters.brightness_factor))
            .into()
    }

    fn animation_name(&self) -> &str {
        self.animation.animation_name()
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

impl<P: GetParametersSchema + Default, A: Animation<Parameters = P>> BrightnessControlled<P, A> {
    pub fn new(animation: A) -> Self {
        Self {
            animation,
            parameters: Parameters {
                brightness_factor: 1.0,
                inner: Default::default(),
            },
        }
    }
}
