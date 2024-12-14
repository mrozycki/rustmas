use actix_web::{get, post, web, HttpResponse, Scope};
use serde_json::json;
use webapi_model::{GetParametersResponse, SetAnimationParametersRequest};

use crate::{parameters, AnimationController};

#[get("/")]
async fn get(controller: web::Data<AnimationController>) -> HttpResponse {
    match controller.lock().await.get_parameters().await {
        Ok(animation) => HttpResponse::Ok().json(GetParametersResponse {
            animation: Some(animation),
        }),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[post("/")]
async fn post(
    params: web::Json<SetAnimationParametersRequest>,
    controller: web::Data<AnimationController>,
) -> HttpResponse {
    match controller.lock().await.set_parameters(&params.values).await {
        Ok(_) => HttpResponse::Ok().json(()),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[post("/save/")]
async fn save(
    controller: web::Data<AnimationController>,
    parameters: web::Data<parameters::Logic>,
) -> HttpResponse {
    let controller = controller.lock().await;
    match parameters.save(&controller).await {
        Ok(_) => HttpResponse::Ok().json(()),
        Err(parameters::LogicError::InternalError(e)) => {
            HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }))
        }
        Err(e @ parameters::LogicError::NoAnimationSelected) => {
            HttpResponse::PreconditionFailed().json(json!({ "error": e.to_string() }))
        }
    }
}

#[post("/reset/")]
async fn reset(
    controller: web::Data<AnimationController>,
    parameters: web::Data<parameters::Logic>,
) -> HttpResponse {
    let mut controller = controller.lock().await;
    match parameters.reset(&mut controller).await {
        Ok(animation) => HttpResponse::Ok().json(GetParametersResponse {
            animation: Some(animation),
        }),
        Err(parameters::LogicError::InternalError(e)) => {
            HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }))
        }
        Err(e @ parameters::LogicError::NoAnimationSelected) => {
            HttpResponse::PreconditionFailed().json(json!({ "error": e.to_string() }))
        }
    }
}

pub fn service() -> Scope {
    web::scope("/params")
        .service(get)
        .service(save)
        .service(post)
        .service(reset)
}
