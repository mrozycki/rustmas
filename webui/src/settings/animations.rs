use log::error;
use rustmas_webapi_client::{Animation, RustmasApiClient};
use yew::{Callback, Html, html};

#[yew::function_component(AnimationsSettings)]
pub fn animations_settings() -> Html {
    let api = yew::use_context::<RustmasApiClient>().expect("gateway to be open");
    let animations = yew::use_state::<Option<Vec<Animation>>, _>(|| None);

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

    if let Some(ref animations) = *animations {
        html! {
            <>
                <button onclick={discover}>{"⟳ Refresh list"}</button>
                <ul class="animation-list">
                {
                    animations.iter()
                        .map(|animation| html! { <li>{ animation.name.clone() }</li> })
                        .collect::<Html>()
                }
                </ul>
            </>
        }
    } else {
        html! { <p> { "Loading animations... " } </p> }
    }
}
