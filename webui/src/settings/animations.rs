use log::{error, warn};
use rustmas_webapi_client::{Animation, RustmasApiClient};
use wasm_bindgen::JsCast;
use web_sys::{FormData, HtmlFormElement};
use yew::{Callback, Html, SubmitEvent, html};

#[yew::function_component(AnimationsSettings)]
pub fn animations_settings() -> Html {
    let api = yew::use_context::<RustmasApiClient>().expect("gateway to be open");
    let animations = yew::use_state::<Option<Vec<Animation>>, _>(|| None);

    if animations.is_none() {
        wasm_bindgen_futures::spawn_local({
            let api = api.clone();
            let animations = animations.clone();
            async move {
                match api.list_animations().await {
                    Ok(mut new_animations) => {
                        new_animations
                            .animations
                            .sort_by(|a, b| a.name.cmp(&b.name));
                        animations.set(Some(new_animations.animations));
                    }
                    Err(e) => error!("Could not load available animations: {}", e),
                }
            }
        });
    }

    let discover = Callback::from({
        let api = api.clone();
        let animations = animations.clone();
        move |_| {
            let api = api.clone();
            let animations = animations.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api.discover_animations().await {
                    Ok(mut new_animations) => {
                        new_animations.sort_by(|a, b| a.name.cmp(&b.name));
                        animations.set(Some(new_animations));
                    }
                    Err(e) => error!("Failed to discover animations, reason: {}", e),
                }
            });
        }
    });

    let remove = |id: String| {
        Callback::from({
            let api = api.clone();
            let animations = animations.clone();
            let id = id.clone();
            move |_| {
                let api = api.clone();
                let animations = animations.clone();
                let id = id.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api.remove_animation(id).await {
                        Ok(mut new_animations) => {
                            new_animations.sort_by(|a, b| a.name.cmp(&b.name));
                            animations.set(Some(new_animations))
                        }
                        Err(e) => error!("Failed to remove animation, reason: {}", e),
                    }
                });
            }
        })
    };

    let animation_installed_callback = Callback::from({
        let animations = animations.clone();
        move |new_animations| {
            animations.set(Some(new_animations));
        }
    });

    if let Some(ref animations) = *animations {
        html! {
            <>
                <h2>{ "Install new animation" }</h2>
                <AnimationsPluginInstallForm {animation_installed_callback} />

                <h2>{ "Available animations" }</h2>
                <button onclick={discover}>{"⟳ Refresh list"}</button>
                <ul class="animation-list">
                {
                    animations.iter()
                        .map(|animation| html! {
                            <li>
                                { animation.name.clone() }
                                <button onclick={remove(animation.id.clone())}>{ "Remove" }</button>
                            </li>
                        })
                        .collect::<Html>()
                }
                </ul>
            </>
        }
    } else {
        html! { <p> { "Loading animations... " } </p> }
    }
}

#[derive(Clone, Debug, PartialEq, yew::Properties)]
pub struct AnimationPluginInstallFormProps {
    pub animation_installed_callback: Callback<Vec<Animation>>,
}

#[yew::function_component(AnimationsPluginInstallForm)]
pub fn animations_plugin_install_form(props: &AnimationPluginInstallFormProps) -> Html {
    let api = yew::use_context::<RustmasApiClient>().expect("gateway to be open");
    let uploading = yew::use_state(|| false);
    let error = yew::use_state(|| false);

    let install = Callback::from({
        let api = api.clone();
        let uploading = uploading.clone();
        let error = error.clone();
        let animation_installed_callback = props.animation_installed_callback.clone();
        move |submit: SubmitEvent| {
            let api = api.clone();
            let uploading = uploading.clone();
            let error = error.clone();
            let animation_installed_callback = animation_installed_callback.clone();
            submit.prevent_default();

            uploading.set(true);
            let Some(form) = submit
                .target()
                .and_then(|t| t.dyn_into::<HtmlFormElement>().ok())
            else {
                warn!("Submit target not a form");
                return;
            };
            let Ok(form) = FormData::new_with_form(&form) else {
                warn!("Failed to get plugin upload form data");
                return;
            };
            wasm_bindgen_futures::spawn_local(async move {
                match api.install_animation(form).await {
                    Ok(mut new_animations) => {
                        new_animations.sort_by(|a, b| a.name.cmp(&b.name));
                        animation_installed_callback.emit(new_animations);
                        uploading.set(false);
                        error.set(false);
                    }
                    Err(e) => {
                        error!("Failed to install animation, reason: {}", e);
                        uploading.set(false);
                        error.set(true);
                    }
                }
            });
        }
    });

    html! {
        if *uploading {
            { "Uploading..." }
        } else {
            if *error {
                { "Failed to install plugin" }
            }
            <form onsubmit={install}>
                <input id="plugin-file" name="file" type="file" />
                <input id="plugin-install-button" type="submit" value="+ Install animation" />
            </form>
        }
    }
}
