mod animations;
mod controls;
mod settings;
mod utils;

use std::error::Error;

use animations::AnimationControl;
use rustmas_webapi_client::RustmasApiClient;
use settings::Settings;
use url::Url;
use yew::prelude::*;
use yew_router::{prelude::Link, BrowserRouter, Routable, Switch};

#[derive(Debug, Clone, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Home,
    #[at("/settings")]
    SettingsMain,
    #[at("/settings/:section")]
    Settings { section: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

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

    html! {
        <ContextProvider<RustmasApiClient> context={api.clone()}>
            <BrowserRouter>
                <header>
                    <h1>{"Rustmas"}</h1>
                    <Link<Route> to={Route::SettingsMain}>
                        <img class="button" src="/settings.png" alt="Settings" />
                    </Link<Route>>
                </header>
                <div class="content">
                    <Switch<Route> render={switch} />
                </div>
            </BrowserRouter>
        </ContextProvider<RustmasApiClient>>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <AnimationControl /> },
        Route::Settings { section } => html! { <Settings section={section} /> },
        Route::SettingsMain => html! { <Settings section={None::<String>} /> },
        Route::NotFound => html! { <h2> {"Not found"} </h2> },
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Error));
    yew::Renderer::<App>::new().render();

    Ok(())
}
