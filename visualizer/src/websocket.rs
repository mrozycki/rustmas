use bevy::prelude::*;
use ewebsock::{WsEvent, WsMessage};
use itertools::Itertools;
use std::{
    ops::ControlFlow,
    sync::{mpsc, Mutex},
};
use url::Url;

use crate::Led;

pub(crate) struct WebsocketPlugin {
    endpoint: Url,
}

impl WebsocketPlugin {
    pub(crate) fn new(endpoint: Url) -> Self {
        Self { endpoint }
    }
}

impl Plugin for WebsocketPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = mpsc::channel();
        ewebsock::ws_receive(
            self.endpoint.to_string(),
            Box::new(move |event| match sender.send(event) {
                Ok(_) => ControlFlow::Continue(()),
                Err(_) => ControlFlow::Break(()),
            }),
        )
        .unwrap();
        app.insert_resource(Receiver(Mutex::new(receiver)))
            .add_systems(Update, listen_for_frame);
    }
}

struct Receiver(Mutex<mpsc::Receiver<WsEvent>>);
impl Resource for Receiver {}

fn listen_for_frame(
    recv: Res<Receiver>,
    query: Query<(&Handle<StandardMaterial>, &Led)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut last_frame = None;

    while let Ok(event) = recv.0.lock().unwrap().try_recv() {
        if let WsEvent::Message(WsMessage::Binary(bytes)) = event {
            last_frame = Some(bytes);
        }
    }

    if let Some(frame) = last_frame {
        let colors: Vec<_> = frame.into_iter().tuples::<(u8, u8, u8)>().collect();
        for (material, led) in query.iter() {
            if let Some(color) = colors.get(led.0).map(|(r, g, b)| Color::rgb_u8(*r, *g, *b)) {
                materials.get_mut(material).unwrap().emissive = color;
            };
        }
    }
}
