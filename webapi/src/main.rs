mod db;
mod frame_broadcaster;

use actix::{Actor, Addr};
use actix_cors::Cors;
use actix_web_actors::ws;
use dotenvy::dotenv;
use log::{error, info, warn};
use rustmas_animator::Controller;
use serde_json::json;
use std::{env, error::Error};
use tokio::sync::{mpsc, Mutex};
use webapi_model::{
    Animation, Configuration, GetEventGeneratorSchemaResponse, GetParametersResponse,
    GetPointsResponse, ListAnimationsResponse, SetAnimationParametersRequest,
    SetEventGeneratorParametersRequest, SwitchAnimationRequest, SwitchAnimationResponse,
};

use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer};

use crate::frame_broadcaster::{FrameBroadcaster, FrameBroadcasterSession};

#[post("/events/restart")]
async fn restart_events(controller: web::Data<AnimationController>) -> HttpResponse {
    controller.0.lock().await.restart_event_generators();
    HttpResponse::Ok().json(())
}

#[get("/events/schema")]
async fn events_schema(controller: web::Data<AnimationController>) -> HttpResponse {
    HttpResponse::Ok().json(GetEventGeneratorSchemaResponse {
        event_generators: controller.0.lock().await.get_event_generator_parameters(),
    })
}

#[post("/events/values")]
async fn set_event_parameters(
    params: web::Json<SetEventGeneratorParametersRequest>,
    controller: web::Data<AnimationController>,
) -> HttpResponse {
    match controller
        .0
        .lock()
        .await
        .set_event_generator_parameters(&params.event_generators)
    {
        Ok(_) => HttpResponse::Ok().json(()),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

async fn restore_params(
    controller: &mut Controller,
    configuration: Configuration,
    db: &db::Db,
) -> Result<Configuration, String> {
    match db.get_parameters(&configuration.id).await {
        Ok(Some(values)) => {
            let _ = controller.set_parameters(&values);
            Ok(Configuration {
                values,
                ..configuration
            })
        }
        Ok(None) => {
            match controller.get_parameter_values() {
                Ok(params) => {
                    let _ = db.set_parameters(&configuration.id, &params).await;
                }
                Err(e) => {
                    warn!("Failed to set parameters in DB: {}", e);
                }
            }
            Ok(configuration)
        }
        Err(e) => Err(format!("Failed to load parameters from db: {}", e)),
    }
}

#[post("/reload")]
async fn reload(controller: web::Data<AnimationController>, db: web::Data<Db>) -> HttpResponse {
    let mut controller = controller.0.lock().await;
    let configuration = match controller.reload_animation() {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": format!("{:#}", e) }));
        }
    };

    match restore_params(&mut controller, configuration, &db.0).await {
        Ok(animation) => HttpResponse::Ok().json(SwitchAnimationResponse { animation }),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[post("/switch")]
async fn switch(
    form: web::Json<SwitchAnimationRequest>,
    controller: web::Data<AnimationController>,
    db: web::Data<Db>,
) -> HttpResponse {
    let mut controller = controller.0.lock().await;
    let configuration = match controller.switch_animation(&form.animation) {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": format!("{:#}", e) }));
        }
    };

    if let Some(values) = form.params.clone() {
        let _ = controller.set_parameters(&values);
        HttpResponse::Ok().json(SwitchAnimationResponse {
            animation: Configuration {
                values,
                ..configuration
            },
        })
    } else {
        match restore_params(&mut controller, configuration, &db.0).await {
            Ok(animation) => HttpResponse::Ok().json(SwitchAnimationResponse { animation }),
            Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
        }
    }
}

#[post("/turn_off")]
async fn turn_off(controller: web::Data<AnimationController>) -> HttpResponse {
    controller.0.lock().await.turn_off();
    HttpResponse::Ok().json(())
}

#[get("/params")]
async fn get_params(controller: web::Data<AnimationController>) -> HttpResponse {
    match controller.0.lock().await.get_parameters() {
        Ok(animation) => HttpResponse::Ok().json(GetParametersResponse { animation }),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[post("/params/save")]
async fn save_params(
    controller: web::Data<AnimationController>,
    db: web::Data<Db>,
) -> HttpResponse {
    let controller = controller.0.lock().await;
    let parameter_values = match controller.get_parameter_values() {
        Ok(params) => params,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }))
        }
    };

    let animation = controller.current_animation();
    let Some(animation_id) = animation.as_ref().map(|a| a.animation_id()) else {
        return HttpResponse::PreconditionFailed().json(json!({ "error": "No animation set" }));
    };

    match db.0.set_parameters(animation_id, &parameter_values).await {
        Ok(_) => HttpResponse::Ok().json(()),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[post("/params/reset")]
async fn reset_params(
    controller: web::Data<AnimationController>,
    db: web::Data<Db>,
) -> HttpResponse {
    let mut controller = controller.0.lock().await;
    let animation = controller.current_animation();
    let Some(animation_id) = animation.as_ref().map(|a| a.animation_id()) else {
        return HttpResponse::PreconditionFailed().json(json!({ "error": "No animation set" }));
    };

    match db.0.get_parameters(animation_id).await {
        Ok(Some(params)) => {
            let _ = controller.set_parameters(&params);
            match controller.get_parameters() {
                Ok(animation) => HttpResponse::Ok().json(GetParametersResponse { animation }),
                Err(e) => {
                    HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }))
                }
            }
        }
        Ok(None) => HttpResponse::InternalServerError()
            .json(json!({"error": "No parameters stored for this animation"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[post("/params")]
async fn post_params(
    params: web::Json<SetAnimationParametersRequest>,
    controller: web::Data<AnimationController>,
) -> HttpResponse {
    match controller.0.lock().await.set_parameters(&params.values) {
        Ok(_) => HttpResponse::Ok().json(()),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

fn get_animations(controller: &Controller) -> ListAnimationsResponse {
    ListAnimationsResponse {
        animations: controller
            .list_animations()
            .iter()
            .map(|(id, plugin)| Animation {
                id: id.to_owned(),
                name: plugin.animation_name().to_owned(),
            })
            .collect::<Vec<_>>(),
        current_animation_id: controller
            .current_animation()
            .map(|a| a.animation_id().to_owned()),
    }
}

#[post("/discover")]
async fn discover(controller: web::Data<AnimationController>) -> HttpResponse {
    let mut controller = controller.0.lock().await;
    match controller.discover_animations() {
        Ok(_) => HttpResponse::Ok().json(get_animations(&controller)),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/list")]
async fn list(controller: web::Data<AnimationController>) -> HttpResponse {
    let controller = controller.0.lock().await;
    HttpResponse::Ok().json(get_animations(&controller))
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
    HttpResponse::Ok().json(GetPointsResponse {
        points: controller
            .0
            .lock()
            .await
            .points()
            .iter()
            .map(|(x, y, z)| (*x as f32, *y as f32, *z as f32))
            .collect(),
    })
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
