use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin already registered: {0}")]
    AlreadyRegistered(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin registration failed: {plugin} — {reason}")]
    RegistrationFailed { plugin: String, reason: String },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
