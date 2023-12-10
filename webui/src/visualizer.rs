use log::error;
use yew::{html, prelude::Html, Callback, Component, Context};

use crate::api;

#[derive(Default)]
pub struct Visualizer {}

pub enum Msg {
    PointsLoaded(Vec<(f32, f32, f32)>),
}

fn get_api(ctx: &Context<Visualizer>) -> api::Gateway {
    ctx.link()
        .context::<api::Gateway>(Callback::noop())
        .expect("gateway to be created")
        .0
}

impl Component for Visualizer {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PointsLoaded(points) => {
                let api = get_api(ctx);
                wasm_bindgen_futures::spawn_local(async move {
                    rustmas_visualizer::run(api.frames(), points);
                });
                false
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let api = get_api(ctx);
            let points_loaded = ctx.link().callback(Msg::PointsLoaded);
            wasm_bindgen_futures::spawn_local(async move {
                match api.get_points().await {
                    Ok(points) => points_loaded.emit(points),
                    Err(e) => error!("Failed to load points for visualizer, reason: {}", e),
                }
            })
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <section class="visualizer-container">
                <canvas id="visualizer"></canvas>
            </section>
        }
    }
}
