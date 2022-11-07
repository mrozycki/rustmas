use actix_cors::Cors;
use dotenvy::dotenv;
use log::LevelFilter;
use serde::Deserialize;
use serde_json::json;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::{env, error::Error, fs::File, sync::Mutex};

use actix_web::{post, web, App, HttpResponse, HttpServer};

#[derive(Deserialize)]
struct SwitchForm {
    animation: String,
}

#[post("/switch")]
async fn switch(form: web::Json<SwitchForm>, app_state: web::Data<AppState>) -> HttpResponse {
    match app_state
        .animation_controller
        .lock()
        .unwrap()
        .switch_animation(&form.animation)
    {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"success": false})),
    }
}

struct AppState {
    animation_controller: Mutex<rustmas_animator::Controller>,
}

#[actix_web::main]
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
            File::create("webapi.log")?,
        ),
    ])?;

    dotenv().ok();

    let controller = rustmas_animator::Controller::builder()
        .points_from_file(&env::var("RUSTMAS_POINTS_PATH").unwrap_or("lights.csv".to_owned()))?
        .remote_lights(&env::var("RUSTMAS_LIGHTS_URL").unwrap_or("http://127.0.0.1/".to_owned()))?
        .build()?;

    let app_state = web::Data::new(AppState {
        animation_controller: Mutex::new(controller),
    });

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(switch)
            .app_data(app_state.clone())
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await?;

    Ok(())
}
