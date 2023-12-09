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

fn get_points(endpoint: &Url) -> Vec<(f32, f32, f32)> {
    let endpoint = {
        let mut endpoint = endpoint.clone();
        endpoint.set_scheme("http").unwrap();
        endpoint.join("points").unwrap()
    };

    reqwest::blocking::get(endpoint)
        .unwrap()
        .json::<GetPointsResponse>()
        .unwrap()
        .points
}

fn main() {
    let args = Args::parse();

    let frames_endpoint = get_frames_url(&args.endpoint);
    let points = get_points(&args.endpoint);

    rustmas_visualizer::run(frames_endpoint, points);
}
