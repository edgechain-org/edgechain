use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Model not loaded: {0}")]
    NotLoaded(String),

    #[error("Inference failed: {0}")]
    InferenceFailed(String),

    #[error("Context window exceeded: max {max}, got {got}")]
    ContextWindowExceeded { max: usize, got: usize },

    #[error("Invalid model format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
