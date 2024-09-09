use animation_api::schema::ValueSchema;
use yew::html;

use crate::controls::{
    color_control::ColorParameterControl, select_control::SelectParameterControl,
    slider_control::SliderParameterControl, speed_control::SpeedParameterControl,
};

use super::ParameterControlProps;

#[yew::function_component(ParameterControl)]
pub fn parameter_control(props: &ParameterControlProps) -> Html {
    let class = match props.schema.value {
        ValueSchema::Color => "parameter-control color-control",
        _ => "parameter-control",
    };

    html! {
        <div {class}>
            <h3>{ &props.schema.name }</h3>
            {
                if let Some(description) = &props.schema.description {
                    html! {
                        <p>{ description }</p>
                    }
                } else {
                    html!{}
                }
            }
            {
                match props.schema.value {
                    ValueSchema::Enum {..} => html!{<SelectParameterControl schema={props.schema.clone()} value={props.value.clone()} dummy_update={props.dummy_update} />},
                    ValueSchema::Color {..} => html!{<ColorParameterControl schema={props.schema.clone()} value={props.value.clone()} dummy_update={props.dummy_update} />},
                    ValueSchema::Number {..} | ValueSchema::Percentage => {
                        html!{<SliderParameterControl schema={props.schema.clone()} value={props.value.clone()} dummy_update={props.dummy_update} />}
                    },
                    ValueSchema::Speed {..} => html!{<SpeedParameterControl schema={props.schema.clone()} value={props.value.clone()} dummy_update={props.dummy_update} />},
                }
            }
        </div>
    }
}
