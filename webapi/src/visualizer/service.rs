use actix::{Actor, Addr};
use actix_web::{get, web, HttpRequest, HttpResponse, Scope};
use actix_web_actors::ws;
use tokio::sync::mpsc;
use webapi_model::GetPointsResponse;

use super::frame_broadcaster::{FrameBroadcaster, FrameBroadcasterSession};
use crate::AnimationController;

#[get("/frames/")]
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

#[get("/points/")]
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

pub fn service_factory(
    frame_receiver: mpsc::Receiver<lightfx::Frame>,
) -> impl Fn() -> Scope + Clone {
    let frame_broadcaster = web::Data::new(FrameBroadcaster::new(frame_receiver).start());

    move || {
        let frame_broadcaster = frame_broadcaster.clone();
        web::scope("/visualizer")
            .app_data(frame_broadcaster)
            .service(frames)
            .service(points)
    }
}
