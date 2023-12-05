mod api;
mod controls;

use std::error::Error;

use api::Gateway;
use log::error;
use yew::prelude::*;

use crate::controls::ParameterControlList;

enum Msg {
    LoadedAnimations(Vec<api::AnimationEntry>),
    SwitchAnimation(String),
    LoadedParameters(Option<api::Animation>),
    ParametersDirty(bool),
    TurnOff,
    Discover,
    RestartEvents,
}

#[derive(Default)]
struct AnimationSelector {
    api: api::Gateway,
    animations: Vec<api::AnimationEntry>,
    parameters: Option<api::Animation>,
    dirty: bool,
}

impl Component for AnimationSelector {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let api = if cfg!(feature = "local") {
            api::Gateway::new("http://127.0.0.1:8081")
        } else {
            api::Gateway::new("/api")
        };

        {
            let api = api.clone();
            let link = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api.list_animations().await {
                    Ok(animations) => link.send_message(Msg::LoadedAnimations(animations)),
                    Err(e) => error!("Failed to load animations, reason: {}", e),
                }
            });
        }

        {
            let api = api.clone();
            let link = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api.get_params().await {
                    Ok(params) => link.send_message(Msg::LoadedParameters(params)),
                    Err(e) => error!("Failed to load parameters, reason: {}", e),
                }
            });
        }

        Self {
            api,
            ..Default::default()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SwitchAnimation(animation_id) => {
                if self.dirty {
                    let response = web_sys::window()
                        .and_then(|w| {
                            w.confirm_with_message(
                                "You have unsaved changes that will be lost. Continue?",
                            )
                            .ok()
                        })
                        .unwrap_or(false);
                    if !response {
                        return false;
                    }
                }
                let link = ctx.link().clone();
                let api = self.api.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api.switch_animation(animation_id).await {
                        Ok(resp) => link.send_message(Msg::LoadedParameters(resp)),
                        Err(e) => error!("Failed to switch animations, reason: {}", e),
                    }
                });
                false
            }
            Msg::LoadedAnimations(mut animations) => {
                animations.sort_by(|a, b| a.name.cmp(&b.name));
                self.animations = animations;
                true
            }
            Msg::LoadedParameters(parameters) => {
                self.parameters = parameters;
                self.dirty = false;
                true
            }
            Msg::ParametersDirty(dirty) => {
                self.dirty = dirty;
                false
            }
            Msg::TurnOff => {
                let api = self.api.clone();
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api.turn_off().await {
                        Ok(_) => link.send_message(Msg::LoadedParameters(None)),
                        Err(e) => error!("Failed to turn off animation, reason: {}", e),
                    }
                });
                true
            }
            Msg::Discover => {
                let api = self.api.clone();
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api.discover_animations().await {
                        Ok(animations) => link.send_message(Msg::LoadedAnimations(animations)),
                        Err(e) => error!("Failed to discover animations, reason: {}", e),
                    }
                });
                true
            }
            Msg::RestartEvents => {
                let api = self.api.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(e) = api.restart_events().await {
                        error!("Failed to restart events: {}", e);
                    }
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let animations = self.animations.clone();
        html! {
            <ContextProvider<Gateway> context={self.api.clone()}>
            <>
                <header><h1>{"Rustmas Lights"}</h1></header>
                <div class="content">
                    <nav>
                        <ul>
                            <li><a onclick={link.callback(move |_| Msg::TurnOff)}>{ "⏻ Off" }</a></li>
                            <li><a onclick={link.callback(move |_| Msg::Discover)}>{ "⟳ Refresh list" }</a></li>
                            <li><a onclick={link.callback(move |_| Msg::RestartEvents)}>{ "⟳ Restart events" }</a></li>
                            <hr />
                            {
                                animations.into_iter().map(|animation| html! {
                                    <li><a onclick={link.callback(move |_| Msg::SwitchAnimation(animation.id.clone()))}>{ animation.name }</a></li>
                                }).collect::<Html>()
                            }
                        </ul>
                    </nav>
                    {if let Some(parameters) = &self.parameters {
                        html! {
                            <ParameterControlList
                                name={parameters.name.clone()}
                                schema={parameters.schema.clone()}
                                values={parameters.values.clone()}
                                update_values={link.callback(Msg::LoadedParameters)}
                                parameters_dirty={link.callback(Msg::ParametersDirty)} />
                        }
                    } else {
                        html!{
                            <div class="parameter-control-list">
                                <h2>{ "Off" }</h2>
                                <p>{ "Select an animation from the list" }</p>
                            </div>
                        }
                    }}
                </div>
            </>
            </ContextProvider<Gateway>>
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<AnimationSelector>();

    Ok(())
}
