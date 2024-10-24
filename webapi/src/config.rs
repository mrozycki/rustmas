use std::path::PathBuf;

use rustmas_animator::ControllerConfig;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RustmasConfig {
    pub database_path: PathBuf,
    pub controller: ControllerConfig,
}
