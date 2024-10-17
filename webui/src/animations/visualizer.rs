use log::error;
use rustmas_webapi_client::RustmasApiClient;
use web_sys::MouseEvent;
use yew::{html, Callback, Html};

#[yew::function_component(Visualizer)]
pub fn visualizer() -> Html {
    let api = yew::use_context::<RustmasApiClient>().expect("gateway to be created");
    let loaded = yew::use_mut_ref(|| false);

    let points_loaded = Callback::from({
        let api = api.clone();
        let loaded = loaded.clone();
        move |points| {
            let api = api.clone();
            wasm_bindgen_futures::spawn_local(async move {
                rustmas_visualizer::run(api, points);
            });
            *loaded.borrow_mut() = true;
        }
    });

    yew::use_effect({
        let api = api.clone();
        move || {
            if !(*loaded.borrow()) {
                wasm_bindgen_futures::spawn_local(async move {
                    match api.get_points().await {
                        Ok(points) => points_loaded.emit(points),
                        Err(e) => error!("Failed to load points for visualizer, reason: {}", e),
                    }
                })
            }

            // Do nothing on cleanup
            || {}
        }
    });

    html! {
        <section class="visualizer-container">
            <canvas id="visualizer"
                oncontextmenu={Callback::from(|e: MouseEvent| e.prevent_default())}
                onmousedown={Callback::from(|e: MouseEvent| {
                    if e.button() == 1 {
                        e.prevent_default();
                    }
                })}
            ></canvas>
        </section>
    }
}
