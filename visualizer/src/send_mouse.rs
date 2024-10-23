use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;

use crate::api::RustmasApiClient;

pub fn send_mouse(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut ev_position: EventReader<CursorMoved>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    api_client: Res<RustmasApiClient>,
) {
    let drag_button = MouseButton::Left;
    let (camera, camera_transform) = camera_query.single();

    let mouse_down = input_mouse
        .just_pressed(drag_button)
        .then_some(webapi_model::Event::MouseDown)
        .into_iter();

    let mouse_up = input_mouse
        .just_released(drag_button)
        .then_some(webapi_model::Event::MouseUp)
        .into_iter();

    let mouse_move = ev_position.read().flat_map(|a| {
        let ray = camera.viewport_to_world(camera_transform, a.position)?;

        Some(webapi_model::Event::MouseMove {
            ray_origin: [ray.origin.x, ray.origin.y, -ray.origin.z],
            ray_direction: [ray.direction.x, ray.direction.y, -ray.direction.z],
        })
    });

    for event in mouse_down.chain(mouse_move).chain(mouse_up) {
        let api = api_client.0.clone();
        AsyncComputeTaskPool::get().spawn_local(async move { api.send_event(event).await });
    }
}
