use log::error;
use rustmas_webapi_client::{Animation, RustmasApiClient};
use wasm_bindgen::JsCast;
use web_sys::HtmlAnchorElement;
use yew::{Callback, Html, MouseEvent, html};
use yew_router::prelude::Link;

use crate::Route;

#[derive(Clone, Debug, PartialEq, yew::Properties)]
pub struct AnimationListProps {
    pub dirty: bool,
    pub animation_switched_callback: Callback<Option<String>>,
}

#[yew::function_component(AnimationList)]
pub fn animation_list(props: &AnimationListProps) -> Html {
    let api = yew::use_context::<RustmasApiClient>().expect("gateway to be open");
    let animation_list = yew::use_state::<Option<Vec<Animation>>, _>(|| None);
    let animation_id = yew::use_state::<Option<String>, _>(|| None);

    let turn_off = Callback::from({
        let api = api.clone();
        let animation_id = animation_id.clone();
        let animation_switched_callback = props.animation_switched_callback.clone();
        move |_| {
            let api = api.clone();
            let animation_id = animation_id.clone();
            let animation_switched_callback = animation_switched_callback.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api.turn_off().await {
                    Ok(_) => {
                        animation_id.set(None);
                        animation_switched_callback.emit(None);
                    }
                    Err(e) => error!("Failed to turn off animation, reason: {}", e),
                }
            });
        }
    });

    let switch_animation = Callback::from({
        let api = api.clone();
        let animation_id = animation_id.clone();
        let animation_switched_callback = props.animation_switched_callback.clone();
        let dirty = props.dirty;
        move |event: MouseEvent| {
            if dirty {
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

            let Some(new_animation_id) = event
                .target()
                .and_then(|t| t.dyn_into::<HtmlAnchorElement>().ok())
                .and_then(|a| a.get_attribute("data-animation-id"))
            else {
                error!("Could not get animation id from anchor element");
                return;
            };

            let api = api.clone();
            let animation_id = animation_id.clone();
            let animation_switched_callback = animation_switched_callback.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api.switch_animation(new_animation_id).await {
                    Ok(new_animation) => {
                        animation_id.set(Some(new_animation.id.clone()));
                        animation_switched_callback.emit(Some(new_animation.id));
                    }
                    Err(e) => error!("Failed to switch animations, reason: {}", e),
                }
            });
        }
    });

    if animation_list.is_none() {
        let api = api.clone();
        let animation_id = animation_id.clone();
        let animation_list = animation_list.clone();
        let animation_switched_callback = props.animation_switched_callback.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api.list_animations().await {
                Ok(mut response) => {
                    response.animations.sort_by(|a, b| a.name.cmp(&b.name));
                    animation_list.set(Some(response.animations));
                    animation_id.set(response.current_animation_id.clone());
                    animation_switched_callback.emit(response.current_animation_id);
                }
                Err(e) => error!("Failed to load animations, reason: {}", e),
            }
        });
    }

    html! {
        <nav>
            <ul>
                <li><a onclick={turn_off}>{ "⏻ Off" }</a></li>
                <li>
                    <Link<Route> to={Route::SettingsMain}>
                        <img class="button" src="/settings.png" alt="Settings" />
                        { " Settings"}
                    </Link<Route>>
                </li>
                <hr />
                {
                    if let Some(ref animations) = *animation_list {
                        animations.iter().map(|animation| html! {
                            <li class={
                                if animation_id.as_ref().is_some_and(|id| id == &animation.id) {
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
                        html! { <li> {"Loading animations..."} </li> }
                    }
                }
            </ul>
        </nav>
    }
}
