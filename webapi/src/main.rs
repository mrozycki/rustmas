mod db;
mod frame_broadcaster;

use actix::{Actor, Addr};
use actix_cors::Cors;
use actix_web_actors::ws;
use db::Db;
use dotenvy::dotenv;
use log::info;
use serde::Deserialize;
use serde_json::json;
use std::{env, error::Error};
use tokio::sync::{mpsc, Mutex};

use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer};

use crate::frame_broadcaster::{FrameBroadcaster, FrameBroadcasterSession};

#[derive(Deserialize)]
struct SwitchForm {
    animation: String,
}

async fn switch_inner(animation_name: &str, app_state: web::Data<AppState>) -> HttpResponse {
    let mut controller = app_state.animation_controller.lock().await;
    if let Err(e) = controller.switch_animation(animation_name).await {
        return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
    }

    if let Ok(Some(params)) = app_state.db.get_parameters(animation_name).await {
        let _ = controller.set_parameters(params).await;
    } else {
        let _ = app_state
            .db
            .set_parameters(animation_name, &controller.parameter_values().await)
            .await;
    }

    *app_state.animation_name.lock().await = animation_name.to_owned();
    HttpResponse::Ok().json(controller.parameters().await)
}

#[post("/reload")]
async fn reload(app_state: web::Data<AppState>) -> HttpResponse {
    let name = app_state.animation_name.lock().await.clone();
    switch_inner(&name, app_state).await
}

#[post("/switch")]
async fn switch(form: web::Json<SwitchForm>, app_state: web::Data<AppState>) -> HttpResponse {
    switch_inner(&form.animation, app_state).await
}

#[get("/params")]
async fn get_params(app_state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(
        app_state
            .animation_controller
            .lock()
            .await
            .parameters()
            .await,
    )
}

#[post("/params/save")]
async fn save_params(app_state: web::Data<AppState>) -> HttpResponse {
    match app_state
        .db
        .set_parameters(
            &app_state.animation_name.lock().await,
            &app_state
                .animation_controller
                .lock()
                .await
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
        .get_parameters(&app_state.animation_name.lock().await)
        .await
    {
        let mut controller = app_state.animation_controller.lock().await;
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
        .await
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
            { "id": "doom_fire", "name": "Doom Fire" },
            { "id": "particle_fire", "name": "Particle Fire" },
            { "id": "circle_boom", "name": "Circle boom" },
            { "id": "classic", "name": "Classic" },
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
            { "id": "snow", "name": "Snow" },
            { "id": "stars", "name": "Stars" },
            { "id": "test_check", "name": "Testing: Check lights" },
            { "id": "test_sweep", "name": "Testing: Sweep" },
            { "id": "test_manual_sweep", "name": "Testing: Manual Sweep" },
            { "id": "test_indexing", "name": "Testing: Indexing" },
            { "id": "test_detection_status", "name": "Testing: Detection status" },
        ]
    }))
}

#[get("/frames")]
async fn frames(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<FrameBroadcaster>>,
) -> Result<HttpResponse, actix_web::Error> {
    ws::start(
        FrameBroadcasterSession::new(server.get_ref().clone()),
        &req,
        stream,
    )
}

struct AppState {
    animation_controller: Mutex<rustmas_animator::Controller>,
    animation_name: Mutex<String>,
    db: Db,
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    let (sender, receiver) = mpsc::channel::<lightfx::Frame>(1);

    info!("Starting controller");
    let controller = {
        let mut builder = rustmas_animator::Controller::builder()
            .points_from_file(&env::var("RUSTMAS_POINTS_PATH").unwrap_or("lights.csv".to_owned()))?
            .lights_feedback(sender)
            .plugin_dir(env::var("RUSTMAS_PLUGIN_DIR").unwrap_or(".".to_owned()));

        if let Ok(url) = env::var("RUSTMAS_LIGHTS_URL") {
            builder = builder.remote_lights(&url)?;
        }
        if let Ok(path) = env::var("RUSTMAS_TTY_PATH") {
            builder = builder.local_lights(&path)?;
        }

        #[cfg(feature = "visualiser")]
        {
            builder = builder.visualiser_lights()?;
        }

        builder.build()
    };

    info!("Establishing database connection");
    let db = Db::new(&env::var("RUSTMAS_DB_PATH").unwrap_or("db.sqlite".to_owned())).await?;

    info!("Starting http server");
    let app_state = web::Data::new(AppState {
        animation_controller: Mutex::new(controller),
        animation_name: Mutex::new("blank".to_owned()),
        db,
    });

    let frame_broadcaster = web::Data::new(FrameBroadcaster::new(receiver).start());

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(reload)
            .service(switch)
            .service(list)
            .service(get_params)
            .service(post_params)
            .service(save_params)
            .service(reset_params)
            .service(frames)
            .app_data(app_state.clone())
            .app_data(frame_broadcaster.clone())
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await?;

    Ok(())
}
