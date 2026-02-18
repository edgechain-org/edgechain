use thiserror::Error;

#[derive(Debug, Error)]
pub enum RagError {
    #[error("Embedding failed: {0}")]
    EmbeddingFailed(String),

    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Document not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
