use std::collections::HashMap;

use animation_api::schema::{ParameterSchema, ValueSchema};
use itertools::Itertools;
use log::error;
use rustmas_webapi_client::{ParamsSchemaEntry, RustmasApiClient};
use serde_json::json;
use wasm_bindgen::JsCast;
use web_sys::{
    DomRect, Event, FormData, HtmlDialogElement, HtmlFormElement, InputEvent, MouseEvent,
};
use yew::{html, prelude::Html, Callback, Component, Context, NodeRef, Properties};

use crate::controls::color_control::ColorParameterControl;
use crate::controls::select_control::SelectParameterControl;
use crate::controls::slider_control::SliderParameterControl;
use crate::controls::speed_control::SpeedParameterControl;
use crate::utils;

#[derive(Default)]
pub struct SettingsModal {
    schema: Vec<ParamsSchemaEntry>,
    open_dummy: usize,
    modal_ref: NodeRef,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub open_dummy: usize,
}

pub enum Msg {
    Open,
    Close,
    Click(MouseEvent),
    ValuesChanged {
        form: Option<HtmlFormElement>,
        force: bool,
    },
    SchemaLoaded(Vec<ParamsSchemaEntry>),
}

fn get_api(ctx: &Context<SettingsModal>) -> RustmasApiClient {
    ctx.link()
        .context::<RustmasApiClient>(Callback::noop())
        .expect("gateway to be created")
        .0
}

fn outside(click: &MouseEvent, rect: &DomRect) -> bool {
    ((click.x() as f64) < rect.x()
        || (click.x() as f64) > rect.x() + rect.width()
        || (click.y() as f64) < rect.y()
        || (click.y() as f64) > rect.y() + rect.height())
        && click
            .target()
            .and_then(|t| t.dyn_into::<HtmlDialogElement>().ok())
            .is_some()
}

impl Component for SettingsModal {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let api = get_api(ctx);
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api.events_schema().await {
                Ok(schema) => link.send_message(Msg::SchemaLoaded(schema)),
                Err(e) => error!("Could not load event generator parameter schema: {}", e),
            }
        });

        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Open => {
                let api = get_api(ctx);
                let link = ctx.link().clone();

                wasm_bindgen_futures::spawn_local(async move {
                    match api.events_schema().await {
                        Ok(schema) => link.send_message(Msg::SchemaLoaded(schema)),
                        Err(e) => error!("Could not load event generator parameter schema: {}", e),
                    }
                });

                let modal = self.modal_ref.cast::<HtmlDialogElement>().unwrap();
                let _ = modal.show_modal();
                self.open_dummy = ctx.props().open_dummy;

                false
            }
            Msg::Close => {
                let modal = self.modal_ref.cast::<HtmlDialogElement>().unwrap();
                modal.close();

                false
            }
            Msg::Click(event) => {
                let modal = self.modal_ref.cast::<HtmlDialogElement>().unwrap();
                let bounding_box = modal.get_bounding_client_rect();

                if outside(&event, &bounding_box) {
                    modal.close();
                }

                false
            }
            Msg::ValuesChanged { form, .. } => {
                let form_data = FormData::new_with_form(&form.unwrap()).unwrap();
                let params = self
                    .schema
                    .iter()
                    .map(|evg| {
                        (
                            evg.id.clone(),
                            evg.schema
                                .parameters
                                .iter()
                                .map(|schema| {
                                    (
                                        schema.id.clone(),
                                        serde_json::from_str::<serde_json::Value>(
                                            &form_data
                                                .get(&format!("{}.{}", evg.id, schema.id))
                                                .as_string()
                                                .unwrap(),
                                        )
                                        .unwrap(),
                                    )
                                })
                                .collect::<HashMap<_, _>>(),
                        )
                    })
                    .map(|(id, values)| {
                        json!({
                            "id": id,
                            "values": values
                        })
                    })
                    .collect_vec();
                let params = serde_json::to_value(params).unwrap();

                let api = get_api(ctx);
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(e) = api.set_events_params(&params).await {
                        error!("Failed to set event generator parameters: {}", e);
                    }
                });

                false
            }
            Msg::SchemaLoaded(schema) => {
                self.schema = schema
                    .into_iter()
                    .sorted_by(|evg1, evg2| evg1.name.cmp(&evg2.name))
                    .collect();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        if ctx.props().open_dummy > self.open_dummy {
            link.send_message(Msg::Open);
        }

        html! {
            <dialog class="settings" ref={ self.modal_ref.clone() } onclick={link.callback(Msg::Click)} onshow={link.callback(|_| Msg::Open)}>
                <header>
                    <h2>{ "Settings" }</h2>
                    <a href="#" class="button" onclick={link.callback(|_| Msg::Close)}>{ "X" }</a>
                </header>
                <form
                    oninput={ctx.link().callback(|e: InputEvent| {
                        Msg::ValuesChanged { form: utils::get_form(e.target()), force: false }
                    })}
                    onchange={ctx.link().callback(|e: Event| {
                        Msg::ValuesChanged { form: utils::get_form(e.target()), force: true }
                    })}>
                    {
                        self.schema.iter().map(|evg| {
                            if evg.schema.parameters.is_empty() {
                                html! { }
                            } else {
                                html! {
                                    <>
                                        <h3>{ &evg.name }</h3>
                                        {
                                            evg.schema.parameters.iter().cloned().map(|schema| {
                                                let value = evg.values.get(&schema.id).unwrap_or(&json!(())).clone();
                                                let schema = ParameterSchema {
                                                    id: format!("{}.{}", evg.id, schema.id),
                                                    ..schema
                                                };
                                                error!("evg: {}, param: {}, value: {}", evg.name, &schema.id, value);
                                                let dummy_update = 0;
                                                match schema.value {
                                                    ValueSchema::Enum {..} => html!{<SelectParameterControl {schema} {value} {dummy_update} />},
                                                    ValueSchema::Color => html!{<ColorParameterControl {schema} {value} {dummy_update} />},
                                                    ValueSchema::Number {..} | ValueSchema::Percentage => {
                                                        html!{<SliderParameterControl {schema} {value} {dummy_update} />}
                                                    },
                                                    ValueSchema::Speed => html!{<SpeedParameterControl {schema} {value} {dummy_update} />}
                                                }
                                            }).collect::<Html>()
                                        }
                                    </>
                                }
                            }
                        }).collect::<Html>()
                    }
                </form>
            </dialog>
        }
    }
}
