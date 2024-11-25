mod events;

use events::EventsSettings;
use yew::Properties;
use yew::{html, prelude::Html};
use yew_router::hooks::use_navigator;
use yew_router::prelude::Link;

use crate::Route;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub section: Option<String>,
}

#[yew::function_component(Settings)]
pub fn settings(props: &Props) -> Html {
    let navigator = use_navigator().expect("to have a navigator");
    html! {
        <>
            <nav>
                <h2>{ "Settings" }</h2>
                <ul>
                    <li class="selected"><Link<Route> to={Route::Settings { section: "events".to_owned() }}>{ "Events" }</Link<Route>></li>
                    <li><Link<Route> to={Route::Home}>{ "Back" }</Link<Route>></li>
                </ul>
            </nav>
            <section class="parameter-control-list">
                {
                    match props.section.as_deref() {
                        None | Some("events") => html!{ <EventsSettings /> },
                        _ => {
                            navigator.replace(&Route::NotFound);
                            html! {}
                        }

                    }
                }
            </section>
        </>
    }
}
