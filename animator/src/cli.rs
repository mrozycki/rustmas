use std::{error::Error, fs};

use clap::Parser;
use log::LevelFilter;
use simplelog::{
    ColorChoice, CombinedLogger, Config, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};

use tracing::*;
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{prelude::*, registry::Registry};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    lights_endpoint: Option<String>,
    #[arg(short, long, default_value = "lights.csv")]
    positions_file: String,
    #[arg(short, long, default_value = "rainbow_waterfall")]
    animation: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // CombinedLogger::init(vec![
    //     TermLogger::new(
    //         LevelFilter::Info,
    //         Config::default(),
    //         TerminalMode::Mixed,
    //         ColorChoice::Auto,
    //     ),
    //     WriteLogger::new(
    //         #[cfg(debug_assertions)]
    //         LevelFilter::Debug,
    //         #[cfg(not(debug_assertions))]
    //         LevelFilter::Info,
    //         ConfigBuilder::new().set_time_format_rfc3339().build(),
    //         fs::OpenOptions::new()
    //             .append(true)
    //             .create(true)
    //             .open("animator.log")?,
    //     ),
    // ])?;
    let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
    tracing_subscriber::registry().with(chrome_layer).init();

    let cli = Cli::parse();

    let mut builder =
        rustmas_animator::Controller::builder().points_from_file(&cli.positions_file)?;

    builder = match cli.lights_endpoint {
        Some(path) => builder.remote_lights(&path)?,
        None => {
            #[cfg(not(feature = "visualiser"))]
            panic!("Visualiser feature is disabled, please provide a light client endpoint");

            #[cfg(feature = "visualiser")]
            builder.visualiser_lights()?
        }
    };

    let controller = builder.build()?;
    controller.switch_animation(&cli.animation).await?;
    controller.join().await?;

    Ok(())
}
