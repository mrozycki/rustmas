use clap::Parser;
use serde::Deserialize;
use url::Url;

/// Visualizer for Rustmas animations
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Address of the WebAPI endpoint
    #[arg(short, long, default_value = "http://127.0.0.1:8081")]
    endpoint: Url,
}

fn get_frames_url(endpoint: &Url) -> Url {
    let mut endpoint = endpoint.clone();
    endpoint.set_scheme("ws").unwrap();
    endpoint.join("frames").unwrap()
}

#[derive(Deserialize)]
struct GetPointsResponse {
    points: Vec<(f32, f32, f32)>,
}

fn get_points_endpoint(endpoint: &Url) -> Url {
    let mut endpoint = endpoint.clone();
    endpoint.set_scheme("http").unwrap();
    endpoint.join("points").unwrap()
}

fn main() {
    let endpoint = Args::parse().endpoint;
    let frames_endpoint = get_frames_url(&endpoint);
    let points_endpoint = get_points_endpoint(&endpoint);

    #[cfg(not(target_arch = "wasm32"))]
    {
        let points = reqwest::blocking::get(points_endpoint)
            .unwrap()
            .json::<GetPointsResponse>()
            .unwrap()
            .points;

        rustmas_visualizer::run(frames_endpoint, points);
    }

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let points = gloo_net::http::Request::get(points_endpoint.as_str())
            .send()
            .await
            .unwrap()
            .json::<GetPointsResponse>()
            .await
            .unwrap()
            .points;
        rustmas_visualizer::run(frames_endpoint, points);
    });
}
