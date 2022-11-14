mod color_control;
mod select_control;
mod slider_control;

use std::{collections::HashMap, rc::Rc};

use rustmas_animation_model::{
    parameter_schema::{ParameterValue, ParametersSchema},
    schema::Parameter,
};
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::api;
use color_control::ColorParameterControl;
use select_control::SelectParameterControl;
use slider_control::SliderParameterControl;

fn register_input<COMP: Component, IN: Input + 'static>(
    ctx: &Context<COMP>,
    input: Rc<IN>,
) -> Rc<IN> {
    ctx.link()
        .get_parent()
        .unwrap()
        .clone()
        .downcast::<ParameterControlList>()
        .send_message(ParameterControlListMsg::RegisterChild(input.clone()));

    input
}

pub trait Input {
    fn get_key_value(&self) -> (String, serde_json::Value);
}

#[derive(Properties, PartialEq, Clone)]
pub struct ParameterControlProps {
    schema: Parameter,
}

pub struct ParameterControlList {
    children: Vec<Rc<dyn Input>>,
}

impl ParameterControlList {
    pub fn get_values(&self) -> serde_json::Value {
        let values = self
            .children
            .iter()
            .map(|c| c.get_key_value())
            .collect::<HashMap<_, _>>();
        serde_json::to_value(values).unwrap()
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct ParameterControlListProps {
    pub schema: ParametersSchema,
}

pub enum ParameterControlListMsg {
    RegisterChild(Rc<dyn Input>),
    SubmitInfo,
}

impl Component for ParameterControlList {
    type Message = ParameterControlListMsg;
    type Properties = ParameterControlListProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            children: Vec::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ParameterControlListMsg::RegisterChild(child) => {
                self.children.push(child);
                false
            }
            ParameterControlListMsg::SubmitInfo => {
                let params = self.get_values();
                let (api, _) = _ctx
                    .link()
                    .context::<api::Gateway>(Callback::noop())
                    .expect("gateway to be created");

                wasm_bindgen_futures::spawn_local(async move {
                    let _ = api.set_params(&params).await;
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        html! {
            <div>
                {
                    ctx.props().schema.parameters.iter().map(|schema| html! {
                        <div>
                            <h3>{ &schema.name }</h3>
                            {
                                if let Some(description) = &schema.description {
                                    html! {
                                        <p>{ description }</p>
                                    }
                                } else { html!{} }
                            }
                            {
                                match schema.value {
                                    ParameterValue::Enum {..} => html!{<SelectParameterControl schema={schema.clone()} />},
                                    ParameterValue::Color => html!{<ColorParameterControl schema={schema.clone()} />},
                                    ParameterValue::Number {..} => html!{<SliderParameterControl schema={schema.clone()} />},
                                }
                            }
                        </div>
                    }).collect::<Html>()
                }
                <input type="submit" value="Send" onclick={ctx.link().callback(|_| Self::Message::SubmitInfo)}/>
            </div>
        }
    }
}
