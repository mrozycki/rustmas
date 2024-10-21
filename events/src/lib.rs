#[cfg(feature = "audio")]
pub mod beat_generator;

pub mod event_generator;

#[cfg(feature = "audio")]
pub mod fft_generator;

#[cfg(feature = "midi")]
pub mod midi_generator;
