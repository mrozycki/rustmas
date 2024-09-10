use std::collections::HashMap;

use animation_api::schema::ParameterSchema;
use log::error;
use rustmas_webapi_client::{Configuration, ParameterValue, RustmasApiClient};
use wasm_bindgen::JsCast;
use web_sys::{DomRect, Event, FormData, HtmlDialogElement, InputEvent, MouseEvent};
use yew::{html, prelude::Html, Callback, Properties};

use crate::controls::ParameterControl;
use crate::utils;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub open_dummy: usize,
}

#[yew::function_component(SettingsModal)]
pub fn settings_modal(props: &Props) -> Html {
    let api = yew::use_context::<RustmasApiClient>().expect("gateway to be open");
    let schema = yew::use_state::<Option<Vec<Configuration>>, _>(|| None);
    let open_dummy = yew::use_mut_ref(|| 0);
    let modal_ref = yew::use_node_ref();

    let open_modal = {
        let api = api.clone();
        let modal_ref = modal_ref.clone();
        let schema = schema.clone();
        let props_open_dummy = props.open_dummy;
        let open_dummy = open_dummy.clone();

        move || {
            let api = api.clone();
            let schema = schema.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api.events_schema().await {
                    Ok(new_schema) => {
                        schema.set(Some(new_schema));
                    }
                    Err(e) => error!("Could not load event generator parameter schema: {}", e),
                }
            });

            let modal = modal_ref.cast::<HtmlDialogElement>().unwrap();
            let _ = modal.show_modal();
            *open_dummy.borrow_mut() = props_open_dummy;
        }
    };

    let close_modal = {
        let modal_ref = modal_ref.clone();
        move || {
            modal_ref.cast::<HtmlDialogElement>().unwrap().close();
        }
    };

    let values_changed = {
        let api = api.clone();
        let schema = schema.clone();
        move |form| {
            let Some(ref schema) = *schema else {
                error!("Values changed without schema loaded");
                return;
            };

            let Some(form) = form else {
                error!("Could not access settings form");
                return;
            };

            let form_data = FormData::new_with_form(&form).unwrap();
            let params = schema
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
                                    serde_json::from_str::<ParameterValue>(
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
                .collect::<HashMap<_, _>>();

            let api = api.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = api.set_events_params(&params).await {
                    error!("Failed to set event generator parameters: {}", e);
                }
            });
        }
    };

    let restart_events = Callback::from({
        let api = api.clone();
        move |_| {
            let api = api.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = api.restart_events().await {
                    error!("Failed to restart events: {}", e);
                }
            });
        }
    });

    let onshow = Callback::from({
        let open_modal = open_modal.clone();
        move |_| {
            open_modal();
        }
    });

    let onclick = Callback::from({
        let modal_ref = modal_ref.clone();
        move |event| {
            let modal = modal_ref.cast::<HtmlDialogElement>().unwrap();
            let bounding_box = modal.get_bounding_client_rect();

            if outside(&event, &bounding_box) {
                modal.close();
            }
        }
    });

    let oninput = Callback::from({
        let values_changed = values_changed.clone();
        move |event: InputEvent| {
            values_changed(utils::get_form(event.target()));
        }
    });

    let onchange = Callback::from({
        move |event: Event| {
            values_changed(utils::get_form(event.target()));
        }
    });

    if props.open_dummy > *open_dummy.borrow() {
        open_modal();
    }

    html! {
        <dialog
            ref={modal_ref}
            {onclick} {onshow}
            class="settings">

            <header>
                <h2>{ "Settings" }</h2>
                <a href="#" class="button" onclick={Callback::from(move |_| close_modal())}>{ "X" }</a>
            </header>
            <button onclick={restart_events}>{"⟳ Restart events"}</button>
            <form {oninput} {onchange}>
                {
                    if let Some(ref schema) = *schema {
                        schema.iter().map(|evg| {
                            if evg.schema.parameters.is_empty() {
                                html! { }
                            } else {
                                html! {
                                    <>
                                        <h3>{ &evg.name }</h3>
                                        {
                                            evg.schema.parameters.iter().cloned().map(|schema| {
                                                let Some(value) = evg.values.get(&schema.id) else {
                                                    return html!{};
                                                };
                                                let schema = ParameterSchema {
                                                    id: format!("{}.{}", evg.id, schema.id),
                                                    ..schema
                                                };
                                                html!{ <ParameterControl {schema} value={value.clone()} dummy_update=0 /> }
                                            }).collect::<Html>()
                                        }
                                    </>
                                }
                            }
                        }).collect::<Html>()
                    } else {
                        html! { <p> { "Loading schema... " } </p> }
                    }
                }
            </form>
        </dialog>
    }
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
