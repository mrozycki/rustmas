use rustmas_animation_model::parameter_schema::{Parameter, ParameterValue};
use serde_json::json;
use yew::{html, Component, Context, Html, NodeRef};

use super::ParameterControlProps;

#[derive(Default)]
pub struct SelectParameterControl {
    node_ref: NodeRef,
}

impl Component for SelectParameterControl {
    type Message = ();
    type Properties = ParameterControlProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Parameter {
            id,
            value: ParameterValue::Enum { values },
            ..
        } = ctx.props().schema.clone()
        {
            let selected_value = serde_json::to_string(ctx.props().value.as_ref().unwrap_or(
                &json!(values.first().map(|v| v.name.clone()).unwrap_or_default()),
            ))
            .unwrap();
            html! {
                <select name={id} ref={self.node_ref.clone()}>
                    {values.into_iter().map(|item| {
                        let value = format!("\"{}\"", item.value);
                        let selected = value == selected_value;
                        html!(
                        <option {selected} {value}><strong>{item.name}</strong> {item.description.unwrap_or_default()}</option>
                    )}).collect::<Html>()}
                </select>
            }
        } else {
            html!()
        }
    }
}
