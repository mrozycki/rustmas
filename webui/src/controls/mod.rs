mod color_control;
mod select_control;
mod slider_control;

use std::collections::HashMap;

use lightfx::{
    parameter_schema::{ParameterValue, ParametersSchema},
    schema::Parameter,
};
use wasm_bindgen::JsCast;
use web_sys::{FocusEvent, FormData, HtmlFormElement};
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::api;
use color_control::ColorParameterControl;
use select_control::SelectParameterControl;
use slider_control::SliderParameterControl;

#[derive(Properties, PartialEq, Clone)]
pub struct ParameterControlProps {
    schema: Parameter,
    value: Option<serde_json::Value>,
}

pub struct ParameterControlList {}

#[derive(Clone, PartialEq, Properties)]
pub struct ParameterControlListProps {
    pub schema: ParametersSchema,
    pub values: HashMap<String, serde_json::Value>,
}

pub enum ParameterControlListMsg {
    SubmitInfo(FocusEvent),
}

impl Component for ParameterControlList {
    type Message = ParameterControlListMsg;
    type Properties = ParameterControlListProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ParameterControlListMsg::SubmitInfo(e) => {
                e.prevent_default();
                let form_data = FormData::new_with_form(
                    &e.target().unwrap().dyn_into::<HtmlFormElement>().unwrap(),
                )
                .unwrap();
                let params = ctx
                    .props()
                    .schema
                    .parameters
                    .iter()
                    .map(|p| {
                        (
                            p.id.clone(),
                            serde_json::from_str::<serde_json::Value>(
                                &form_data.get(&p.id).as_string().unwrap(),
                            )
                            .unwrap(),
                        )
                    })
                    .collect::<HashMap<_, _>>();
                let params = serde_json::to_value(&params).unwrap();

                let (api, _) = ctx
                    .link()
                    .context::<api::Gateway>(Callback::noop())
                    .expect("gateway to be created");

                wasm_bindgen_futures::spawn_local(async move {
                    let _ = api.set_params(&params).await;
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        html! {
            <div>
                <form onsubmit={ctx.link().callback(|e| Self::Message::SubmitInfo(e))}>
                    {
                        ctx.props().schema.parameters.iter().cloned().map(|schema| {
                            let value = ctx.props().values.get(&schema.id).cloned();
                            html! {
                            <div>
                                <h3>{ &schema.name }</h3>
                                {
                                    if let Some(description) = &schema.description {
                                        html! {
                                            <p>{ description }</p>
                                        }
                                    } else { html!{} }
                                }
                                {
                                    match schema.value {
                                        ParameterValue::Enum {..} => html!{<SelectParameterControl {schema} {value} />},
                                        ParameterValue::Color => html!{<ColorParameterControl {schema} {value} />},
                                        ParameterValue::Number {..} => html!{<SliderParameterControl {schema} {value} />},
                                    }
                                }
                            </div>
                        }}).collect::<Html>()
                    }
                    <input type="submit" value="Send" />
                </form>
            </div>
        }
    }
}
