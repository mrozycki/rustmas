mod api;
mod controls;

use std::error::Error;

use api::{Gateway, GetParamsResponse};
use yew::prelude::*;

use crate::controls::ParameterControlList;

enum Msg {
    LoadedAnimations(Vec<api::Animation>),
    SwitchAnimation(String),
    LoadedParameters(Option<GetParamsResponse>),
    ParametersDirty(bool),
    TurnOff,
    Discover,
}

#[derive(Default)]
struct AnimationSelector {
    api: api::Gateway,
    animations: Vec<api::Animation>,
    parameters: Option<GetParamsResponse>,
    dirty: bool,
}

impl Component for AnimationSelector {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let api = api::Gateway::new("/api");

        {
            let api = api.clone();
            let link = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                link.send_message(Msg::LoadedAnimations(
                    api.list_animations().await.unwrap_or_default(),
                ));
                link.send_message(Msg::LoadedParameters(api.get_params().await.ok()));
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
                    link.send_message(Msg::LoadedParameters(
                        api.switch_animation(animation_id).await.ok(),
                    ));
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
                    if api.turn_off().await.is_ok() {
                        link.send_message(Msg::LoadedParameters(None));
                    }
                });
                true
            }
            Msg::Discover => {
                let api = self.api.clone();
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    link.send_message(Msg::LoadedAnimations(
                        api.discover_animations().await.unwrap_or_default(),
                    ));
                });
                true
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
    yew::start_app::<AnimationSelector>();

    Ok(())
}
