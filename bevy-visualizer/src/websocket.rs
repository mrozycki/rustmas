use bevy::prelude::*;
use bevy_websocket_adapter::{impl_message_type, server::Server, shared::ConnectionHandle};
use serde::{Deserialize, Serialize};

use crate::Led;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct FrameEvent {
    pixels: Vec<Color>,
}
impl_message_type!(FrameEvent, "frame");

pub(crate) fn start_listen(mut ws: ResMut<Server>) {
    ws.listen("0.0.0.0:12345")
        .expect("failed to start websocket server");
}

pub(crate) fn listen_for_frame(
    mut evs: EventReader<(ConnectionHandle, FrameEvent)>,
    query: Query<(&Handle<StandardMaterial>, &Led)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Some((handle, ev)) = evs.iter().last() {
        trace!(
            "received FrameEvent from {:?}; size = {}",
            handle,
            ev.pixels.len()
        );
        for (material, led) in query.iter() {
            let Some(color) = ev.pixels.get(led.0) else {
                continue;
            };
            materials.get_mut(material).unwrap().base_color =
                bevy::prelude::Color::rgb_u8(color.r, color.g, color.b);
        }
    }
}
