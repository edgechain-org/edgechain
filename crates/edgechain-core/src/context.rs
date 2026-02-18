use std::sync::Arc;
use serde::{Deserialize, Serialize};
use edgechain_model::ModelMessage;
use edgechain_memory::MemoryStore;

/// Shared context passed through each step of the agent loop.
pub struct AgentContext {
    pub agent_id: String,
    pub session_id: String,
    pub conversation: Vec<ModelMessage>,
    pub memory: Arc<dyn MemoryStore>,
}

impl AgentContext {
    pub fn new(
        agent_id: impl Into<String>,
        session_id: impl Into<String>,
        system_prompt: impl Into<String>,
        memory: Arc<dyn MemoryStore>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            session_id: session_id.into(),
            conversation: vec![ModelMessage::system(system_prompt)],
            memory,
        }
    }

    pub fn push_user(&mut self, content: impl Into<String>) {
        self.conversation.push(ModelMessage::user(content));
    }

    pub fn push_assistant(&mut self, content: impl Into<String>) {
        self.conversation.push(ModelMessage::assistant(content));
    }

    pub fn push_tool_result(&mut self, content: impl Into<String>) {
        self.conversation.push(ModelMessage::tool(content));
    }
}

/// Parsed action the model wants to take next.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ParsedAction {
    Respond { text: String },
    CallCommand { name: String, args: serde_json::Value },
    Retrieve { query: String },
}

/// Parse the raw model output into a structured action.
/// Tries JSON first; falls back to treating the whole output as a plain response.
pub fn parse_model_output(raw: &str) -> ParsedAction {
    let trimmed = raw.trim();

    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(action) = val.get("action").and_then(|a| a.as_str()) {
            match action {
                "call_command" => {
                    if let (Some(name), Some(args)) = (
                        val.get("name").and_then(|n| n.as_str()),
                        val.get("args"),
                    ) {
                        return ParsedAction::CallCommand {
                            name: name.to_string(),
                            args: args.clone(),
                        };
                    }
                }
                "retrieve" => {
                    if let Some(query) = val.get("query").and_then(|q| q.as_str()) {
                        return ParsedAction::Retrieve { query: query.to_string() };
                    }
                }
                "respond" => {
                    if let Some(text) = val.get("text").and_then(|t| t.as_str()) {
                        return ParsedAction::Respond { text: text.to_string() };
                    }
                }
                _ => {}
            }
        }
    }

    ParsedAction::Respond { text: trimmed.to_string() }
}
