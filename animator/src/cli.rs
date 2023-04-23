use std::error::Error;

use clap::Parser;

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
    env_logger::init();

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

    let controller = builder.build();
    controller.switch_animation(&cli.animation).await?;
    controller.join().await?;

    Ok(())
}
