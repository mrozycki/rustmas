use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ByteOrder {
    #[default]
    Rgb,
    Grb,
    Bgr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightsConfig {
    #[serde(flatten)]
    pub endpoint: LightsEndpoint,
    #[serde(default)]
    pub byte_order: ByteOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LightsEndpoint {
    Remote(Url),
    #[serde(with = "tty_serde")]
    Tty(TtyLightsConfig),
}

#[derive(Debug, Clone)]
pub enum TtyLightsConfig {
    Detect,
    Path(PathBuf),
}

mod tty_serde {
    use super::TtyLightsConfig;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::path::PathBuf;

    pub(super) fn serialize<S>(value: &TtyLightsConfig, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            TtyLightsConfig::Detect => serializer.serialize_str("detect"),
            TtyLightsConfig::Path(path_buf) => {
                serializer.serialize_str(path_buf.to_string_lossy().as_ref())
            }
        }
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<TtyLightsConfig, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if s == "detect" {
            Ok(TtyLightsConfig::Detect)
        } else {
            Ok(TtyLightsConfig::Path(PathBuf::from(s)))
        }
    }
}
