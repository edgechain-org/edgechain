use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::{embedder::EmbeddingVector, error::RagError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: String,
    pub text: String,
    pub embedding: EmbeddingVector,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// In-process flat vector index with cosine similarity search.
/// Suitable for small corpora (up to ~10k documents).
/// Will be replaced with HNSW for larger corpora in v0.2.
pub struct VectorIndex {
    entries: RwLock<Vec<VectorEntry>>,
}

impl VectorIndex {
    pub fn new() -> Self {
        Self { entries: RwLock::new(vec![]) }
    }

    pub fn insert(&self, entry: VectorEntry) {
        let mut entries = self.entries.write().unwrap();
        entries.retain(|e| e.id != entry.id);
        entries.push(entry);
    }

    pub fn remove(&self, id: &str) {
        self.entries.write().unwrap().retain(|e| e.id != id);
    }

    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the top-k most similar entries by cosine similarity.
    pub fn search(&self, query: &EmbeddingVector, top_k: usize) -> Result<Vec<(f32, VectorEntry)>, RagError> {
        let entries = self.entries.read().unwrap();
        let mut scored: Vec<(f32, &VectorEntry)> = entries
            .iter()
            .map(|e| (cosine_similarity(query, &e.embedding), e))
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        Ok(scored.into_iter().map(|(score, e)| (score, e.clone())).collect())
    }
}

impl Default for VectorIndex {
    fn default() -> Self {
        Self::new()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) }
}
