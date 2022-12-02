mod color_control;
mod debouncer;
mod select_control;
mod slider_control;

use std::{collections::HashMap, time::Duration};

use lightfx::{
    parameter_schema::{ParameterValue, ParametersSchema},
    schema::Parameter,
};
use wasm_bindgen::JsCast;
use web_sys::{
    Event, EventTarget, FocusEvent, FormData, HtmlFormElement, HtmlInputElement, InputEvent,
};
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::api;
use color_control::ColorParameterControl;
use select_control::SelectParameterControl;
use slider_control::SliderParameterControl;

use self::debouncer::Debouncer;

fn get_form(target: Option<EventTarget>) -> Option<HtmlFormElement> {
    target
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .and_then(|e| e.form())
}

#[derive(Properties, PartialEq, Clone)]
pub struct ParameterControlProps {
    schema: Parameter,
    value: Option<serde_json::Value>,
}

pub struct ParameterControlList {
    change_debouncer: Debouncer,
}

#[derive(Clone, PartialEq, Properties)]
pub struct ParameterControlListProps {
    pub schema: ParametersSchema,
    pub values: HashMap<String, serde_json::Value>,
}

pub enum ParameterControlListMsg {
    SaveParams(FocusEvent),
    ValuesChanged {
        form: Option<HtmlFormElement>,
        force: bool,
    },
}

impl Component for ParameterControlList {
    type Message = ParameterControlListMsg;
    type Properties = ParameterControlListProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            change_debouncer: Debouncer::new(Duration::from_millis(100)),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let (api, _) = ctx
            .link()
            .context::<api::Gateway>(Callback::noop())
            .expect("gateway to be created");

        match msg {
            ParameterControlListMsg::SaveParams(e) => {
                e.prevent_default();

                wasm_bindgen_futures::spawn_local(async move {
                    let _ = api.save_params().await;
                });
                false
            }
            ParameterControlListMsg::ValuesChanged { form, force } => {
                if !force && !self.change_debouncer.poll() {
                    return false;
                }

                let form_data = FormData::new_with_form(&form.unwrap()).unwrap();
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

                wasm_bindgen_futures::spawn_local(async move {
                    let _ = api.set_params(&params).await;
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        html! {
            <section class="parameter-control-list">
                <datalist id="warmWhites">
                    {(2200..=2800).step_by(100).map(lightfx::Color::kelvin).into_iter().map(|c| html! {
                        <option value={c.to_hex_string()}></option>
                    }).collect::<Html>()}
                </datalist>
                {
                    if ctx.props().schema.parameters.len() != 0 { html! {
                        <form
                            onsubmit={ctx.link().callback(|e| Self::Message::SaveParams(e))}
                            oninput={ctx.link().callback(|e: InputEvent| Self::Message::ValuesChanged { form: get_form(e.target()), force: false })}
                            onchange={ctx.link().callback(|e: Event| Self::Message::ValuesChanged { form: get_form(e.target()), force: true })}>
                            {
                                ctx.props().schema.parameters.iter().cloned().map(|schema| {
                                    let value = ctx.props().values.get(&schema.id).cloned();
                                    html! {
                                    <div class="parameter-control">
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
                            <div class="parameter-control">
                                <input type="submit" value="Save" />
                            </div>
                        </form>
                    }} else { html! {
                        <p>{"This animation does not have any parameters"}</p>
                    }}
                }
            </section>
        }
    }
}
