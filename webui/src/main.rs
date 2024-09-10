mod animations;
mod controls;
mod settings;
mod utils;

use std::error::Error;

use animations::AnimationList;
use rustmas_webapi_client::RustmasApiClient;
use url::Url;
use yew::prelude::*;

use crate::animations::Visualizer;
use crate::controls::ParameterControlList;
use crate::settings::SettingsModal;

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

#[yew::function_component(App)]
pub fn app() -> Html {
    let api = create_api();
    let animation_id = yew::use_state::<Option<String>, _>(|| None);
    let modal_open_dummy = yew::use_state(|| 0);
    let dirty = yew::use_state(|| false);

    let toggle_settings = Callback::from({
        let modal_open_dummy = modal_open_dummy.clone();
        move |_| {
            modal_open_dummy.set(*modal_open_dummy + 1);
        }
    });

    let animation_switched_callback = Callback::from({
        let animation_id = animation_id.clone();
        let dirty = dirty.clone();
        move |new_animation_id: Option<String>| {
            animation_id.set(new_animation_id);
            dirty.set(false);
        }
    });

    let parameters_dirty = Callback::from({
        let dirty = dirty.clone();
        move |new_dirty| dirty.set(new_dirty)
    });

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
                <AnimationList dirty={*dirty} {animation_switched_callback} />
                {
                    if window_width() > 640 {
                        html!(<Visualizer />)
                    } else {
                        html!()
                    }
                }
                <ParameterControlList
                    animation_id={(*animation_id).clone()}
                    parameters_dirty={parameters_dirty} />
            </div>
            <SettingsModal open_dummy={*modal_open_dummy} />
        </>
        </ContextProvider<RustmasApiClient>>
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Error));
    yew::Renderer::<App>::new().render();

    Ok(())
}
