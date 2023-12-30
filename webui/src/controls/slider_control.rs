use animation_api::schema::ValueSchema;
use web_sys::HtmlInputElement;
use yew::{html, Component, Context, Html, NodeRef};

use super::ParameterControlProps;

fn display_value(value: f64, percent: bool) -> String {
    if percent {
        format!("{}%", (value * 100.0) as i32)
    } else {
        format!("{:.2}", value)
    }
}
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

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::InputChange => {
                let value = self
                    .slider_ref
                    .cast::<HtmlInputElement>()
                    .map(|elem| elem.value())
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let percent = matches!(ctx.props().schema.value, ValueSchema::Percentage);

                self.value_display_ref
                    .cast::<HtmlInputElement>()
                    .unwrap()
                    .set_value(&display_value(value, percent));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let schema = &ctx.props().schema;
        let (min, max, step, percent) = match &schema.value {
            ValueSchema::Number { min, max, step } => (*min, *max, *step, false),
            ValueSchema::Percentage => (0.0, 1.0, 0.01, true),
            _ => return html!(),
        };

        let link = ctx.link();
        let value = ctx
            .props()
            .value
            .as_ref()
            .and_then(|v| v.as_f64())
            .unwrap_or((max + min) / 2.0);

        html! {
            <div class="slider-control">
                <input name={schema.id.clone()}
                    type="range" min={min.to_string()} max={max.to_string()} step={step.to_string()}
                    ref={self.slider_ref.clone()}
                    oninput={link.callback(|_| Msg::InputChange)}
                    value={value.to_string()} />
                <input type="text" ref={self.value_display_ref.clone()} value={display_value(value, percent)} disabled=true class="value-display" />
            </div>
        }
    }
}
