pub(crate) mod color_control;
mod debouncer;
pub(crate) mod select_control;
pub(crate) mod slider_control;
pub(crate) mod speed_control;

use std::{collections::HashMap, time::Duration};

use animation_api::schema::{Configuration, ConfigurationSchema, ParameterSchema, ValueSchema};
use log::error;
use rustmas_webapi_client::{ParameterValue, RustmasApiClient};
use wasm_bindgen::JsCast;
use web_sys::{
    Event, EventTarget, FocusEvent, FormData, HtmlFormElement, HtmlInputElement, HtmlSelectElement,
    InputEvent,
};
use yew::{html, Callback, Component, Context, Html, Properties};

use color_control::ColorParameterControl;
use select_control::SelectParameterControl;
use slider_control::SliderParameterControl;
use speed_control::SpeedParameterControl;

use self::debouncer::Debouncer;

fn get_form(target: Option<EventTarget>) -> Option<HtmlFormElement> {
    target
        .clone()
        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        .and_then(|e| e.form())
        .or(target
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            .and_then(|e| e.form()))
}

#[derive(Properties, PartialEq, Clone)]
pub struct ParameterControlProps {
    pub schema: ParameterSchema,
    pub value: Option<ParameterValue>,
    pub dummy_update: usize,
}

pub struct ParameterControlList {
    change_debouncer: Debouncer,
    dummy_update: usize,
}

#[derive(Clone, PartialEq, Properties)]
pub struct ParameterControlListProps {
    pub name: String,
    pub schema: ConfigurationSchema,
    pub values: HashMap<String, ParameterValue>,
    pub update_values: Callback<Option<Configuration>>,
    pub parameters_dirty: Callback<bool>,
}

pub enum ParameterControlListMsg {
    SaveParams(FocusEvent),
    ValuesChanged {
        form: Option<HtmlFormElement>,
        force: bool,
    },
    RestoreParams,
    ReloadAnimation,
}

impl Component for ParameterControlList {
    type Message = ParameterControlListMsg;
    type Properties = ParameterControlListProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            change_debouncer: Debouncer::new(Duration::from_millis(100)),
            dummy_update: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let (api, _) = ctx
            .link()
            .context::<RustmasApiClient>(Callback::noop())
            .expect("gateway to be created");

        match msg {
            ParameterControlListMsg::SaveParams(e) => {
                e.prevent_default();

                let parameters_dirty = ctx.props().parameters_dirty.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api.save_params().await {
                        Ok(_) => parameters_dirty.emit(false),
                        Err(e) => error!("Failed to save parameters, reason: {}", e),
                    }
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
                            serde_json::from_str::<ParameterValue>(
                                &form_data.get(&p.id).as_string().unwrap(),
                            )
                            .unwrap(),
                        )
                    })
                    .collect::<HashMap<_, _>>();

                ctx.props().parameters_dirty.emit(true);
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(e) = api.set_params(&params).await {
                        error!("Failed to update parameters, reason: {}", e);
                    }
                });
                false
            }
            ParameterControlListMsg::RestoreParams => {
                let update_values = ctx.props().update_values.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api.reset_params().await {
                        Ok(params) => update_values.emit(Some(params)),
                        Err(e) => error!("Failed to reset parameters, reason: {}", e),
                    }
                });

                self.dummy_update += 1;

                false
            }
            ParameterControlListMsg::ReloadAnimation => {
                let update_values = ctx.props().update_values.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api.reload_animation().await {
                        Ok(params) => update_values.emit(Some(params)),
                        Err(e) => error!("Failed to reload animation, reason: {}", e),
                    }
                });

                self.dummy_update += 1;

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        html! {
            <section class="parameter-control-list">
                <h2>{&ctx.props().name}</h2>
                <datalist id="warmWhites">
                    {(2200..=2800).step_by(100).map(lightfx::Color::kelvin).into_iter().map(|c| html! {
                        <option value={c.to_hex_string()}></option>
                    }).collect::<Html>()}
                </datalist>
                {
                    if !ctx.props().schema.parameters.is_empty() { html! {
                        <form
                            onsubmit={ctx.link().callback(Self::Message::SaveParams)}
                            oninput={ctx.link().callback(|e: InputEvent| Self::Message::ValuesChanged { form: get_form(e.target()), force: false })}
                            onchange={ctx.link().callback(|e: Event| Self::Message::ValuesChanged { form: get_form(e.target()), force: true })}>
                            {
                                ctx.props().schema.parameters.iter().cloned().map(|schema| {
                                    let value = ctx.props().values.get(&schema.id).cloned();
                                    let dummy_update = self.dummy_update;
                                    html! {
                                    <div class={match schema.value {
                                        ValueSchema::Color => "parameter-control color-control",
                                        _ => "parameter-control",
                                    }}>
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
                                                ValueSchema::Enum {..} => html!{<SelectParameterControl {schema} {value} {dummy_update} />},
                                                ValueSchema::Color => html!{<ColorParameterControl {schema} {value} {dummy_update} />},
                                                ValueSchema::Number {..} | ValueSchema::Percentage => {
                                                    html!{<SliderParameterControl {schema} {value} {dummy_update} />}
                                                },
                                                ValueSchema::Speed => html!{<SpeedParameterControl {schema} {value} {dummy_update} />}
                                            }
                                        }
                                    </div>
                                }}).collect::<Html>()
                            }
                            <div class="parameter-control buttons">
                                <input type="button" value="Reload" onclick={ctx.link().callback(|_| Self::Message::ReloadAnimation)} />
                                <input type="button" value="Reset" onclick={ctx.link().callback(|_| Self::Message::RestoreParams)} />
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
