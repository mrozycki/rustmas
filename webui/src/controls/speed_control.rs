use animation_api::schema::{ParameterSchema, ValueSchema};
use web_sys::HtmlInputElement;
use yew::{html, Callback};

use super::ParameterControlProps;

const MAX_SPEED: f64 = 5.0;

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

#[yew::function_component(SpeedParameterControl)]
pub fn speed_parameter_control(props: &ParameterControlProps) -> Html {
    let slider_ref = yew::use_node_ref();
    let value_display_ref = yew::use_node_ref();
    let hidden_ref = yew::use_node_ref();

    let oninput: Callback<_> = Callback::from({
        let slider_ref = slider_ref.clone();
        let value_display_ref = value_display_ref.clone();
        let hidden_ref = hidden_ref.clone();
        move |_| {
            let speed = slider_ref
                .cast::<HtmlInputElement>()
                .unwrap()
                .value()
                .parse::<f64>()
                .map(raw_to_speed)
                .unwrap_or(1.0);

            value_display_ref
                .cast::<HtmlInputElement>()
                .unwrap()
                .set_value(&format!("{:.2}x", speed));
            hidden_ref
                .cast::<HtmlInputElement>()
                .unwrap()
                .set_value(&speed.to_string());
        }
    });

    if let ParameterSchema {
        id,
        value: ValueSchema::Speed,
        ..
    } = &props.schema
    {
        let speed = props.value.as_ref().and_then(|v| v.number()).unwrap_or(1.0);
        let raw = speed_to_raw(speed);

        html! {
            <div class="slider-control">
                <input
                    ref={slider_ref}
                    type="range" min="-2.0" max="2.0" step="0.05"
                    value={raw.to_string()}
                    {oninput} />
                <input
                    ref={value_display_ref}
                    type="text"
                    value={format!("{:.2}x", speed)}
                    disabled=true
                    class="value-display" />
                <input
                    ref={hidden_ref}
                    name={id.clone()}
                    type="hidden"
                    value={speed.to_string()} />
            </div>
        }
    } else {
        html!()
    }
}
