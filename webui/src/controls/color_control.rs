use animation_api::schema::{ParameterSchema, ValueSchema};
use web_sys::HtmlInputElement;
use yew::{html, Callback};

use super::ParameterControlProps;

#[yew::function_component(ColorParameterControl)]
pub fn color_parameter_control(props: &ParameterControlProps) -> Html {
    let input_ref = yew::use_node_ref();
    let hidden_ref = yew::use_node_ref();

    let onchange = Callback::from({
        let input_ref = input_ref.clone();
        let hidden_ref = hidden_ref.clone();
        move |_| {
            if let (Some(node), Some(hidden)) = (
                input_ref.cast::<HtmlInputElement>(),
                hidden_ref.cast::<HtmlInputElement>(),
            ) {
                hidden.set_value(
                    &serde_json::to_string(&lightfx::Color::from_hex_str(&node.value()).unwrap())
                        .unwrap(),
                );
            };
        }
    });

    if let ParameterSchema {
        id,
        value: ValueSchema::Color,
        ..
    } = &props.schema
    {
        let value = props
            .value
            .as_ref()
            .and_then(|v| v.color())
            .map(lightfx::Color::to_hex_string)
            .unwrap_or("#000000".to_owned());
        let value_hex = serde_json::to_string(&props.value).unwrap();

        html! {
            <>
                <input type="color" ref={input_ref} {onchange} {value} list="warmWhites" />
                <input name={id.clone()} type="hidden" ref={hidden_ref} value={value_hex}/>
            </>
        }
    } else {
        html!()
    }
}
