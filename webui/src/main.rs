use std::error::Error;

use gloo_net::http::Request;
use serde::Deserialize;
use serde_json::json;
use yew::prelude::*;

enum Msg {
    LoadedAnimations(Vec<Animation>),
    SwitchAnimation(String),
}

#[derive(Clone, Deserialize)]
struct Animation {
    id: String,
    name: String,
}

#[derive(Default, Deserialize)]
struct AnimationSelector {
    animations: Vec<Animation>,
}

impl Component for AnimationSelector {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let response = Request::get("http://localhost:8081/list").send().await;
            let animations = match response {
                Ok(response) => {
                    response
                        .json::<AnimationSelector>()
                        .await
                        .unwrap_or_default()
                        .animations
                }
                Err(_) => vec![],
            };
            link.send_message(Msg::LoadedAnimations(animations));
        });

        Default::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SwitchAnimation(name) => {
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = Request::post("http://localhost:8081/switch")
                        .header("Content-Type", "application/json")
                        .json(&json!({ "animation": name }))
                        .expect("Could not build that request.")
                        .send()
                        .await;
                });
                false
            }
            Msg::LoadedAnimations(animations) => {
                self.animations = animations;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let animations = self.animations.clone();
        html! {
            <div>
                <ul style="list-style-type: none;"> {
                    animations.into_iter().map(|animation| html! {
                        <li><button onclick={link.callback(move |_| Msg::SwitchAnimation(animation.id.clone()))}>{ animation.name }</button></li>
                    }).collect::<Html>()
                } </ul>
            </div>
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    yew::start_app::<AnimationSelector>();

    Ok(())
}
