use lightfx::parameter_schema::{Parameter, ParameterValue};
use web_sys::HtmlInputElement;
use yew::{html, Component, Context, Html, NodeRef};

use super::ParameterControlProps;

const MAX_SPEED: f64 = 5.0;

#[derive(Default)]
pub struct SpeedParameterControl {
    slider_ref: NodeRef,
    value_display_ref: NodeRef,
    hidden_ref: NodeRef,
}

pub enum Msg {
    InputChange,
}

impl Component for SpeedParameterControl {
    type Message = Msg;
    type Properties = ParameterControlProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::InputChange => {
                let input = self.slider_ref.cast::<HtmlInputElement>().unwrap();
                let raw_value = input.value().parse::<f64>().unwrap_or(1.0);

                let speed = if raw_value > 1.0 {
                    (raw_value - 1.0) * (MAX_SPEED - 1.0) + 1.0
                } else {
                    raw_value
                };

                self.value_display_ref
                    .cast::<HtmlInputElement>()
                    .unwrap()
                    .set_value(&format!("{:.2}x", speed));
                self.hidden_ref
                    .cast::<HtmlInputElement>()
                    .unwrap()
                    .set_value(&speed.to_string());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Parameter {
            id,
            value: ParameterValue::Speed,
            ..
        } = ctx.props().schema.clone()
        {
            let link = ctx.link();
            let speed = ctx
                .props()
                .value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);
            let raw_value = if speed > 1.0 {
                (speed - 1.0) / (MAX_SPEED - 1.0) + 1.0
            } else {
                speed
            };

            html! {
                <div class="slider-control">
                    <input
                        type="range" min="0.0" max="2.0" step="0.05"
                        ref={self.slider_ref.clone()}
                        oninput={link.callback(|_| Msg::InputChange)}
                        value={raw_value.to_string()} />
                    <input type="text" ref={self.value_display_ref.clone()} value={format!("{:.2}x", speed)} class="value-display" />
                    <input name={id} type="hidden" ref={self.hidden_ref.clone()} value={speed.to_string()} />
                </div>
            }
        } else {
            html!()
        }
    }
}
