use std::time::Duration;

use client::{LightClient, LightClientError};
use rustmas_light_client as client;

fn generate_checkers(index: usize, size: usize) -> client::Frame {
    (0..size)
        .into_iter()
        .map(|x| (x + index) % 2)
        .map(|x| client::Color::gray((x * 255).try_into().unwrap()))
        .collect::<Vec<_>>()
        .into()
}

#[tokio::main]
async fn main() -> Result<(), LightClientError> {
    let client = client::RemoteLightClient::new("http://192.168.0.204/pixels");
    loop {
        client.display_frame(&generate_checkers(0, 500)).await?;
        std::thread::sleep(Duration::from_secs(1));
        client.display_frame(&generate_checkers(1, 500)).await?;
        std::thread::sleep(Duration::from_secs(1));
    }
}
