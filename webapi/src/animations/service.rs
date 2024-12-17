use actix_web::{get, post, web, HttpResponse, Scope};
use serde_json::json;
use webapi_model::{SwitchAnimationRequest, SwitchAnimationResponse};

use crate::{animations, parameters, AnimationController};

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
    }
}

#[get("/list/")]
async fn list(
    animations: web::Data<animations::Logic>,
    controller: web::Data<AnimationController>,
) -> HttpResponse {
    let controller = controller.lock().await;
    HttpResponse::Ok().json(animations.list(&controller).await)
}

pub fn service() -> Scope {
    web::scope("/animations")
        .service(reload)
        .service(switch)
        .service(turn_off)
        .service(discover)
        .service(list)
}
