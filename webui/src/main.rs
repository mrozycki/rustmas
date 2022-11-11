mod api;

use std::error::Error;

use gloo_utils::document;
use serde_json::json;
use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

enum Msg {
    LoadedAnimations(Vec<api::Animation>),
    SwitchAnimation(String),
    LoadedParameterSchema(String),
    SendParams,
}

#[derive(Default)]
struct AnimationSelector {
    api: api::Gateway,
    animations: Vec<api::Animation>,
    parameter_schema: String,
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
                link.send_message(Msg::LoadedParameterSchema(
                    api.get_params().await.unwrap_or(json!({})).to_string(),
                ));
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
                    link.send_message(Msg::LoadedParameterSchema(
                        api.get_params().await.unwrap_or(json!({})).to_string(),
                    ));
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
                let api = self.api.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = api
                        .set_params(
                            &document()
                                .get_element_by_id("params")
                                .unwrap()
                                .dyn_into::<HtmlTextAreaElement>()
                                .unwrap()
                                .value(),
                        )
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
