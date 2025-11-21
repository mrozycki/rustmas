#[cfg(feature = "wrap")]
pub mod wrap;

#[cfg(feature = "unwrap")]
pub mod unwrap;

#[derive(Debug, thiserror::Error)]
pub enum PluginConfigError {
    #[error("Failed to parse manifest: {reason}")]
    InvalidManifest { reason: String },

    #[error("Failed to unwrap plugin")]
    InvalidCrab(#[from] unwrap::PluginUnwrapError),

    #[error("Directory containing plugin has non UTF-8 name")]
    NonUtf8DirectoryName,
}
