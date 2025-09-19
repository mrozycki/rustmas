use actix_web::{HttpResponse, Scope, get, post, web};
use log::error;
use serde_json::json;
use webapi_model::{SwitchAnimationRequest, SwitchAnimationResponse};

use crate::{AnimationController, animations, parameters};

#[post("/reload/")]
async fn reload(
    animations: web::Data<animations::Logic>,
    controller: web::Data<AnimationController>,
    parameters: web::Data<parameters::Logic>,
) -> HttpResponse {
    let mut controller = controller.lock().await;
    match animations.reload(&mut controller, &parameters).await {
        Ok(animation) => HttpResponse::Ok().json(SwitchAnimationResponse { animation }),
        Err(animations::LogicError::InternalError(e)) => {
            HttpResponse::InternalServerError().json(json!({ "error": e }))
        }
        Err(animations::LogicError::InvalidAnimation(e)) => {
            HttpResponse::NotAcceptable().json(json!({ "error": e.to_string() }))
        }
        Err(animations::LogicError::NoSuchAnimation(animation_id)) => HttpResponse::NotAcceptable()
            .json(json!({ "error": format!("no such animation: {animation_id}, it might have been removed")} )),
        Err(e @ animations::LogicError::NoAnimationSelected) => {
            HttpResponse::BadRequest().json(json!({ "error": e.to_string() }))
        }
    }
}

#[post("/switch/")]
async fn switch(
    mut form: web::Json<SwitchAnimationRequest>,
    animations: web::Data<animations::Logic>,
    controller: web::Data<AnimationController>,
    parameters: web::Data<parameters::Logic>,
) -> HttpResponse {
    let mut controller = controller.lock().await;
    let initial_parameters = form.params.take();
    match animations
        .switch(
            &form.animation_id,
            initial_parameters,
            &mut controller,
            &parameters,
        )
        .await
    {
        Ok(animation) => HttpResponse::Ok().json(SwitchAnimationResponse { animation }),
        Err(animations::LogicError::InternalError(e)) => {
            HttpResponse::InternalServerError().json(json!({ "error": e }))
        }
        Err(animations::LogicError::InvalidAnimation(e)) => {
            HttpResponse::NotAcceptable().json(json!({ "error": e.to_string() }))
        }
        Err(animations::LogicError::NoSuchAnimation(animation_id)) => HttpResponse::NotFound()
            .json(json!({ "error": format!("no such animation: {animation_id}") })),
        Err(e @ animations::LogicError::NoAnimationSelected) => {
            error!("Unexpected logic error in /switch/ call: {e}");
            HttpResponse::InternalServerError().json(json!({"error": "unexpected failure"}))
        }
    }
}

#[post("/turn_off/")]
async fn turn_off(
    animations: web::Data<animations::Logic>,
    controller: web::Data<AnimationController>,
) -> HttpResponse {
    let mut controller = controller.lock().await;
    animations.turn_off(&mut controller).await;
    HttpResponse::Ok().json(())
}

#[post("/discover/")]
async fn discover(
    animations: web::Data<animations::Logic>,
    controller: web::Data<AnimationController>,
) -> HttpResponse {
    let mut controller = controller.lock().await;
    match animations.discover(&mut controller).await {
        Ok(animations) => HttpResponse::Ok().json(animations),
        Err(animations::LogicError::InternalError(e)) => {
            HttpResponse::InternalServerError().json(json!({"error": e}))
        }
        Err(e) => {
            error!("Unexpected logic error in /discover/ call: {e}");
            HttpResponse::InternalServerError().json(json!({"error": "unexpected failure"}))
        }
    }
}

#[get("/list/")]
async fn list(
    animations: web::Data<animations::Logic>,
    controller: web::Data<AnimationController>,
) -> HttpResponse {
    let controller = controller.lock().await;
    match animations.list(&controller).await {
        Ok(animations) => HttpResponse::Ok().json(animations),
        Err(animations::LogicError::InternalError(e)) => {
            HttpResponse::InternalServerError().json(json!({"error": e}))
        }
        Err(e) => {
            error!("Unexpected logic error in /list/ call: {e}");
            HttpResponse::InternalServerError().json(json!({"error": "unexpected failure"}))
        }
    }
}

pub fn service() -> Scope {
    web::scope("/animations")
        .service(reload)
        .service(switch)
        .service(turn_off)
        .service(discover)
        .service(list)
}
