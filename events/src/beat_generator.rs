use animation_api::event::Event;
use tokio::sync::mpsc;

use std::time::Duration;

use crate::event_generator::EventGenerator;

pub struct BeatEventGenerator {
    _join_handle: tokio::task::JoinHandle<()>,
}

impl BeatEventGenerator {
    pub fn new(bpm: f64, channel: mpsc::Sender<Event>) -> Self {
        Self {
            _join_handle: tokio::spawn(async move {
                while channel.send(Event::BeatEvent { bpm }).await.is_ok() {
                    tokio::time::sleep(Duration::from_secs_f64(60.0 / bpm)).await;
                }
            }),
        }
    }
}

impl EventGenerator for BeatEventGenerator {
    fn get_name(&self) -> &str {
        "Beats"
    }
}
