use actix_cors::Cors;
use dotenvy::dotenv;
use log::LevelFilter;
use serde::Deserialize;
use serde_json::json;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::{env, error::Error, fs::File, sync::Mutex};

use actix_web::{get, post, web, App, HttpResponse, HttpServer};

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
        .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"success": false})),
    }
}

#[get("/params")]
async fn get_params(app_state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "schema": app_state
            .animation_controller
            .lock()
            .unwrap()
            .parameter_schema()
            .await,
        "values": app_state.animation_controller.lock().unwrap().parameter_values().await,
    }))
}

#[post("/params")]
async fn post_params(
    params: web::Json<serde_json::Value>,
    app_state: web::Data<AppState>,
) -> HttpResponse {
    match app_state
        .animation_controller
        .lock()
        .unwrap()
        .set_parameters(params.0)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"success": false})),
    }
}

#[get("/list")]
async fn list() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "animations": [
            { "id": "rainbow_waterfall", "name": "Rainbow Waterfall" },
            { "id": "rainbow_cylinder", "name": "Rainbow Cylinder" },
            { "id": "rainbow_sphere", "name": "Rainbow Sphere" },
            { "id": "rainbow_spiral", "name": "Rainbow Spiral" },
            { "id": "rainbow_cable", "name": "Rainbow Cable" },
            { "id": "barber_pole", "name": "Barber Pole" },
            { "id": "sweep", "name": "Test Sweep" },
            { "id": "random_sweep", "name": "Random Sweep" },
            { "id": "blank", "name": "Blank" },
            { "id": "check", "name": "Check" },
        ]
    }))
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
            #[cfg(debug_assertions)]
            LevelFilter::Debug,
            #[cfg(not(debug_assertions))]
            LevelFilter::Info,
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
            .service(list)
            .service(get_params)
            .service(post_params)
            .app_data(app_state.clone())
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await?;

    Ok(())
}
