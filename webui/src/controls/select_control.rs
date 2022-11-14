use std::rc::Rc;

use rustmas_animation_model::parameter_schema::{Parameter, ParameterValue};
use serde_json::json;
use web_sys::HtmlInputElement;
use yew::{html, Component, Context, Html, NodeRef};

use super::{register_input, Input, ParameterControlProps};

#[derive(Default)]
pub struct SelectInput {
    node_ref: NodeRef,
}

impl Input for SelectInput {
    fn get_key_value(&self) -> (String, serde_json::Value) {
        match self.node_ref.cast::<HtmlInputElement>() {
            Some(node) => (node.name().clone(), serde_json::Value::String(node.value())),
            None => ("".to_owned(), json!({})),
        }
    }
}

pub struct SelectParameterControl {
    input: Rc<SelectInput>,
}

impl Component for SelectParameterControl {
    type Message = ();
    type Properties = ParameterControlProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            input: register_input(ctx, Rc::new(SelectInput::default())),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Parameter {
            id,
            value: ParameterValue::Enum { values },
            ..
        } = ctx.props().schema.clone()
        {
            html! {
                <select name={id} ref={self.input.node_ref.clone()}>
                    {values.into_iter().map(|item| html!(
                        <option value={item.value}><strong>{item.name}</strong> {item.description.unwrap_or_default()}</option>
                    )).collect::<Html>()}
                </select>
            }
        } else {
            html!()
        }
    }
}
