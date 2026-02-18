use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::debug;
use crate::{
    document::Document,
    embedder::Embedder,
    error::RagError,
    index::{VectorEntry, VectorIndex},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedChunk {
    pub id: String,
    pub text: String,
    pub score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[async_trait]
pub trait Retriever: Send + Sync {
    async fn index_document(&self, doc: Document) -> Result<(), RagError>;
    async fn remove_document(&self, id: &str) -> Result<(), RagError>;
    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<RetrievedChunk>, RagError>;
    fn document_count(&self) -> usize;
}

/// Local in-process retriever backed by a flat vector index and a pluggable embedder.
pub struct LocalRetriever {
    embedder: Arc<dyn Embedder>,
    index: VectorIndex,
}

impl LocalRetriever {
    pub fn new(embedder: impl Embedder + 'static) -> Self {
        Self {
            embedder: Arc::new(embedder),
            index: VectorIndex::new(),
        }
    }
}

#[async_trait]
impl Retriever for LocalRetriever {
    async fn index_document(&self, doc: Document) -> Result<(), RagError> {
        debug!(id = %doc.id, "Indexing document");
        let embedding = self.embedder.embed(&doc.text).await?;
        self.index.insert(VectorEntry {
            id: doc.id,
            text: doc.text,
            embedding,
            metadata: doc.metadata,
        });
        Ok(())
    }

    async fn remove_document(&self, id: &str) -> Result<(), RagError> {
        self.index.remove(id);
        Ok(())
    }

    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<RetrievedChunk>, RagError> {
        debug!(query = %query, top_k = top_k, "Retriever search");
        let query_embedding = self.embedder.embed(query).await?;
        let results = self.index.search(&query_embedding, top_k)?;
        Ok(results
            .into_iter()
            .map(|(score, entry)| RetrievedChunk {
                id: entry.id,
                text: entry.text,
                score,
                metadata: entry.metadata,
            })
            .collect())
    }

    fn document_count(&self) -> usize {
        self.index.len()
    }
}
