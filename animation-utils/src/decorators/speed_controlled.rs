use animation_api::parameter_schema::{
    GetParametersSchema, Parameter, ParameterValue, ParametersSchema,
};
use animation_api::Animation;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Parameters<P: GetParametersSchema> {
    speed_factor: f64,

    #[serde(flatten)]
    inner: P,
}

impl<P: GetParametersSchema> GetParametersSchema for Parameters<P> {
    fn schema() -> ParametersSchema {
        let mut parameters = vec![Parameter {
            id: "speed_factor".to_owned(),
            name: "Speed Factor".to_owned(),
            description: None,
            value: ParameterValue::Speed,
        }];
        parameters.extend(P::schema().parameters);
        ParametersSchema { parameters }
    }
}

pub struct SpeedControlled<P: GetParametersSchema, A: Animation<Parameters = P>> {
    animation: A,
    parameters: Parameters<P>,
}

impl<A, P> Animation for SpeedControlled<P, A>
where
    A: Animation<Parameters = P>,
    P: GetParametersSchema + Default + Clone + Serialize + DeserializeOwned,
{
    type Parameters = Parameters<P>;

    fn update(&mut self, delta: f64) {
        self.animation.update(delta * self.parameters.speed_factor);
    }

    fn render(&self) -> lightfx::Frame {
        self.animation.render()
    }

    fn animation_name(&self) -> &str {
        self.animation.animation_name()
    }

    fn parameter_schema(&self) -> ParametersSchema {
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

impl<P: GetParametersSchema + Default, A: Animation<Parameters = P>> SpeedControlled<P, A> {
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
