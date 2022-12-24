mod animation;
mod brightness_controlled;
mod speed_controlled;
mod utils;

pub(crate) mod barber_pole;
pub(crate) mod blank;
pub(crate) mod check;
pub(crate) mod classic;
pub(crate) mod detection_status;
pub(crate) mod indexing;
pub(crate) mod manual_sweep;
pub(crate) mod present;
pub(crate) mod rainbow_cable;
pub(crate) mod rainbow_cylinder;
pub(crate) mod rainbow_halves;
pub(crate) mod rainbow_sphere;
pub(crate) mod rainbow_spiral;
pub(crate) mod rainbow_waterfall;
pub(crate) mod random_sweep;
pub(crate) mod snow;
pub(crate) mod spinning_halves;
pub(crate) mod stars;
pub(crate) mod sweep;

pub use animation::make_animation;
pub use animation::Animation;
pub use animation::AnimationParameters;
