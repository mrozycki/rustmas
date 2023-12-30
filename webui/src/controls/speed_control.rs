use animation_api::schema::{ParameterSchema, ValueSchema};
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

fn raw_to_speed(raw: f64) -> f64 {
    if raw.abs() > 1.0 {
        ((raw.abs() - 1.0) * (MAX_SPEED - 1.0) + 1.0) * raw.signum()
    } else {
        raw
    }
}

fn speed_to_raw(speed: f64) -> f64 {
    if speed.abs() > 1.0 {
        ((speed.abs() - 1.0) / (MAX_SPEED - 1.0) + 1.0) * speed.signum()
    } else {
        speed
    }
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
                let raw = input.value().parse::<f64>().unwrap_or(1.0);

                let speed = raw_to_speed(raw);

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
        if let ParameterSchema {
            id,
            value: ValueSchema::Speed,
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

            let raw = speed_to_raw(speed);

            html! {
                <div class="slider-control">
                    <input
                        type="range" min="-2.0" max="2.0" step="0.05"
                        ref={self.slider_ref.clone()}
                        oninput={link.callback(|_| Msg::InputChange)}
                        value={raw.to_string()} />
                    <input type="text" ref={self.value_display_ref.clone()} value={format!("{:.2}x", speed)} disabled=true class="value-display" />
                    <input name={id} type="hidden" ref={self.hidden_ref.clone()} value={speed.to_string()} />
                </div>
            }
        } else {
            html!()
        }
    }
}
