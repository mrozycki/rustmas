use lightfx::parameter_schema::{Parameter, ParameterValue};
use serde_json::json;
use web_sys::{HtmlElement, HtmlInputElement};
use yew::{html, Component, Context, Html, NodeRef};

use super::ParameterControlProps;

#[derive(Default)]
pub struct SliderParameterControl {
    slider_ref: NodeRef,
    value_display_ref: NodeRef,
}

pub enum Msg {
    InputChange,
}

impl Component for SliderParameterControl {
    type Message = Msg;
    type Properties = ParameterControlProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::InputChange => {
                self.value_display_ref
                    .cast::<HtmlElement>()
                    .unwrap()
                    .set_inner_text(&self.slider_ref.cast::<HtmlInputElement>().unwrap().value());
                false
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
            let value = serde_json::to_string(
                ctx.props()
                    .value
                    .as_ref()
                    .unwrap_or(&json!((max + min) / 2.0)),
            )
            .unwrap();

            html! {
                <>
                    <input name={id}
                        type="range" min={min.to_string()} max={max.to_string()} step={step.to_string()}
                        ref={self.slider_ref.clone()}
                        oninput={link.callback(|_| Msg::InputChange)}
                        value={value.clone()} />
                    <output ref={self.value_display_ref.clone()}>{value}</output>
                </>
            }
        } else {
            html!()
        }
    }
}
