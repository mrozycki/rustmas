use animation_api::schema::{ParameterSchema, ValueSchema};
use web_sys::HtmlSelectElement;
use yew::{Html, html};

use super::ParameterControlProps;

#[yew::function_component(SelectParameterControl)]
pub fn select_parameter_control(props: &ParameterControlProps) -> Html {
    let select_ref = yew::use_node_ref();

    yew::use_effect({
        let select_ref = select_ref.clone();
        move || {
            select_ref
                .cast::<HtmlSelectElement>()
                .and_then(|elem| elem.get_attribute("data-selected-index"))
                .and_then(|attr| attr.parse::<i32>().ok())
                .and_then(|index| {
                    select_ref.cast::<HtmlSelectElement>().map(|elem| {
                        elem.set_selected_index(index);
                    })
                });

            // Do nothing on cleanup
            || {}
        }
    });

    if let ParameterSchema {
        id,
        value: ValueSchema::Enum { values },
        ..
    } = &props.schema
    {
        let selected_index = props
            .value
            .as_ref()
            .and_then(|v| v.enum_option())
            .and_then(|v| values.iter().position(|opt| opt.value == v))
            .unwrap_or(0);

        html! {
            <select
                ref={select_ref}
                name={id.clone()}
                class="parameter-select"
                data-selected-index={selected_index.to_string()}>
                {
                    values.iter()
                        .map(|item| {
                            let value = format!("\"{}\"", item.value);
                            html!(<option {value}>{item.name.clone()}</option>)})
                        .collect::<Html>()
                }
            </select>
        }
    } else {
        html!()
    }
}
