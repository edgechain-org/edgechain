use async_trait::async_trait;
use crate::error::RagError;

pub type EmbeddingVector = Vec<f32>;

#[async_trait]
pub trait Embedder: Send + Sync {
    fn dimensions(&self) -> usize;
    async fn embed(&self, text: &str) -> Result<EmbeddingVector, RagError>;
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<EmbeddingVector>, RagError> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }
}

/// Stub embedder that returns a deterministic pseudo-embedding based on character codes.
/// Used for development and testing until a real embedding model is integrated.
pub struct StubEmbedder {
    dimensions: usize,
}

impl StubEmbedder {
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }
}

impl Default for StubEmbedder {
    fn default() -> Self {
        Self::new(384)
    }
}

#[async_trait]
impl Embedder for StubEmbedder {
    fn dimensions(&self) -> usize {
        self.dimensions
    }

    async fn embed(&self, text: &str) -> Result<EmbeddingVector, RagError> {
        let bytes = text.as_bytes();
        let mut vec = vec![0.0f32; self.dimensions];
        for (i, &b) in bytes.iter().enumerate() {
            vec[i % self.dimensions] += b as f32 / 255.0;
        }
        // L2-normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        Ok(vec)
    }
}
