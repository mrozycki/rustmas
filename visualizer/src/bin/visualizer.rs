use clap::Parser;
use rustmas_webapi_client::RustmasApiClient;
use url::Url;

/// Visualizer for Rustmas animations
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Address of the WebAPI endpoint
    #[arg(short, long, default_value = "http://127.0.0.1:8081")]
    endpoint: Url,
}

fn main() {
    let endpoint = Args::parse().endpoint;
    let api = RustmasApiClient::new(endpoint);
    let frames_endpoint = api.frames();
    let points = {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.handle().block_on(async move { api.get_points().await })
    }
    .unwrap();

    rustmas_visualizer::run(frames_endpoint, points);
}
