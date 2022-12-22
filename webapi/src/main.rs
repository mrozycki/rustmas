mod db;

use actix_cors::Cors;
use db::Db;
use dotenvy::dotenv;
use log::{info, LevelFilter};
use serde::Deserialize;
use serde_json::json;
use simplelog::{
    ColorChoice, CombinedLogger, Config, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};
use std::{env, error::Error, fs, sync::Mutex};

use actix_web::{get, post, web, App, HttpResponse, HttpServer};

#[derive(Deserialize)]
struct SwitchForm {
    animation: String,
}

#[post("/switch")]
async fn switch(form: web::Json<SwitchForm>, app_state: web::Data<AppState>) -> HttpResponse {
    let mut controller = app_state.animation_controller.lock().unwrap();
    if let Err(e) = controller.switch_animation(&form.animation).await {
        return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
    }

    if let Ok(Some(params)) = app_state.db.get_parameters(&form.animation).await {
        let _ = controller.set_parameters(params).await;
    } else {
        let _ = app_state
            .db
            .set_parameters(&form.animation, &controller.parameter_values().await)
            .await;
    }

    *app_state.animation_name.lock().unwrap() = form.animation.clone();
    HttpResponse::Ok().json(controller.parameters().await)
}

#[get("/params")]
async fn get_params(app_state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(
        app_state
            .animation_controller
            .lock()
            .unwrap()
            .parameters()
            .await,
    )
}

#[post("/params/save")]
async fn save_params(app_state: web::Data<AppState>) -> HttpResponse {
    match app_state
        .db
        .set_parameters(
            &app_state.animation_name.lock().unwrap(),
            &app_state
                .animation_controller
                .lock()
                .unwrap()
                .parameter_values()
                .await,
        )
        .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"success": false})),
    }
}

#[post("/params/reset")]
async fn reset_params(app_state: web::Data<AppState>) -> HttpResponse {
    if let Ok(Some(params)) = app_state
        .db
        .get_parameters(&app_state.animation_name.lock().unwrap())
        .await
    {
        let mut controller = app_state.animation_controller.lock().unwrap();
        let _ = controller.set_parameters(params).await;
        HttpResponse::Ok().json(controller.parameters().await)
    } else {
        HttpResponse::InternalServerError().json(json!({"success": false}))
    }
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
            { "id": "blank", "name": "Off" },
            { "id": "rainbow_waterfall", "name": "Rainbow Waterfall" },
            { "id": "rainbow_cylinder", "name": "Rainbow Cylinder" },
            { "id": "rainbow_halves", "name": "Rainbow Halves" },
            { "id": "rainbow_sphere", "name": "Rainbow Sphere" },
            { "id": "rainbow_spiral", "name": "Rainbow Spiral" },
            { "id": "rainbow_cable", "name": "Rainbow Cable" },
            { "id": "barber_pole", "name": "Barber Pole" },
            { "id": "random_sweep", "name": "Random Sweep" },
            { "id": "spinning_halves", "name": "Spinning Halves" },
            { "id": "present", "name": "Present" },
            { "id": "test_check", "name": "Testing: Check lights" },
            { "id": "test_sweep", "name": "Testing: Sweep" },
            { "id": "test_manual_sweep", "name": "Testing: Manual Sweep" },
            { "id": "test_indexing", "name": "Testing: Indexing" },
            { "id": "test_detection_status", "name": "Testing: Detection status" },
        ]
    }))
}

struct AppState {
    animation_controller: Mutex<rustmas_animator::Controller>,
    animation_name: Mutex<String>,
    db: Db,
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
            ConfigBuilder::new().set_time_format_rfc3339().build(),
            fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open("webapi.log")?,
        ),
    ])?;

    dotenv().ok();

    info!("Starting controller");
    #[cfg(not(feature = "visualiser"))]
    let controller = rustmas_animator::Controller::builder()
        .points_from_file(&env::var("RUSTMAS_POINTS_PATH").unwrap_or("lights.csv".to_owned()))?
        .remote_lights(&env::var("RUSTMAS_LIGHTS_URL").unwrap_or("http://127.0.0.1/".to_owned()))?
        .build()?;

    #[cfg(feature = "visualiser")]
    let controller = {
        let mut builder = rustmas_animator::Controller::builder().points_from_file(
            &env::var("RUSTMAS_POINTS_PATH").unwrap_or("lights.csv".to_owned()),
        )?;
        builder = if let Ok(url) = env::var("RUSTMAS_LIGHTS_URL") {
            builder.remote_lights(&url)?
        } else {
            builder.visualiser_lights()?
        };

        builder.build()?
    };

    info!("Establishing database connection");
    let db = Db::new(&env::var("RUSTMAS_DB_PATH").unwrap_or("db.sqlite".to_owned())).await?;

    info!("Starting http server");
    let app_state = web::Data::new(AppState {
        animation_controller: Mutex::new(controller),
        animation_name: Mutex::new("blank".to_owned()),
        db,
    });

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(switch)
            .service(list)
            .service(get_params)
            .service(post_params)
            .service(save_params)
            .service(reset_params)
            .app_data(app_state.clone())
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await?;

    Ok(())
}
