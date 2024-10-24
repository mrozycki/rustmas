use std::path::PathBuf;

use rustmas_light_client::LightsConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerConfig {
    pub points_path: PathBuf,
    pub lights: Vec<LightsConfig>,
    pub plugin_path: PathBuf,
}
