mod controls;
mod settings;
mod utils;

#[cfg(feature = "visualizer")]
mod visualizer;

#[cfg(not(feature = "visualizer"))]
mod dummy;

use std::error::Error;

use log::error;
use rustmas_webapi_client::{Animation, Configuration, RustmasApiClient};
use url::Url;
use wasm_bindgen::JsCast;
use web_sys::HtmlAnchorElement;
use yew::prelude::*;

use crate::controls::ParameterControlList;
#[cfg(not(feature = "visualizer"))]
use crate::dummy::Dummy as Visualizer;
use crate::settings::SettingsModal;
#[cfg(feature = "visualizer")]
use crate::visualizer::Visualizer;

fn create_api() -> RustmasApiClient {
    let api_url = if cfg!(feature = "local") {
        Url::parse("http://127.0.0.1:8081").unwrap()
    } else if let Some(url) = web_sys::window().and_then(|w| w.location().href().ok()) {
        Url::parse(&url).and_then(|u| u.join("api/")).unwrap()
    } else {
        Url::parse("http://127.0.0.1:8081").unwrap()
    };
    RustmasApiClient::new(api_url)
}

fn window_width() -> i32 {
    web_sys::window()
        .and_then(|w| w.screen().ok())
        .and_then(|s| s.avail_width().ok())
        .unwrap_or_default()
}

#[yew::function_component(AnimationSelector)]
pub fn animation_selector() -> Html {
    let api = create_api();
    let animation_list = yew::use_state::<Option<Vec<Animation>>, _>(|| None);
    let animation = yew::use_state::<Option<Configuration>, _>(|| None);
    let modal_open_dummy = yew::use_state(|| 0);
    let dirty = yew::use_mut_ref(|| false);

    let animation_id = animation
        .as_ref()
        .map(|a| a.id.as_str())
        .unwrap_or_default();

    let toggle_settings = Callback::from({
        let modal_open_dummy = modal_open_dummy.clone();
        move |_| {
            modal_open_dummy.set(*modal_open_dummy + 1);
        }
    });

    let turn_off = Callback::from({
        let api = api.clone();
        let animation = animation.clone();
        move |_| {
            let api = api.clone();
            let animation = animation.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api.turn_off().await {
                    Ok(_) => animation.set(None),
                    Err(e) => error!("Failed to turn off animation, reason: {}", e),
                }
            });
        }
    });

    let discover = Callback::from({
        let api = api.clone();
        let animation_list = animation_list.clone();
        move |_| {
            let api = api.clone();
            let animation_list = animation_list.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api.discover_animations().await {
                    Ok(mut new_animations) => {
                        new_animations.sort_by(|a, b| a.name.cmp(&b.name));
                        animation_list.set(Some(new_animations));
                    }
                    Err(e) => error!("Failed to discover animations, reason: {}", e),
                }
            });
        }
    });

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

    let switch_animation = Callback::from({
        let api = api.clone();
        let animation = animation.clone();
        let dirty = dirty.clone();
        move |event: MouseEvent| {
            if *dirty.borrow() {
                let response = web_sys::window()
                    .and_then(|w| {
                        w.confirm_with_message(
                            "You have unsaved changes that will be lost. Continue?",
                        )
                        .ok()
                    })
                    .unwrap_or(false);
                if !response {
                    return;
                }
            }

            let Some(animation_id) = event
                .target()
                .and_then(|t| t.dyn_into::<HtmlAnchorElement>().ok())
                .and_then(|a| a.get_attribute("data-animation-id"))
            else {
                error!("Could not get animation id from anchor element");
                return;
            };

            let api = api.clone();
            let animation = animation.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api.switch_animation(animation_id).await {
                    Ok(new_animation) => animation.set(Some(new_animation)),
                    Err(e) => error!("Failed to switch animations, reason: {}", e),
                }
            });
        }
    });

    let loaded_parameters = Callback::from({
        let animation = animation.clone();
        let dirty = dirty.clone();
        move |new_animation| {
            animation.set(new_animation);
            *dirty.borrow_mut() = false;
        }
    });

    let parameters_dirty = Callback::from({
        let dirty = dirty.clone();
        move |new_dirty| *dirty.borrow_mut() = new_dirty
    });

    if animation_list.is_none() {
        let api = api.clone();
        let animation_list = animation_list.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api.list_animations().await {
                Ok(mut new_animations) => {
                    new_animations.sort_by(|a, b| a.name.cmp(&b.name));
                    animation_list.set(Some(new_animations));
                }
                Err(e) => error!("Failed to load animations, reason: {}", e),
            }
        });
    }

    if animation.is_none() {
        let api = api.clone();
        let animation = animation.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api.get_params().await {
                Ok(new_animation) => animation.set(new_animation),
                Err(e) => error!("Failed to load parameters, reason: {}", e),
            }
        });
    }

    html! {
        <ContextProvider<RustmasApiClient> context={api.clone()}>
        <>
            <header>
                <h1>{"Rustmas"}</h1>
                <a href="#settings" class="button" onclick={toggle_settings}>
                    <img src="/settings.png" alt="Settings" />
                </a>
            </header>
            <div class="content">
                <nav>
                    <ul>
                        <li><a onclick={turn_off}>{ "⏻ Off" }</a></li>
                        <li><a onclick={discover}>{ "⟳ Refresh list" }</a></li>
                        <li><a onclick={restart_events}>{ "⟳ Restart events" }</a></li>
                        <hr />
                        {
                            if let Some(ref animations) = *animation_list {
                                animations.iter().map(|animation| html! {
                                    <li class={
                                        if animation.id == animation_id {
                                            "selected"
                                        } else {
                                            ""
                                        }
                                    }>
                                        <a onclick={switch_animation.clone()} data-animation-id={animation.id.clone()}>
                                            { animation.name.clone() }
                                        </a>
                                    </li>
                                }).collect::<Html>()
                            } else {
                                html! { <p> {"Loading animations..."} </p> }
                            }
                        }
                    </ul>
                </nav>
                {
                    #[expect(clippy::let_unit_value)]
                    if window_width() > 640 {
                        html!(<Visualizer />)
                    } else {
                        html!()
                    }
                }
                {
                    if let Some(ref parameters) = *animation {
                        html! {
                            <ParameterControlList
                                name={parameters.name.clone()}
                                schema={parameters.schema.clone()}
                                values={parameters.values.clone()}
                                update_values={loaded_parameters}
                                parameters_dirty={parameters_dirty} />
                        }
                    } else {
                        html!{
                            <div class="parameter-control-list">
                                <h2>{ "Off" }</h2>
                                <p>{ "Select an animation from the list" }</p>
                            </div>
                        }
                    }
                }
            </div>
            <SettingsModal open_dummy={*modal_open_dummy} />
        </>
        </ContextProvider<RustmasApiClient>>
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Error));
    yew::start_app::<AnimationSelector>();

    Ok(())
}
