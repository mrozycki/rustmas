use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::{CameraProjection, Projection};
use bevy::window::PrimaryWindow;

use crate::pan_orbit_camera::PanOrbitCamera;

pub fn send_mouse(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&mut Window, With<PrimaryWindow>>,
    mut ev_position: EventReader<CursorMoved>,
    input_mouse: Res<ButtonInput<MouseButton>>,
) {
    let drag_button = MouseButton::Left;

    if input_mouse.just_pressed(drag_button) {
        let (camera, camera_transform) = camera_query.single();

        let Some(cursor_position) = windows.single().cursor_position() else {
            return;
        };

        let Some(cursor_position2) = ev_position.read().next().map(|ev| ev.position) else {
            return;
        };

        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            return;
        };

        info!("{:?}", ray);
        info!("{:?} {:?}", cursor_position, cursor_position2);
    }
}
