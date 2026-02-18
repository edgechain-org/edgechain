use async_trait::async_trait;
use tracing::debug;
use crate::{
    error::ModelError,
    provider::{FinishReason, ModelProvider, ModelRequest, ModelResponse},
};

/// A mock model provider for testing and development.
/// Returns scripted or echo responses without requiring a real model file.
pub struct MockModelProvider {
    name: String,
    scripted_responses: Vec<String>,
    call_count: std::sync::atomic::AtomicUsize,
}

impl MockModelProvider {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            scripted_responses: vec![],
            call_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Pre-load responses that will be returned in order, cycling when exhausted.
    pub fn with_responses(mut self, responses: Vec<impl Into<String>>) -> Self {
        self.scripted_responses = responses.into_iter().map(|r| r.into()).collect();
        self
    }
}

#[async_trait]
impl ModelProvider for MockModelProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let count = self.call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let content = if self.scripted_responses.is_empty() {
            let last = request.messages.last().map(|m| m.content.as_str()).unwrap_or("");
            format!("[MockModel] Echo: {last}")
        } else {
            let idx = count % self.scripted_responses.len();
            self.scripted_responses[idx].clone()
        };

        debug!(provider = %self.name, call = count, "MockModelProvider::complete");

        Ok(ModelResponse {
            tokens_used: content.split_whitespace().count(),
            finish_reason: FinishReason::Stop,
            content,
        })
    }

    fn context_window(&self) -> usize {
        4096
    }

    fn is_available(&self) -> bool {
        true
    }
}
