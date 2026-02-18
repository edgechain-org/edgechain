use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Command execution failed: {name} — {reason}")]
    CommandFailed { name: String, reason: String },

    #[error("Command input validation failed: {0}")]
    InvalidCommandInput(String),

    #[error("Agent step limit reached ({0} steps)")]
    StepLimitReached(usize),

    #[error("Agent timed out after {0}ms")]
    Timeout(u64),

    #[error("Model error: {0}")]
    Model(#[from] edgechain_model::ModelError),

    #[error("Memory error: {0}")]
    Memory(#[from] edgechain_memory::MemoryError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
