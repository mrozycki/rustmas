use midi_msg::MidiMsg;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Event {
    BeatEvent {
        bpm: f64,
    },
    FftEvent {
        bands: Vec<f32>,
    },
    MidiEvent(
        #[serde(
            serialize_with = "serialize_midi_msg",
            deserialize_with = "deserialize_midi_msg"
        )]
        MidiMsg,
    ),
}

fn serialize_midi_msg<S>(midi_msg: &MidiMsg, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_bytes(&midi_msg.to_midi())
}

fn deserialize_midi_msg<'de, D>(deserializer: D) -> Result<MidiMsg, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: Vec<u8> = de::Deserialize::deserialize(deserializer)?;
    MidiMsg::from_midi(&bytes)
        .map(|m| m.0)
        .map_err(de::Error::custom)
}
