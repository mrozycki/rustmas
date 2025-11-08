use bevy::prelude::*;
use bevy_tasks::AsyncComputeTaskPool;
use ewebsock::{WsEvent, WsMessage};
use itertools::Itertools;
use std::{ops::ControlFlow, sync::Mutex};
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
        let (inner_sender, ws_receiver) = async_channel::unbounded();
        let (ws_sender, inner_receiver) = async_channel::unbounded();

        let mut ews_sender = ewebsock::ws_connect(
            self.endpoint.to_string(),
            Box::new(move |event| match inner_sender.force_send(event) {
                Ok(_) => ControlFlow::Continue(()),
                Err(_) => ControlFlow::Break(()),
            }),
        )
        .unwrap();

        AsyncComputeTaskPool::get().spawn(async move {
            loop {
                let event = inner_receiver.recv().await;
                ews_sender.send(event.unwrap());
            }
        });

        app.insert_resource(WsConnection(Mutex::new((ws_sender, ws_receiver))))
            .add_systems(Update, listen_for_frame);
    }
}

struct WsConnection(
    Mutex<(
        async_channel::Sender<WsMessage>,
        async_channel::Receiver<WsEvent>,
    )>,
);
impl Resource for WsConnection {}

fn listen_for_frame(
    recv: Res<WsConnection>,
    query: Query<(&Handle<StandardMaterial>, &Led)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (sender, receiver) = &mut *recv.0.lock().unwrap();

    let mut last_frame = None;
    while let Ok(event) = receiver.try_recv() {
        if let WsEvent::Message(WsMessage::Binary(bytes)) = event {
            last_frame = Some(bytes);
            break;
        }
    }

    if let Some(frame) = last_frame {
        let colors: Vec<_> = frame.into_iter().tuples::<(u8, u8, u8)>().collect();
        for (material, led) in query.iter() {
            if let Some(color) = colors
                .get(led.0)
                .map(|(r, g, b)| Color::srgb_u8(*r, *g, *b))
            {
                materials.get_mut(material).unwrap().emissive = color.into();
            };
        }
        let _ = sender.force_send(WsMessage::Binary(vec![1]));
    }
}

pub struct RustmasApiClient(pub rustmas_webapi_client::RustmasApiClient);
impl Resource for RustmasApiClient {}
