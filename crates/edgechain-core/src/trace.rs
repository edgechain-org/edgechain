use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use async_trait::async_trait;
use serde_json::json;
use crate::{Hook, HookEvent};

/// A hook that exports a structured trace of the agent's run to a JSON file.
pub struct TraceExportHook {
    pub file_path: PathBuf,
    events: Mutex<Vec<serde_json::Value>>,
}

impl TraceExportHook {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: path.into(),
            events: Mutex::new(Vec::new()),
        }
    }

    /// Flush the recorded events to the file.
    pub fn flush(&self) -> Result<(), std::io::Error> {
        let events = self.events.lock().unwrap();
        let mut file = File::create(&self.file_path)?;
        let json = serde_json::to_string_pretty(&*events)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

impl Drop for TraceExportHook {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[async_trait]
impl Hook for TraceExportHook {
    fn name(&self) -> &str {
        "trace_export"
    }

    async fn on_event(&self, event: &HookEvent) {
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        let value = match event {
            HookEvent::BeforeModelCall(ctx) => json!({
                "type": "before_model_call",
                "timestamp": timestamp,
                "agent_id": ctx.agent_id,
                "step": ctx.step,
                "prompt_tokens": ctx.prompt_tokens,
            }),
            HookEvent::AfterModelCall(ctx) => json!({
                "type": "after_model_call",
                "timestamp": timestamp,
                "agent_id": ctx.agent_id,
                "step": ctx.step,
                "tokens_used": ctx.tokens_used,
                "content": ctx.content,
            }),
            HookEvent::BeforeCommand { name, args } => json!({
                "type": "before_command",
                "timestamp": timestamp,
                "command": name,
                "args": args,
            }),
            HookEvent::AfterCommand(res) => json!({
                "type": "after_command",
                "timestamp": timestamp,
                "command": res.command,
                "success": res.success,
                "output": res.output,
                "error": res.error,
            }),
            HookEvent::CommandFailedPermanent { name, args, error, attempts } => json!({
                "type": "command_failed_permanent",
                "timestamp": timestamp,
                "command": name,
                "args": args,
                "error": error,
                "attempts": attempts,
            }),
            HookEvent::MemoryWrite { key, value } => json!({
                "type": "memory_write",
                "timestamp": timestamp,
                "key": key,
                "value": value,
            }),
            HookEvent::Error(ctx) => json!({
                "type": "error",
                "timestamp": timestamp,
                "agent_id": ctx.agent_id,
                "step": ctx.step,
                "error": ctx.error,
            }),
        };

        self.events.lock().unwrap().push(value);
    }
}
