//! GGUF-backed embedder using llama.cpp embedding models.
//!
//! Enable with the `gguf` feature on `edgechain-rag`:
//! ```toml
//! edgechain-rag = { path = "...", features = ["gguf"] }
//! ```

#[cfg(feature = "gguf")]
mod inner {
    use std::path::PathBuf;
    use std::sync::Arc;
    use async_trait::async_trait;
    use tracing::info;

    use llama_cpp_2::{
        context::params::LlamaContextParams,
        llama_backend::LlamaBackend,
        llama_batch::LlamaBatch,
        model::{params::LlamaModelParams, AddBos, LlamaModel},
    };

    use crate::{embedder::{Embedder, EmbeddingVector}, error::RagError};

    pub struct GgufEmbedder {
        model: Arc<LlamaModel>,
        backend: Arc<LlamaBackend>,
        dimensions: usize,
    }

    impl GgufEmbedder {
        pub fn load(model_path: impl Into<PathBuf>) -> Result<Self, RagError> {
            let path: PathBuf = model_path.into();
            info!(path = %path.display(), "Loading GGUF embedding model");

            let backend = LlamaBackend::init()
                .map_err(|e| RagError::EmbeddingFailed(e.to_string()))?;

            let model_params = LlamaModelParams::default();
            let model = LlamaModel::load_from_file(&backend, &path, &model_params)
                .map_err(|e| RagError::EmbeddingFailed(e.to_string()))?;

            // Infer embedding dimensions from the model's embedding size
            let dimensions = model.n_embd() as usize;

            info!(dimensions, "GGUF embedding model loaded");

            Ok(Self {
                model: Arc::new(model),
                backend: Arc::new(backend),
                dimensions,
            })
        }

        fn embed_blocking(
            model: &LlamaModel,
            backend: &LlamaBackend,
            text: &str,
        ) -> Result<EmbeddingVector, RagError> {
            let ctx_params = LlamaContextParams::default()
                .with_embeddings(true);

            let mut ctx = model
                .new_context(backend, ctx_params)
                .map_err(|e| RagError::EmbeddingFailed(e.to_string()))?;

            let tokens = model
                .str_to_token(text, AddBos::Always)
                .map_err(|e| RagError::EmbeddingFailed(e.to_string()))?;

            let mut batch = LlamaBatch::new(tokens.len().max(512), 1);
            for (i, &token) in tokens.iter().enumerate() {
                let is_last = i == tokens.len() - 1;
                batch
                    .add(token, i as i32, &[0], is_last)
                    .map_err(|e| RagError::EmbeddingFailed(e.to_string()))?;
            }

            ctx.decode(&mut batch)
                .map_err(|e| RagError::EmbeddingFailed(e.to_string()))?;

            let embedding = ctx
                .embeddings_seq_ith(0)
                .map_err(|e| RagError::EmbeddingFailed(e.to_string()))?
                .to_vec();

            // L2-normalize
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            let normalized = if norm > 0.0 {
                embedding.iter().map(|x| x / norm).collect()
            } else {
                embedding
            };

            Ok(normalized)
        }
    }

    #[async_trait]
    impl Embedder for GgufEmbedder {
        fn dimensions(&self) -> usize {
            self.dimensions
        }

        async fn embed(&self, text: &str) -> Result<EmbeddingVector, RagError> {
            let model = Arc::clone(&self.model);
            let backend = Arc::clone(&self.backend);
            let text = text.to_string();

            tokio::task::spawn_blocking(move || {
                Self::embed_blocking(&model, &backend, &text)
            })
            .await
            .map_err(|e| RagError::EmbeddingFailed(e.to_string()))?
        }
    }
}

#[cfg(feature = "gguf")]
pub use inner::GgufEmbedder;
