mod animations;
mod events;

use animations::AnimationsSettings;
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

struct Section {
    id: &'static str,
    name: &'static str,
    html: Box<dyn Fn() -> Html>,
}

#[yew::function_component(Settings)]
pub fn settings(props: &Props) -> Html {
    let navigator = use_navigator().expect("to have a navigator");
    let sections = [
        Section {
            id: "events",
            name: "Events",
            html: Box::new(|| html! { <EventsSettings /> }),
        },
        Section {
            id: "animations",
            name: "Animations",
            html: Box::new(|| html! { <AnimationsSettings /> }),
        },
    ];
    let active_section_id = props.section.as_deref().unwrap_or("events");
    let Some(active_section) = sections.iter().find(|s| s.id == active_section_id) else {
        navigator.replace(&Route::NotFound);
        return html! {};
    };

    html! {
        <>
            <nav>
                <h2>{ "Settings" }</h2>
                <ul>
                    {
                        sections.iter().map(|s| {
                            html! {
                            <li class={
                                if s.id == active_section.id {
                                    "selected"
                                } else {
                                    ""
                                }
                            }>
                                <Link<Route> to={Route::Settings { section: s.id.to_string() }}>{s.name.to_owned()}</Link<Route>>
                            </li>
                            }
                        }).collect::<Html>()
                    }
                    <li><Link<Route> to={Route::Home}>{ "Back" }</Link<Route>></li>
                </ul>
            </nav>
            <section class="parameter-control-list settings">
                { (*active_section.html)() }
            </section>
        </>
    }
}
