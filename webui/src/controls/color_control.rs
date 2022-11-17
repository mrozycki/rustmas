use rustmas_animation_model::parameter_schema::{Parameter, ParameterValue};
use serde_json::json;
use web_sys::HtmlInputElement;
use yew::{html, Component, Context, Html, NodeRef};

use super::ParameterControlProps;

fn hex_from_str(code: &str) -> Option<serde_json::Value> {
    let code = code.trim_start_matches(|c| c == '#');
    let (r, g, b) = match code.len() {
        6 => match u32::from_str_radix(code, 16) {
            Ok(x) => ((x & 0xFF0000) >> 16, (x & 0x00FF00) >> 8, x & 0x0000FF),
            Err(_) => return None,
        },
        3 => match u32::from_str_radix(code, 16) {
            Ok(x) => (
                ((x & 0xF00) >> 8) * 0x11,
                ((x & 0x0F0) >> 4) * 0x11,
                (x & 0x00F) * 0x11,
            ),
            Err(_) => return None,
        },
        _ => return None,
    };

    Some(json!({"r": r, "g": g, "b": b}))
}

fn hex_from_value(value: &serde_json::Value) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        value.get("r").unwrap().as_i64().unwrap(),
        value.get("g").unwrap().as_i64().unwrap(),
        value.get("b").unwrap().as_i64().unwrap()
    )
}

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
                match (
                    self.node_ref.cast::<HtmlInputElement>(),
                    self.hidden_ref.cast::<HtmlInputElement>(),
                ) {
                    (Some(node), Some(hidden)) => hidden
                        .set_value(&serde_json::to_string(&hex_from_str(&node.value())).unwrap()),
                    _ => (),
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
                .as_ref()
                .map(hex_from_value)
                .unwrap_or("#000000".to_owned());
            let value_hex = serde_json::to_string(&ctx.props().value).unwrap();

            html! {
                <>
                    <input type="color" ref={self.node_ref.clone()} onchange={ctx.link().callback(|_| Msg::ValueChanged)} value={value}/>
                    <input name={id} type="hidden" ref={self.hidden_ref.clone()} value={value_hex}/>
                </>
            }
        } else {
            html!()
        }
    }
}
