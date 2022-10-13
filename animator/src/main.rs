use std::{error::Error, time::Duration};

use client::LightClient;
use rustmas_light_client as client;

fn generate_checkers(index: usize, size: usize) -> client::Frame {
    (0..size)
        .into_iter()
        .map(|x| match (x + index) % 3 {
            0 => client::Color::rgb(255, 0, 0),
            1 => client::Color::rgb(0, 255, 0),
            _ => client::Color::rgb(0, 0, 255),
        })
        .collect::<Vec<_>>()
        .into()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = client::VisualiserLightClient::new("lights.csv")?;
    loop {
        client.display_frame(&generate_checkers(0, 500)).await?;
        std::thread::sleep(Duration::from_secs(1));
        client.display_frame(&generate_checkers(1, 500)).await?;
        std::thread::sleep(Duration::from_secs(1));
    }
}
