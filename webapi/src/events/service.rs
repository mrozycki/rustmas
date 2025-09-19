use actix_web::{HttpResponse, Scope, get, post, web};
use serde_json::json;
use webapi_model::{
    GetEventGeneratorSchemaResponse, SendEventRequest, SetEventGeneratorParametersRequest,
};

use crate::AnimationController;

#[post("/restart/")]
async fn restart(controller: web::Data<AnimationController>) -> HttpResponse {
    controller.lock().await.restart_event_generators().await;
    HttpResponse::Ok().json(())
}

#[get("/schema/")]
async fn get_schema(controller: web::Data<AnimationController>) -> HttpResponse {
    HttpResponse::Ok().json(GetEventGeneratorSchemaResponse {
        event_generators: controller
            .lock()
            .await
            .get_event_generator_parameters()
            .await,
    })
}

#[post("/values/")]
async fn set_parameters(
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

#[post("/send/")]
async fn send(
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

pub fn service() -> Scope {
    web::scope("/events")
        .service(restart)
        .service(get_schema)
        .service(set_parameters)
        .service(send)
}
