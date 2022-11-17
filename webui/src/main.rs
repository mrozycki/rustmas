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
}

#[derive(Default)]
struct AnimationSelector {
    api: api::Gateway,
    animations: Vec<api::Animation>,
    parameters: Option<GetParamsResponse>,
}

impl Component for AnimationSelector {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let api = api::Gateway::new("http://localhost:8081");

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
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let animations = self.animations.clone();
        html! {
            <ContextProvider<Gateway> context={self.api.clone()}>
            <div>
                <ul style="list-style-type: none;"> {
                    animations.into_iter().map(|animation| html! {
                        <li><button onclick={link.callback(move |_| Msg::SwitchAnimation(animation.id.clone()))}>{ animation.name }</button></li>
                    }).collect::<Html>()
                } </ul>
                <hr />
                {if let Some(parameters) = &self.parameters {
                    html! {
                        <ParameterControlList schema={parameters.schema.clone()} values={parameters.values.clone()} />
                    }
                } else { html!{} }}
            </div>
            </ContextProvider<Gateway>>
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    yew::start_app::<AnimationSelector>();

    Ok(())
}
