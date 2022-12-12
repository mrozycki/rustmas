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
    ParametersChanged(()),
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
            Msg::SwitchAnimation(name) => {
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
                    let _ = api.switch_animation(name).await;
                    link.send_message(Msg::LoadedParameters(api.get_params().await.ok()));
                });
                false
            }
            Msg::LoadedAnimations(animations) => {
                self.animations = animations;
                true
            }
            Msg::LoadedParameters(parameters) => {
                self.parameters = parameters;
                self.dirty = false;
                true
            }
            Msg::ParametersChanged(_) => {
                self.dirty = true;
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
                        <ul> {
                            animations.into_iter().map(|animation| html! {
                                <li><a onclick={link.callback(move |_| Msg::SwitchAnimation(animation.id.clone()))}>{ animation.name }</a></li>
                            }).collect::<Html>()
                        } </ul>
                    </nav>
                    {if let Some(parameters) = &self.parameters {
                        html! {
                            <ParameterControlList schema={parameters.schema.clone()} values={parameters.values.clone()}
                                update_values={link.callback(Msg::LoadedParameters)}
                                dirty_values={link.callback(Msg::ParametersChanged)} />
                        }
                    } else { html!{} }}
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
