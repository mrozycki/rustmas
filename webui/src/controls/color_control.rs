use animation_api::parameter_schema::{Parameter, ParameterValue};
use web_sys::HtmlInputElement;
use yew::{html, Component, Context, Html, NodeRef};

use super::ParameterControlProps;

#[derive(Default)]
pub struct ColorInput {}

#[derive(Default)]
pub struct ColorParameterControl {
    node_ref: NodeRef,
    hidden_ref: NodeRef,
}

pub enum Msg {
    ValueChanged,
}

impl Component for ColorParameterControl {
    type Message = Msg;
    type Properties = ParameterControlProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ValueChanged => {
                if let (Some(node), Some(hidden)) = (
                    self.node_ref.cast::<HtmlInputElement>(),
                    self.hidden_ref.cast::<HtmlInputElement>(),
                ) {
                    hidden.set_value(
                        &serde_json::to_string(
                            &lightfx::Color::from_hex_str(&node.value()).unwrap(),
                        )
                        .unwrap(),
                    );
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Parameter {
            id,
            value: ParameterValue::Color,
            ..
        } = ctx.props().schema.clone()
        {
            let value = ctx
                .props()
                .value
                .clone()
                .and_then(|v| serde_json::from_value::<lightfx::Color>(v).ok())
                .map(|v| lightfx::Color::to_hex_string(&v))
                .unwrap_or("#000000".to_owned());
            let value_hex = serde_json::to_string(&ctx.props().value).unwrap();

            html! {
                <>
                    <input type="color" ref={self.node_ref.clone()} onchange={ctx.link().callback(|_| Msg::ValueChanged)} {value} list="warmWhites" />
                    <input name={id} type="hidden" ref={self.hidden_ref.clone()} value={value_hex}/>
                </>
            }
        } else {
            html!()
        }
    }
}
