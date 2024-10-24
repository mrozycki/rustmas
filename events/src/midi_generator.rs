use std::{collections::HashMap, error::Error, sync::Mutex};

use animation_api::{
    event::Event,
    schema::{ConfigurationSchema, EnumOption, ParameterSchema, ParameterValue, ValueSchema},
};
use anyhow::anyhow;
use midi_msg::{ChannelVoiceMsg, MidiMsg, ReceiverContext};
use midir::{MidiInput, MidiInputConnection, PortInfoError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc;
use tracing::info;

use crate::event_generator::EventGenerator;

struct MidiStream {
    _midi_connection: Mutex<MidiInputConnection<()>>,
}

impl MidiStream {
    fn new(channel: mpsc::Sender<Event>, device_name: &str) -> Result<Self, Box<dyn Error>> {
        let midi_in = MidiInput::new("Rustmas")?;
        let ports = midi_in.ports();
        let port = ports
            .into_iter()
            .find(|p| midi_in.port_name(p).is_ok_and(|n| n == device_name))
            .ok_or(anyhow!("Selected MIDI port not found: {}", device_name))?;
        info!("Using MIDI device: {:?}", midi_in.port_name(&port));

        let mut ctx = ReceiverContext::new();
        Ok(Self {
            _midi_connection: Mutex::new(midi_in.connect(
                &port,
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

#[derive(Clone, Serialize, Deserialize)]
pub struct Parameters {
    device: String,
}

impl Parameters {
    fn from(midi_input: &MidiInput) -> Self {
        Self {
            device: midi_input
                .ports()
                .first()
                .and_then(|p| midi_input.port_name(p).ok())
                .unwrap_or("default".into()),
        }
    }
}

pub struct MidiEventGenerator {
    midi_input: Mutex<MidiInput>,
    midi_stream: Mutex<Option<MidiStream>>,
    event_sender: mpsc::Sender<Event>,
    parameters: Parameters,
}

impl MidiEventGenerator {
    pub fn new(event_sender: mpsc::Sender<Event>) -> Self {
        let midi_input = MidiInput::new("Rustmas Parameters").unwrap();
        let parameters = Parameters::from(&midi_input);

        Self {
            midi_input: Mutex::new(midi_input),
            midi_stream: Mutex::new(MidiStream::new(event_sender.clone(), &parameters.device).ok()),
            event_sender,
            parameters,
        }
    }
}

impl EventGenerator for MidiEventGenerator {
    fn get_name(&self) -> &str {
        "MIDI"
    }

    fn restart(&mut self) {
        *self.midi_stream.lock().unwrap() =
            MidiStream::new(self.event_sender.clone(), &self.parameters.device).ok();
    }

    fn get_schema(&self) -> ConfigurationSchema {
        let midi_input = self.midi_input.lock().unwrap();
        ConfigurationSchema {
            parameters: vec![ParameterSchema {
                id: "device".into(),
                name: "MIDI Input Device".into(),
                description: None,
                value: ValueSchema::Enum {
                    values: midi_input
                        .ports()
                        .into_iter()
                        .map(|port| -> Result<_, PortInfoError> {
                            let name = midi_input.port_name(&port)?;
                            Ok(EnumOption {
                                name: name.clone(),
                                description: None,
                                value: name,
                            })
                        })
                        .flat_map(|p| p.ok())
                        .collect(),
                },
            }],
        }
    }

    fn get_parameters(&self) -> HashMap<String, ParameterValue> {
        serde_json::from_value(json!(self.parameters)).unwrap()
    }

    fn set_parameters(
        &mut self,
        parameters: &HashMap<String, ParameterValue>,
    ) -> Result<(), serde_json::Error> {
        self.parameters = serde_json::from_value(json!(parameters))?;
        self.restart();
        Ok(())
    }
}
