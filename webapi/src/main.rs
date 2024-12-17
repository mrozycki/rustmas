mod animations;
mod config;
mod events;
mod parameters;
mod visualizer;

use ::config::Config;
use actix_cors::Cors;
use config::RustmasConfig;
use log::info;
use std::error::Error;
use tokio::sync::{mpsc, Mutex};

use actix_web::{web, App, HttpServer};

type AnimationController = Mutex<rustmas_animator::Controller>;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let config = Config::builder()
        .add_source(::config::File::with_name("Rustmas"))
        .add_source(::config::Environment::with_prefix("RUSTMAS"))
        .build()?
        .try_deserialize::<RustmasConfig>()?;

    info!("Setting up database");
    let parameters = web::Data::new(parameters::Logic::from(&config).await?);
    let animations = web::Data::new(animations::Logic::new());

    let (sender, receiver) = mpsc::channel::<lightfx::Frame>(1);

    info!("Starting controller");
    let controller = {
        let mut controller = rustmas_animator::Controller::builder_from(&config.controller)
            .expect("Could not start animations controller")
            .lights_feedback(sender)
            .build();

        let _ = controller.discover_animations();
        info!("Discovered {} plugins", controller.list_animations().len());
        web::Data::new(Mutex::new(controller))
    };

    let visualizer_service = visualizer::service_factory(receiver);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .service(events::service())
            .service(animations::service())
            .service(parameters::service())
            .service(visualizer_service())
            .app_data(controller.clone())
            .app_data(parameters.clone())
            .app_data(animations.clone())
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await?;

    Ok(())
}
