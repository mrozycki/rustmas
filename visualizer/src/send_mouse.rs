use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::window::PrimaryWindow;

use crate::api::RustmasApiClient;

pub fn send_mouse(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&mut Window, With<PrimaryWindow>>,
    // mut ev_position: EventReader<CursorMoved>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    api_client: Res<RustmasApiClient>,
) {
    let drag_button = MouseButton::Left;

    if input_mouse.just_pressed(drag_button) {
        let (camera, camera_transform) = camera_query.single();

        let Some(cursor_position) = windows.single().cursor_position() else {
            return;
        };

        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            return;
        };

        info!("{:?}", ray);

        let api = api_client.0.clone();
        AsyncComputeTaskPool::get().spawn_local(async move {
            api.send_event(webapi_model::Event::MouseEvent {
                ray_origin: (ray.origin.x, ray.origin.y, -ray.origin.z),
                ray_direction: (ray.direction.x, ray.direction.y, -ray.direction.z),
                mouse_event_type: webapi_model::MouseEventType::MouseDown,
            })
            .await
        });
    }
}
