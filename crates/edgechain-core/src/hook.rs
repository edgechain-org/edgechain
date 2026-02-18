use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;
use crate::command::CommandResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCallContext {
    pub agent_id: String,
    pub step: usize,
    pub prompt_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOutputContext {
    pub agent_id: String,
    pub step: usize,
    pub tokens_used: usize,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub agent_id: String,
    pub step: usize,
    pub error: String,
}

#[derive(Debug, Clone)]
pub enum HookEvent {
    BeforeModelCall(ModelCallContext),
    AfterModelCall(ModelOutputContext),
    BeforeCommand { name: String, args: Value },
    AfterCommand(CommandResult),
    MemoryWrite { key: String, value: Value },
    Error(ErrorContext),
}

#[async_trait]
pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
    async fn on_event(&self, event: &HookEvent);
}

/// Registry that fans out events to all registered hooks.
#[derive(Default)]
pub struct HookRegistry {
    hooks: Vec<Arc<dyn Hook>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, hook: impl Hook + 'static) {
        debug!(name = %hook.name(), "Registering hook");
        self.hooks.push(Arc::new(hook));
    }

    pub async fn emit(&self, event: HookEvent) {
        for hook in &self.hooks {
            hook.on_event(&event).await;
        }
    }
}

/// A simple tracing hook that logs all events at DEBUG level.
pub struct TracingHook;

#[async_trait]
impl Hook for TracingHook {
    fn name(&self) -> &str {
        "tracing"
    }

    async fn on_event(&self, event: &HookEvent) {
        match event {
            HookEvent::BeforeModelCall(ctx) => {
                debug!(agent = %ctx.agent_id, step = ctx.step, tokens = ctx.prompt_tokens, "→ model call");
            }
            HookEvent::AfterModelCall(ctx) => {
                debug!(agent = %ctx.agent_id, step = ctx.step, tokens = ctx.tokens_used, "← model response");
            }
            HookEvent::BeforeCommand { name, args } => {
                debug!(command = %name, args = %args, "→ command");
            }
            HookEvent::AfterCommand(result) => {
                debug!(command = %result.command, success = result.success, "← command result");
            }
            HookEvent::MemoryWrite { key, .. } => {
                debug!(key = %key, "memory write");
            }
            HookEvent::Error(ctx) => {
                debug!(agent = %ctx.agent_id, step = ctx.step, error = %ctx.error, "agent error");
            }
        }
    }
}
