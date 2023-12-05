use std::error::Error;

use animation_api::event::Event;
use midi_msg::{ChannelVoiceMsg, MidiMsg, ReceiverContext};
use midir::{MidiInput, MidiInputConnection};
use tokio::sync::{mpsc, Mutex};

pub struct MidiEventGenerator {
    _midi_connection: Mutex<MidiInputConnection<()>>,
}

impl MidiEventGenerator {
    pub fn new(channel: mpsc::Sender<Event>) -> Result<Self, Box<dyn Error>> {
        let midi_in = MidiInput::new("Rustmas")?;
        let ports = midi_in.ports();
        let Some(port) = ports.get(1) else {
            return Err("Midi Port 1 not found".into());
        };

        let mut ctx = ReceiverContext::new();
        Ok(Self {
            _midi_connection: Mutex::new(midi_in.connect(
                port,
                "midir-read-input",
                move |_stamp, message, _| {
                    let (msg, _len) =
                        MidiMsg::from_midi_with_context(message, &mut ctx).expect("Not an error");
                    let event = match msg {
                        ref midi_msg @ MidiMsg::ChannelVoice {
                            msg: ChannelVoiceMsg::NoteOn { velocity, note },
                            channel,
                        } => {
                            if velocity == 0 {
                                Some(MidiMsg::ChannelVoice {
                                    msg: ChannelVoiceMsg::NoteOff {
                                        note,
                                        velocity: 255,
                                    },
                                    channel,
                                })
                            } else {
                                Some(midi_msg.clone())
                            }
                        }
                        midi_msg => Some(midi_msg),
                    };
                    if let Some(event) = event {
                        let _ = channel.blocking_send(Event::MidiEvent(event));
                    }
                },
                (),
            )?),
        })
    }
}
