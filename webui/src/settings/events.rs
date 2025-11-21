use std::collections::HashMap;

use animation_api::schema::ParameterSchema;
use log::error;
use rustmas_webapi_client::{Configuration, ParameterValue, RustmasApiClient};
use web_sys::{Event, FormData, InputEvent};
use yew::{Callback, html, prelude::Html};

use crate::controls::ParameterControl;
use crate::utils;

#[yew::function_component(EventsSettings)]
pub fn events_settings() -> Html {
    let api = yew::use_context::<RustmasApiClient>().expect("gateway to be open");
    let schema = yew::use_state::<Option<Vec<Configuration>>, _>(|| None);

    if schema.is_none() {
        wasm_bindgen_futures::spawn_local({
            let api = api.clone();
            let schema = schema.clone();
            async move {
                match api.events_schema().await {
                    Ok(new_schema) => {
                        schema.set(Some(new_schema));
                    }
                    Err(e) => error!("Could not load event generator parameter schema: {}", e),
                }
            }
        });
    }

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

    html! {
        <form {oninput} {onchange}>
            <button onclick={restart_events}>{"⟳ Restart events"}</button>
            {
                if let Some(ref schema) = *schema {
                    schema.iter().map(|evg| {
                        if evg.schema.parameters.is_empty() {
                            html! { }
                        } else {
                            html! {
                                <>
                                    <h2>{ &evg.name }</h2>
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
    }
}
