mod animations;
mod config;
mod db;
mod events;
mod parameters;
mod visualizer;

use ::config::Config;
use actix_cors::Cors;
use config::RustmasConfig;
use db::SharedDbConnection;
use itertools::Itertools;
use log::info;
use rustmas_animator::points_from_path;
use std::error::Error;
use tokio::sync::{Mutex, mpsc};

use actix_web::{App, HttpServer, web};

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
    let shared_db = SharedDbConnection::from_config(&config).await?;
    let parameters = web::Data::new(parameters::Logic::from(shared_db.clone()));
    let animations = web::Data::new(animations::Logic::from(shared_db, &config)?);

    let (sender, receiver) = mpsc::channel::<lightfx::Frame>(1);

    info!("Starting controller");
    // Workaround for points not being managed by controller anymore,
    // until we have a separate config service, which will take this over
    let points = web::Data::new(
        points_from_path(&config.controller.points_path)?
            .into_iter()
            .map(|(x, y, z)| (x as f32, y as f32, z as f32))
            .collect_vec(),
    );

    let controller = {
        let controller = rustmas_animator::Controller::from_config(
            &config.controller,
            Some(sender),
            points.len(),
        )
        .expect("Could not start animations controller");

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
            .app_data(points.clone())
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await?;

    Ok(())
}
