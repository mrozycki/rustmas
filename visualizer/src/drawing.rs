use bevy::{prelude::*, window::PrimaryWindow};

use crate::async_task_pool::TaskPool;

pub(crate) struct LastDrawPoint(pub Option<(f32, f32)>);
impl Resource for LastDrawPoint {}

pub(crate) struct RustmasApiClient(pub rustmas_webapi_client::RustmasApiClient);
impl Resource for RustmasApiClient {}

fn get_cursor_position(q_windows: &Query<&Window, With<PrimaryWindow>>) -> Option<(f32, f32)> {
    q_windows.single().cursor_position().map(|v| (v.x, v.y))
}

pub(crate) fn draw_events_generator(
    buttons: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut last_draw_point: ResMut<LastDrawPoint>,
    api: Res<RustmasApiClient>,
    task_pool: Res<TaskPool>,
) {
    if buttons.pressed(MouseButton::Left) {
        let new_position = get_cursor_position(&q_windows);

        if let Some((new_position, old_position)) = new_position.zip(last_draw_point.0) {
            if new_position != old_position {
                let api = api.0.clone();
                task_pool.spawn(async move {
                    let _ = api
                        .send_event(rustmas_webapi_client::Event::DrawEvent {
                            from: old_position,
                            to: new_position,
                        })
                        .await;
                });
            }
        }

        last_draw_point.0 = new_position;
    } else {
        last_draw_point.0 = None;
    }
}
