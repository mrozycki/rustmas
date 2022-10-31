use std::{error::Error, fs::File};

use clap::Parser;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

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
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("animator.log")?,
        ),
    ])?;

    let cli = Cli::parse();

    let mut builder =
        rustmas_animator::Controller::builder().points_from_file(&cli.positions_file)?;

    builder = match cli.lights_endpoint {
        Some(path) => builder.remote_lights(&path)?,
        None => builder.visualiser_lights()?,
    };

    let controller = builder.build()?;
    controller.switch_animation(&cli.animation)?;
    controller.join().await?;

    Ok(())
}
