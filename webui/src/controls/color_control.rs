use std::rc::Rc;

use rustmas_animation_model::parameter_schema::{Parameter, ParameterValue};
use serde_json::json;
use web_sys::HtmlInputElement;
use yew::{html, Component, Context, Html, NodeRef};

use super::{register_input, Input, ParameterControlProps};

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

#[derive(Default)]
pub struct ColorInput {
    node_ref: NodeRef,
}

impl Input for ColorInput {
    fn get_key_value(&self) -> (String, serde_json::Value) {
        match self.node_ref.cast::<HtmlInputElement>() {
            Some(node) => (node.name().clone(), hex_from_str(&node.value()).unwrap()),
            None => ("".to_owned(), json!({})),
        }
    }
}

pub struct ColorParameterControl {
    input: Rc<ColorInput>,
}

impl Component for ColorParameterControl {
    type Message = ();
    type Properties = ParameterControlProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            input: register_input(ctx, Rc::new(ColorInput::default())),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Parameter {
            id,
            value: ParameterValue::Color,
            ..
        } = ctx.props().schema.clone()
        {
            html! {
                <input name={id} type="color" ref={self.input.node_ref.clone()} />
            }
        } else {
            html!()
        }
    }
}
