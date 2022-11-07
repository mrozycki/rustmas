use std::error::Error;

use gloo_net::http::Request;
use serde_json::json;
use yew::prelude::*;

enum Msg {
    SwitchAnimation(String),
}

#[derive(Clone)]
struct Animation {
    id: String,
    name: String,
}

impl Animation {
    fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
        }
    }
}

struct AnimationSelector {
    animations: Vec<Animation>,
}

impl Component for AnimationSelector {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            animations: vec![
                Animation::new("rainbow_waterfall", "Rainbow Waterfall"),
                Animation::new("rainbow_cylinder", "Rainbow Cylinder"),
                Animation::new("rainbow_sphere", "Rainbow Sphere"),
                Animation::new("rainbow_spiral", "Rainbow Spiral"),
                Animation::new("rainbow_cable", "Rainbow Cable"),
                Animation::new("barber_pole", "Barber Pole"),
                Animation::new("random_sweep", "Random Sweep"),
                Animation::new("blank", "Blank"),
            ],
        }
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
