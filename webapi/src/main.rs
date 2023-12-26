mod db;
mod frame_broadcaster;

use actix::{Actor, Addr};
use actix_cors::Cors;
use actix_web_actors::ws;
use dotenvy::dotenv;
use log::{error, info};
use rustmas_animator::Controller;
use serde::Deserialize;
use serde_json::json;
use std::{env, error::Error};
use tokio::sync::{mpsc, Mutex};

use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer};

use crate::frame_broadcaster::{FrameBroadcaster, FrameBroadcasterSession};

#[post("/events/restart")]
async fn restart_events(controller: web::Data<AnimationController>) -> HttpResponse {
    controller.0.lock().await.restart_event_generators().await;
    HttpResponse::Ok().json(())
}

#[get("/events/schema")]
async fn events_schema(controller: web::Data<AnimationController>) -> HttpResponse {
    HttpResponse::Ok().json(
        controller
            .0
            .lock()
            .await
            .event_generator_parameter_schema()
            .await,
    )
}

#[post("/events/values")]
async fn set_event_parameters(
    params: web::Json<serde_json::Value>,
    controller: web::Data<AnimationController>,
) -> HttpResponse {
    match controller
        .0
        .lock()
        .await
        .set_event_generator_parameters(params.0)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(json!(())),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[derive(Deserialize)]
struct SwitchForm {
    animation: String,
    params: Option<serde_json::Value>,
}

async fn restore_params(
    params: Option<&serde_json::Value>,
    controller: &mut Controller,
    db: &db::Db,
) -> Result<serde_json::Value, String> {
    let plugin_config = controller.current_animation().await;
    let Some(animation_id) = plugin_config.as_ref().map(|a| a.animation_id()) else {
        return Ok(json!({}));
    };

    if let Some(params) = params {
        let _ = controller.set_parameters(params.clone()).await;
    } else if let Ok(Some(params)) = db.get_parameters(animation_id).await {
        let _ = controller.set_parameters(params).await;
    } else if let Ok(params) = controller.parameter_values().await {
        let _ = db.set_parameters(animation_id, &params).await;
    }

    controller.parameters().await.map_err(|e| e.to_string())
}

#[post("/reload")]
async fn reload(controller: web::Data<AnimationController>, db: web::Data<Db>) -> HttpResponse {
    let mut controller = controller.0.lock().await;
    if let Err(e) = controller.reload_animation().await {
        return HttpResponse::InternalServerError().json(json!({ "error": format!("{:#}", e) }));
    }

    match restore_params(None, &mut controller, &db.0).await {
        Ok(animation) => HttpResponse::Ok().json(json!({ "animation": animation })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[post("/switch")]
async fn switch(
    form: web::Json<SwitchForm>,
    controller: web::Data<AnimationController>,
    db: web::Data<Db>,
) -> HttpResponse {
    let mut controller = controller.0.lock().await;
    if let Err(e) = controller.switch_animation(&form.animation).await {
        return HttpResponse::InternalServerError().json(json!({ "error": format!("{:#}", e) }));
    }

    match restore_params(form.params.as_ref(), &mut controller, &db.0).await {
        Ok(animation) => HttpResponse::Ok().json(json!({ "animation": animation })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[post("/turn_off")]
async fn turn_off(controller: web::Data<AnimationController>) -> HttpResponse {
    controller.0.lock().await.turn_off().await;
    HttpResponse::Ok().json(())
}

#[get("/params")]
async fn get_params(controller: web::Data<AnimationController>) -> HttpResponse {
    match controller.0.lock().await.parameters().await {
        Ok(animation) => HttpResponse::Ok().json(json!({ "animation": animation })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[post("/params/save")]
async fn save_params(
    controller: web::Data<AnimationController>,
    db: web::Data<Db>,
) -> HttpResponse {
    let controller = controller.0.lock().await;
    let parameter_values = match controller.parameter_values().await {
        Ok(params) => params,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"error": e.to_string() }))
        }
    };

    let animation = controller.current_animation().await;
    let Some(animation_id) = animation.as_ref().map(|a| a.animation_id()) else {
        return HttpResponse::Ok().json(json!({}));
    };

    match db.0.set_parameters(animation_id, &parameter_values).await {
        Ok(_) => HttpResponse::Ok().json(json!(())),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[post("/params/reset")]
async fn reset_params(
    controller: web::Data<AnimationController>,
    db: web::Data<Db>,
) -> HttpResponse {
    let mut controller = controller.0.lock().await;
    let animation = controller.current_animation().await;
    let Some(animation_id) = animation.as_ref().map(|a| a.animation_name()) else {
        return HttpResponse::Ok().json(json!({ "animation": () }));
    };

    match db.0.get_parameters(animation_id).await {
        Ok(Some(params)) => {
            let _ = controller.set_parameters(params).await;
            match controller.parameters().await {
                Ok(animation) => HttpResponse::Ok().json(json!({ "animation": animation })),
                Err(e) => {
                    HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }))
                }
            }
        }
        Ok(None) => HttpResponse::InternalServerError()
            .json(json!({"error": "No parameters stored for this animation"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[post("/params")]
async fn post_params(
    params: web::Json<serde_json::Value>,
    controller: web::Data<AnimationController>,
) -> HttpResponse {
    match controller.0.lock().await.set_parameters(params.0).await {
        Ok(_) => HttpResponse::Ok().json(json!(())),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[post("/discover")]
async fn discover(controller: web::Data<AnimationController>) -> HttpResponse {
    let mut controller = controller.0.lock().await;
    match controller.discover_animations() {
        Ok(_) => HttpResponse::Ok().json(json!({
            "animations": controller
                .list_animations()
                .iter()
                .map(|(id, plugin)| json!({"id": id, "name": plugin.animation_name()}))
                .collect::<Vec<_>>()})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/list")]
async fn list(controller: web::Data<AnimationController>) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "animations":
            controller.0
            .lock()
            .await
            .list_animations()
            .iter()
            .map(|(id, plugin)| json!({"id": id, "name": plugin.animation_name()}))
            .collect::<Vec<_>>()}))
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

#[get("/points")]
async fn points(controller: web::Data<AnimationController>) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "points": controller.0.lock().await.points()
    }))
}

struct AnimationController(Mutex<rustmas_animator::Controller>);
struct Db(db::Db);

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
            if url.starts_with("http://") {
                builder = builder.http_lights(&url)?;
            } else if url.starts_with("tcp://") {
                builder = builder.tcp_lights(&url)?;
            } else if url.starts_with("udp://") {
                builder = builder.udp_lights(&url)?;
            } else {
                error!("Unknown remote client protocol, ignoring");
            }
        }
        if env::var("RUSTMAS_USE_TTY")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false)
        {
            builder = builder.local_lights()?;
        }

        let mut controller = builder.build();
        controller.discover_animations()?;
        info!("Discovered {} plugins", controller.list_animations().len());
        web::Data::new(AnimationController(Mutex::new(controller)))
    };

    info!("Establishing database connection");
    let db = web::Data::new(Db(db::Db::new(
        &env::var("RUSTMAS_DB_PATH").unwrap_or("db.sqlite".to_owned()),
    )
    .await?));

    let frame_broadcaster = web::Data::new(FrameBroadcaster::new(receiver).start());

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(restart_events)
            .service(events_schema)
            .service(set_event_parameters)
            .service(reload)
            .service(switch)
            .service(turn_off)
            .service(list)
            .service(discover)
            .service(get_params)
            .service(post_params)
            .service(save_params)
            .service(reset_params)
            .service(frames)
            .service(points)
            .app_data(controller.clone())
            .app_data(db.clone())
            .app_data(frame_broadcaster.clone())
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await?;

    Ok(())
}
