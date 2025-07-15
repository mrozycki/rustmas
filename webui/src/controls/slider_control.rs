use animation_api::schema::ValueSchema;
use web_sys::HtmlInputElement;
use yew::{html, Callback, Html};

use super::ParameterControlProps;

fn display_value(value: f64, percent: bool) -> String {
    if percent {
        format!("{}%", (value * 100.0) as i32)
    } else {
        format!("{value:.2}")
    }
}

#[yew::function_component(SliderParameterControl)]
pub fn slider_parameter_control(props: &ParameterControlProps) -> Html {
    let slider_ref = yew::use_node_ref();
    let value_display_ref = yew::use_node_ref();

    let oninput = Callback::from({
        let slider_ref = slider_ref.clone();
        let value_display_ref = value_display_ref.clone();
        let percent = matches!(props.schema.value, ValueSchema::Percentage);

        move |_| {
            let value = slider_ref
                .cast::<HtmlInputElement>()
                .map(|elem| elem.value())
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);

            value_display_ref
                .cast::<HtmlInputElement>()
                .unwrap()
                .set_value(&display_value(value, percent));
        }
    });

    let (min, max, step, percent) = match &props.schema.value {
        ValueSchema::Number { min, max, step } => (*min, *max, *step, false),
        ValueSchema::Percentage => (0.0, 1.0, 0.01, true),
        _ => return html!(),
    };

    let value = props
        .value
        .as_ref()
        .and_then(|v| v.number())
        .unwrap_or((max + min) / 2.0);

    html! {
        <div class="slider-control">
            <input
                ref={slider_ref}
                name={props.schema.id.clone()}
                type="range" min={min.to_string()} max={max.to_string()} step={step.to_string()}
                value={value.to_string()}
                {oninput} />
            <input
                ref={value_display_ref}
                type="text"
                value={display_value(value, percent)}
                disabled=true class="value-display" />
        </div>
    }
}
