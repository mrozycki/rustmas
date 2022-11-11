use std::error::Error;

use gloo_net::http::Request;
use gloo_utils::document;
use serde::Deserialize;
use serde_json::json;
use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

enum Msg {
    LoadedAnimations(Vec<Animation>),
    SwitchAnimation(String),
    LoadedParameterSchema(String),
    SendParams,
}

#[derive(Clone, Deserialize)]
struct Animation {
    id: String,
    name: String,
}

#[derive(Default, Deserialize)]
struct LoadedAnimations {
    animations: Vec<Animation>,
}

#[derive(Default)]
struct AnimationSelector {
    animations: Vec<Animation>,
    parameter_schema: String,
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
                        .json::<LoadedAnimations>()
                        .await
                        .unwrap_or_default()
                        .animations
                }
                Err(_) => vec![],
            };
            link.send_message(Msg::LoadedAnimations(animations));
            let params = Request::get("http://localhost:8081/params")
                .send()
                .await
                .unwrap()
                .json::<serde_json::Value>()
                .await
                .unwrap()
                .to_string();
            link.send_message(Msg::LoadedParameterSchema(params));
        });

        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SwitchAnimation(name) => {
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = Request::post("http://localhost:8081/switch")
                        .header("Content-Type", "application/json")
                        .json(&json!({ "animation": name }))
                        .expect("Could not build that request.")
                        .send()
                        .await;
                    let params = Request::get("http://localhost:8081/params")
                        .send()
                        .await
                        .unwrap()
                        .json::<serde_json::Value>()
                        .await
                        .unwrap()
                        .to_string();
                    link.send_message(Msg::LoadedParameterSchema(params));
                });
                false
            }
            Msg::LoadedAnimations(animations) => {
                self.animations = animations;
                true
            }
            Msg::LoadedParameterSchema(parameter_schema) => {
                self.parameter_schema = parameter_schema;
                true
            }
            Msg::SendParams => {
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = Request::post("http://localhost:8081/params")
                        .header("Content-Type", "application/json")
                        .body(
                            document()
                                .get_element_by_id("params")
                                .unwrap()
                                .dyn_into::<HtmlTextAreaElement>()
                                .unwrap()
                                .value(),
                        )
                        .send()
                        .await;
                });
                false
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
                <hr />
                <div>
                    <pre>{ &self.parameter_schema }</pre>
                    <textarea cols="80" rows="24" id="params" />
                    <input type="submit" value="Send" onclick={link.callback(move |_| Msg::SendParams)} />
                </div>
            </div>
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    yew::start_app::<AnimationSelector>();

    Ok(())
}
