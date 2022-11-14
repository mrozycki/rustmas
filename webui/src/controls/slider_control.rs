use std::rc::Rc;

use rustmas_animation_model::parameter_schema::{Parameter, ParameterValue};
use serde_json::json;
use web_sys::{HtmlElement, HtmlInputElement};
use yew::{html, Component, Context, Html, NodeRef};

use super::{register_input, Input, ParameterControlProps};

#[derive(Default)]
pub struct SliderInput {
    slider_ref: NodeRef,
    value_display_ref: NodeRef,
}

impl Input for SliderInput {
    fn get_key_value(&self) -> (String, serde_json::Value) {
        match self.slider_ref.cast::<HtmlInputElement>() {
            Some(node) => (
                node.name().clone(),
                json!(node.value().parse::<f64>().unwrap_or_default()),
            ),
            None => ("".to_owned(), json!({})),
        }
    }
}

pub struct SliderParameterControl {
    input: Rc<SliderInput>,
}

pub enum Msg {
    InputChange,
}

impl Component for SliderParameterControl {
    type Message = Msg;
    type Properties = ParameterControlProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            input: register_input(ctx, Rc::new(SliderInput::default())),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::InputChange => {
                self.input
                    .value_display_ref
                    .cast::<HtmlElement>()
                    .unwrap()
                    .set_inner_text(
                        &self
                            .input
                            .slider_ref
                            .cast::<HtmlInputElement>()
                            .unwrap()
                            .value(),
                    );
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Parameter {
            id,
            value: ParameterValue::Number { min, max },
            ..
        } = ctx.props().schema.clone()
        {
            let link = ctx.link();
            let min = min.unwrap_or(0.0);
            let max = max.unwrap_or(100.0);
            let step = (max - min).abs() / 100.0;

            html! {
                <>
                    <input name={id}
                        type="range" min={min.to_string()} max={max.to_string()} step={step.to_string()}
                        ref={self.input.slider_ref.clone()}
                        oninput={link.callback(|_| Msg::InputChange)} />
                    <output ref={self.input.value_display_ref.clone()}>{((max + min) / 2.0).to_string()}</output>
                </>
            }
        } else {
            html!()
        }
    }
}
