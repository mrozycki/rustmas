use actix_files::NamedFile;
use dotenvy::dotenv;
use log::LevelFilter;
use serde::Deserialize;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::{env, error::Error, fs::File, sync::Mutex};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[derive(Deserialize)]
struct SwitchForm {
    animation_name: String,
}

#[post("/switch")]
async fn switch(form: web::Form<SwitchForm>, app_state: web::Data<AppState>) -> impl Responder {
    match app_state
        .animation_controller
        .lock()
        .unwrap()
        .switch_animation(&form.animation_name)
    {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError(),
    }
}

#[get("/")]
async fn index() -> std::io::Result<NamedFile> {
    Ok(NamedFile::open("webapi/index.html")?)
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
        App::new()
            .service(switch)
            .service(index)
            .app_data(app_state.clone())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}
