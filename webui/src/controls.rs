use std::{collections::HashMap, rc::Rc};

use rustmas_animation_model::{parameter_schema::ParametersSchema, schema::Parameter};
use web_sys::HtmlInputElement;
use yew::{html, Callback, Component, Context, Html, NodeRef, Properties};

use crate::api;

#[derive(Default)]
pub struct Input {
    node_ref: NodeRef,
}

impl Input {
    pub fn get_key_value(&self) -> (String, serde_json::Value) {
        let node = self.node_ref.cast::<HtmlInputElement>().unwrap();

        (
            node.name().clone(),
            serde_json::from_str(&node.value()).unwrap(),
        )
    }
}

#[derive(Clone)]
pub struct ParameterControl {
    input: Rc<Input>,
}

#[derive(Properties, PartialEq, Clone)]
pub struct ParameterControlProps {
    schema: Option<Parameter>,
}

impl Component for ParameterControl {
    type Message = ();
    type Properties = ParameterControlProps;

    fn create(ctx: &Context<Self>) -> Self {
        let result = Self {
            input: Default::default(),
        };
        ctx.link()
            .get_parent()
            .unwrap()
            .clone()
            .downcast::<ParameterControlList>()
            .send_message(ParameterControlListMsg::RegisterChild(result.input.clone()));
        result
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        if let Some(schema) = ctx.props().schema.clone() {
            html! {
                <div>
                    <h3>{ schema.name }</h3>
                    {
                        if let Some(description) = schema.description {
                            html! {
                                <p>{ description }</p>
                            }
                        } else { html!{} }
                    }
                    <input type="text" name={schema.id} ref={self.input.node_ref.clone()} />
                </div>
            }
        } else {
            html! {}
        }
    }
}

pub struct ParameterControlList {
    children: Vec<Rc<Input>>,
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
    RegisterChild(Rc<Input>),
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
                    ctx.props().schema.parameters.iter().map(|param| html! {
                        <ParameterControl schema={param.clone()} />
                    }).collect::<Html>()
                }
                <input type="submit" value="Send" onclick={ctx.link().callback(|_| Self::Message::SubmitInfo)}/>
            </div>
        }
    }
}
