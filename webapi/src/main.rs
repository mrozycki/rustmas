mod config;
mod frame_broadcaster;
mod parameters;

use ::config::Config;
use actix::{Actor, Addr};
use actix_cors::Cors;
use actix_web_actors::ws;
use config::RustmasConfig;
use log::info;
use serde_json::json;
use std::error::Error;
use tokio::sync::{mpsc, Mutex};
use webapi_model::{
    Animation, Configuration, GetEventGeneratorSchemaResponse, GetPointsResponse,
    ListAnimationsResponse, SendEventRequest, SetEventGeneratorParametersRequest,
    SwitchAnimationRequest, SwitchAnimationResponse,
};

use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer};

use crate::frame_broadcaster::{FrameBroadcaster, FrameBroadcasterSession};

#[post("/events/restart")]
async fn restart_events(controller: web::Data<AnimationController>) -> HttpResponse {
    controller.lock().await.restart_event_generators().await;
    HttpResponse::Ok().json(())
}

#[get("/events/schema")]
async fn events_schema(controller: web::Data<AnimationController>) -> HttpResponse {
    HttpResponse::Ok().json(GetEventGeneratorSchemaResponse {
        event_generators: controller
            .lock()
            .await
            .get_event_generator_parameters()
            .await,
    })
}

#[post("/events/values")]
async fn set_event_parameters(
    params: web::Json<SetEventGeneratorParametersRequest>,
    controller: web::Data<AnimationController>,
) -> HttpResponse {
    match controller
        .lock()
        .await
        .set_event_generator_parameters(&params.event_generators)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(()),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[post("/events/send")]
async fn send_event(
    params: web::Json<SendEventRequest>,
    controller: web::Data<AnimationController>,
) -> HttpResponse {
    match controller
        .lock()
        .await
        .send_event(params.into_inner().event)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(()),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[post("/reload")]
async fn reload(
    controller: web::Data<AnimationController>,
    parameters: web::Data<parameters::Logic>,
) -> HttpResponse {
    let mut controller = controller.lock().await;
    let configuration = match controller.reload_animation().await {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": format!("{:#}", e) }));
        }
    };

    match parameters.restore(&mut controller, configuration).await {
        Ok(animation) => HttpResponse::Ok().json(SwitchAnimationResponse { animation }),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[post("/switch")]
async fn switch(
    form: web::Json<SwitchAnimationRequest>,
    controller: web::Data<AnimationController>,
    parameters: web::Data<parameters::Logic>,
) -> HttpResponse {
    let mut controller = controller.lock().await;
    let configuration = match controller.switch_animation(&form.animation).await {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": format!("{:#}", e) }));
        }
    };

    if let Some(values) = form.params.clone() {
        let _ = controller.set_parameters(&values).await;
        HttpResponse::Ok().json(SwitchAnimationResponse {
            animation: Configuration {
                values,
                ..configuration
            },
        })
    } else {
        match parameters.restore(&mut controller, configuration).await {
            Ok(animation) => HttpResponse::Ok().json(SwitchAnimationResponse { animation }),
            Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
        }
    }
}

#[post("/turn_off")]
async fn turn_off(controller: web::Data<AnimationController>) -> HttpResponse {
    controller.lock().await.turn_off().await;
    HttpResponse::Ok().json(())
}

async fn get_animations(controller: &rustmas_animator::Controller) -> ListAnimationsResponse {
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
            .await
            .map(|a| a.animation_id().to_owned()),
    }
}

#[post("/discover")]
async fn discover(controller: web::Data<AnimationController>) -> HttpResponse {
    let mut controller = controller.lock().await;
    match controller.discover_animations() {
        Ok(_) => HttpResponse::Ok().json(get_animations(&controller).await),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/list")]
async fn list(controller: web::Data<AnimationController>) -> HttpResponse {
    let controller = controller.lock().await;
    HttpResponse::Ok().json(get_animations(&controller).await)
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
            .lock()
            .await
            .points()
            .iter()
            .map(|(x, y, z)| (*x as f32, *y as f32, *z as f32))
            .collect(),
    })
}

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

    let (sender, receiver) = mpsc::channel::<lightfx::Frame>(1);

    info!("Setting up database");
    let parameters = web::Data::new(parameters::Logic::from(&config).await?);

    info!("Starting controller");
    let controller = {
        let mut controller = rustmas_animator::Controller::builder_from(config.controller)?
            .lights_feedback(sender)
            .build();

        controller.discover_animations()?;
        info!("Discovered {} plugins", controller.list_animations().len());
        web::Data::new(Mutex::new(controller))
    };

    let frame_broadcaster = web::Data::new(FrameBroadcaster::new(receiver).start());

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(restart_events)
            .service(events_schema)
            .service(send_event)
            .service(set_event_parameters)
            .service(reload)
            .service(switch)
            .service(turn_off)
            .service(list)
            .service(discover)
            .service(parameters::service())
            .service(frames)
            .service(points)
            .app_data(controller.clone())
            .app_data(frame_broadcaster.clone())
            .app_data(parameters.clone())
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await?;

    Ok(())
}
