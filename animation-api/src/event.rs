use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Event {
    BeatEvent { bpm: f64 },
    FftEvent { bands: Vec<f32> },
}
