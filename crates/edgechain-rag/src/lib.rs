pub mod error;
pub mod embedder;
pub mod index;
pub mod retriever;
pub mod document;
pub mod gguf_embedder;
pub mod connectors;

pub use error::RagError;
pub use embedder::{Embedder, EmbeddingVector, StubEmbedder};

#[cfg(feature = "gguf")]
pub use gguf_embedder::GgufEmbedder;
pub use index::{VectorIndex, VectorEntry};
pub use retriever::{Retriever, RetrievedChunk, LocalRetriever};
pub use connectors::{SqliteConnector, FileConnector};
pub use document::Document;
