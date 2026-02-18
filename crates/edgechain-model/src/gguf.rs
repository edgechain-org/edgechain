//! GGUF model provider backed by llama.cpp via the `llama-cpp-2` crate.
//!
//! Enable with the `gguf` feature flag:
//! ```toml
//! edgechain-model = { path = "...", features = ["gguf"] }
//! ```

#[cfg(feature = "gguf")]
mod inner {
    use std::path::PathBuf;
    use std::sync::Arc;
    use async_trait::async_trait;
    use tracing::{debug, info, warn};

    use llama_cpp_2::{
        context::params::LlamaContextParams,
        llama_backend::LlamaBackend,
        llama_batch::LlamaBatch,
        model::{params::LlamaModelParams, AddBos, LlamaModel, Special},
        sampling::LlamaSampler,
        token::data_array::LlamaTokenDataArray,
    };

    use crate::{
        error::ModelError,
        provider::{FinishReason, ModelMessage, ModelProvider, ModelRequest, ModelResponse, Role},
    };

    /// Configuration for the GGUF model provider.
    #[derive(Debug, Clone)]
    pub struct GgufConfig {
        /// Path to the `.gguf` model file.
        pub model_path: PathBuf,
        /// Number of GPU layers to offload (0 = CPU only).
        pub n_gpu_layers: u32,
        /// Context size in tokens.
        pub context_size: u32,
        /// Number of threads for inference.
        pub n_threads: Option<u32>,
        /// Seed for sampling (0 = random).
        pub seed: u32,
    }

    impl GgufConfig {
        pub fn new(model_path: impl Into<PathBuf>) -> Self {
            Self {
                model_path: model_path.into(),
                n_gpu_layers: 0,
                context_size: 4096,
                n_threads: None,
                seed: 0,
            }
        }

        pub fn with_gpu_layers(mut self, n: u32) -> Self {
            self.n_gpu_layers = n;
            self
        }

        pub fn with_context_size(mut self, size: u32) -> Self {
            self.context_size = size;
            self
        }

        pub fn with_threads(mut self, n: u32) -> Self {
            self.n_threads = Some(n);
            self
        }
    }

    /// On-device GGUF model provider using llama.cpp.
    ///
    /// Inference runs on a `tokio::task::spawn_blocking` thread to avoid
    /// blocking the async executor.
    pub struct GgufModelProvider {
        config: GgufConfig,
        model: Arc<LlamaModel>,
        backend: Arc<LlamaBackend>,
    }

    impl GgufModelProvider {
        /// Load a GGUF model from disk. Blocks until the model is loaded.
        pub fn load(config: GgufConfig) -> Result<Self, ModelError> {
            info!(path = %config.model_path.display(), "Loading GGUF model");

            let backend = LlamaBackend::init()
                .map_err(|e| ModelError::NotLoaded(e.to_string()))?;

            let model_params = LlamaModelParams::default()
                .with_n_gpu_layers(config.n_gpu_layers);

            let model = LlamaModel::load_from_file(&backend, &config.model_path, &model_params)
                .map_err(|e| ModelError::NotLoaded(e.to_string()))?;

            info!(path = %config.model_path.display(), "GGUF model loaded");

            Ok(Self {
                config,
                model: Arc::new(model),
                backend: Arc::new(backend),
            })
        }

        /// Format messages into a single prompt string using a simple ChatML template.
        /// Real models should use their own chat template via `apply_chat_template`.
        fn format_prompt(&self, messages: &[ModelMessage]) -> String {
            let mut prompt = String::new();
            for msg in messages {
                match msg.role {
                    Role::System => {
                        prompt.push_str("<|im_start|>system\n");
                        prompt.push_str(&msg.content);
                        prompt.push_str("\n<|im_end|>\n");
                    }
                    Role::User => {
                        prompt.push_str("<|im_start|>user\n");
                        prompt.push_str(&msg.content);
                        prompt.push_str("\n<|im_end|>\n");
                    }
                    Role::Assistant => {
                        prompt.push_str("<|im_start|>assistant\n");
                        prompt.push_str(&msg.content);
                        prompt.push_str("\n<|im_end|>\n");
                    }
                    Role::Tool => {
                        prompt.push_str("<|im_start|>tool\n");
                        prompt.push_str(&msg.content);
                        prompt.push_str("\n<|im_end|>\n");
                    }
                }
            }
            prompt.push_str("<|im_start|>assistant\n");
            prompt
        }
    }

    #[async_trait]
    impl ModelProvider for GgufModelProvider {
        fn name(&self) -> &str {
            "gguf"
        }

        fn context_window(&self) -> usize {
            self.config.context_size as usize
        }

        fn is_available(&self) -> bool {
            self.config.model_path.exists()
        }

        async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
            let model = Arc::clone(&self.model);
            let backend = Arc::clone(&self.backend);
            let prompt = self.format_prompt(&request.messages);
            let max_tokens = request.max_tokens.unwrap_or(512);
            let temperature = request.temperature.unwrap_or(0.7);
            let context_size = self.config.context_size;
            let n_threads = self.config.n_threads;
            let stop_sequences = request.stop_sequences.clone();

            debug!(prompt_len = prompt.len(), max_tokens, "GgufModelProvider::complete");

            // Run blocking inference on a dedicated thread pool thread.
            let result = tokio::task::spawn_blocking(move || {
                run_inference(
                    &model,
                    &backend,
                    &prompt,
                    max_tokens,
                    temperature,
                    context_size,
                    n_threads,
                    &stop_sequences,
                )
            })
            .await
            .map_err(|e| ModelError::InferenceFailed(e.to_string()))??;

            Ok(result)
        }
    }

    fn run_inference(
        model: &LlamaModel,
        _backend: &LlamaBackend,
        prompt: &str,
        max_tokens: usize,
        temperature: f32,
        context_size: u32,
        n_threads: Option<u32>,
        stop_sequences: &[String],
    ) -> Result<ModelResponse, ModelError> {
        let mut ctx_params = LlamaContextParams::default()
            .with_n_ctx(std::num::NonZeroU32::new(context_size));

        if let Some(threads) = n_threads {
            ctx_params = ctx_params.with_n_threads(threads);
        }

        let mut ctx = model
            .new_context(_backend, ctx_params)
            .map_err(|e| ModelError::InferenceFailed(e.to_string()))?;

        // Tokenize the prompt
        let tokens = model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| ModelError::InferenceFailed(e.to_string()))?;

        if tokens.len() >= context_size as usize {
            return Err(ModelError::ContextWindowExceeded {
                max: context_size as usize,
                got: tokens.len(),
            });
        }

        // Build initial batch
        let mut batch = LlamaBatch::new(context_size as usize, 1);
        let last_idx = (tokens.len() - 1) as i32;
        for (i, &token) in tokens.iter().enumerate() {
            batch
                .add(token, i as i32, &[0], i as i32 == last_idx)
                .map_err(|e| ModelError::InferenceFailed(e.to_string()))?;
        }

        ctx.decode(&mut batch)
            .map_err(|e| ModelError::InferenceFailed(e.to_string()))?;

        // Sampling setup
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(temperature),
            LlamaSampler::dist(0),
        ]);

        let mut output = String::new();
        let mut n_cur = tokens.len() as i32;
        let mut tokens_generated = 0usize;
        let mut finish_reason = FinishReason::Stop;

        let eos_token = model.token_eos();

        loop {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);

            if token == eos_token {
                break;
            }

            let piece = model
                .token_to_str(token, Special::Tokenize)
                .map_err(|e| ModelError::InferenceFailed(e.to_string()))?;

            output.push_str(&piece);
            tokens_generated += 1;

            // Check stop sequences
            if stop_sequences.iter().any(|s| output.contains(s.as_str())) {
                // Trim at the stop sequence boundary
                for seq in stop_sequences {
                    if let Some(pos) = output.find(seq.as_str()) {
                        output.truncate(pos);
                    }
                }
                break;
            }

            // Check stop tokens for ChatML
            if output.contains("<|im_end|>") {
                if let Some(pos) = output.find("<|im_end|>") {
                    output.truncate(pos);
                }
                break;
            }

            if tokens_generated >= max_tokens {
                warn!(max_tokens, "Max tokens reached");
                finish_reason = FinishReason::Length;
                break;
            }

            // Prepare next batch
            batch.clear();
            batch
                .add(token, n_cur, &[0], true)
                .map_err(|e| ModelError::InferenceFailed(e.to_string()))?;

            n_cur += 1;

            ctx.decode(&mut batch)
                .map_err(|e| ModelError::InferenceFailed(e.to_string()))?;
        }

        Ok(ModelResponse {
            content: output.trim().to_string(),
            tokens_used: tokens.len() + tokens_generated,
            finish_reason,
        })
    }
}

#[cfg(feature = "gguf")]
pub use inner::{GgufConfig, GgufModelProvider};
