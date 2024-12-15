use actix_web::{get, post, web, HttpResponse, Scope};
use serde_json::json;
use webapi_model::{
    Animation, Configuration, ListAnimationsResponse, SwitchAnimationRequest,
    SwitchAnimationResponse,
};

use crate::{parameters, AnimationController};

#[post("/reload/")]
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

#[post("/switch/")]
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

#[post("/turn_off/")]
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

#[post("/discover/")]
async fn discover(controller: web::Data<AnimationController>) -> HttpResponse {
    let mut controller = controller.lock().await;
    match controller.discover_animations() {
        Ok(_) => HttpResponse::Ok().json(get_animations(&controller).await),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/list/")]
async fn list(controller: web::Data<AnimationController>) -> HttpResponse {
    let controller = controller.lock().await;
    HttpResponse::Ok().json(get_animations(&controller).await)
}

pub fn service() -> Scope {
    web::scope("/animations")
        .service(reload)
        .service(switch)
        .service(turn_off)
        .service(discover)
        .service(list)
}
