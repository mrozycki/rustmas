mod animation_list;
#[cfg(feature = "visualizer")]
mod visualizer;

pub use animation_list::AnimationList;

#[cfg(not(feature = "visualizer"))]
pub use crate::utils::Dummy as Visualizer;
#[cfg(feature = "visualizer")]
pub use visualizer::Visualizer;
