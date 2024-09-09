use std::{collections::HashMap, time::Duration};

use animation_api::schema::ConfigurationSchema;
use log::error;
use rustmas_webapi_client::{Configuration, ParameterValue, RustmasApiClient};
use web_sys::{FormData, HtmlFormElement};
use yew::{html, Callback, Event, FocusEvent, Html, InputEvent, Properties};

use crate::{
    controls::{debouncer::Debouncer, parameter_control::ParameterControl},
    utils,
};

#[derive(Clone, PartialEq, Properties)]
pub struct ParameterControlListProps {
    pub name: String,
    pub schema: ConfigurationSchema,
    pub values: HashMap<String, ParameterValue>,
    pub update_values: Callback<Option<Configuration>>,
    pub parameters_dirty: Callback<bool>,
}

fn build_parameter_update(
    schema: &ConfigurationSchema,
    form: &HtmlFormElement,
) -> HashMap<String, ParameterValue> {
    let form_data = FormData::new_with_form(form).unwrap();
    schema
        .parameters
        .iter()
        .map(|p| {
            (
                p.id.clone(),
                serde_json::from_str::<ParameterValue>(&form_data.get(&p.id).as_string().unwrap())
                    .unwrap(),
            )
        })
        .collect()
}

#[yew::function_component(ParameterControlList)]
pub fn parameter_control_list(props: &ParameterControlListProps) -> Html {
    let api = yew::use_context::<RustmasApiClient>().expect("gateway to be created");
    let dummy_update = yew::use_mut_ref(|| 0);

    let save_changes = Callback::from({
        let api = api.clone();
        let parameters_dirty = props.parameters_dirty.clone();
        move |event: FocusEvent| {
            event.prevent_default();

            let api = api.clone();
            let parameters_dirty = parameters_dirty.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api.save_params().await {
                    Ok(_) => parameters_dirty.emit(false),
                    Err(err) => error!("Failed to save parameters, reason: {}", err),
                }
            });
        }
    });

    let values_changed = {
        let api = api.clone();
        let schema = props.schema.clone();
        let change_debouncer = yew::use_mut_ref(|| Debouncer::new(Duration::from_millis(100)));
        let parameters_dirty = props.parameters_dirty.clone();
        move |form: Option<HtmlFormElement>, force: bool| {
            if !force && change_debouncer.borrow_mut().poll() {
                return;
            }

            let Some(form) = form else {
                error!("Cannot access html form");
                return;
            };

            parameters_dirty.emit(true);

            let api = api.clone();
            let params = build_parameter_update(&schema, &form);
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = api.set_params(&params).await {
                    error!("Failed to update parameters, reason: {}", e);
                }
            });
        }
    };

    let oninput = Callback::from({
        let values_changed = values_changed.clone();
        move |event: InputEvent| {
            values_changed(utils::get_form(event.target()), false);
        }
    });

    let onchange = Callback::from({
        move |event: Event| {
            values_changed(utils::get_form(event.target()), true);
        }
    });

    let restore_params = Callback::from({
        let api = api.clone();
        let update_values = props.update_values.clone();
        let dummy_update = dummy_update.clone();
        move |_| {
            let api = api.clone();
            let update_values = update_values.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api.reset_params().await {
                    Ok(params) => update_values.emit(Some(params)),
                    Err(e) => error!("Failed to reset parameters, reason: {}", e),
                }
            });

            *dummy_update.borrow_mut() += 1;
        }
    });

    let reload_animation = Callback::from({
        let api = api.clone();
        let update_values = props.update_values.clone();
        let dummy_update = dummy_update.clone();
        move |_| {
            let api = api.clone();
            let update_values = update_values.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api.reload_animation().await {
                    Ok(params) => update_values.emit(Some(params)),
                    Err(e) => error!("Failed to reload animation, reason: {}", e),
                }
            });

            *dummy_update.borrow_mut() += 1;
        }
    });

    html! {
        <section class="parameter-control-list">
            <datalist id="warmWhites">
                {(2200..=2800).step_by(100).map(lightfx::Color::kelvin).into_iter().map(|c| html! {
                    <option value={c.to_hex_string()}></option>
                }).collect::<Html>()}
            </datalist>
            <h2>{&props.name}</h2>
            {
                if !props.schema.parameters.is_empty() {
                    html! {
                        <form onsubmit={save_changes} {oninput} {onchange}>
                            {
                                props.schema.parameters.iter()
                                    .map(|schema| html! {
                                        <ParameterControl
                                            schema={schema.clone()}
                                            value={props.values.get(&schema.id).cloned()}
                                            dummy_update={*dummy_update.borrow()} />
                                    }).collect::<Html>()
                            }
                            <div class="parameter-control buttons">
                                <input type="button" value="Reload" onclick={reload_animation} />
                                <input type="button" value="Reset" onclick={restore_params} />
                                <input type="submit" value="Save" />
                            </div>
                        </form>
                    }
                } else {
                    html! {
                        <p>{"This animation does not have any parameters"}</p>
                    }
                }
            }
        </section>
    }
}
