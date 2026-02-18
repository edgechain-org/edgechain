//! Public API exposed to Dart via flutter_rust_bridge.
//!
//! All types and functions in this file are annotated with `#[frb]` so the
//! codegen tool can generate the corresponding Dart classes and methods.
//!
//! Run `flutter_rust_bridge_codegen generate` from the `bindings/flutter`
//! directory to regenerate `frb_generated.rs` and `lib/src/frb_generated.dart`.

use flutter_rust_bridge::frb;
use crate::runtime;

// ---------------------------------------------------------------------------
// Dart-visible data types
// ---------------------------------------------------------------------------

#[frb(dart_metadata=("freezed"))]
pub struct EdgeAgentResult {
    pub text: String,
    pub steps: Vec<EdgeAgentStep>,
    pub citations: Vec<EdgeCitation>,
    pub total_tokens: u32,
}

#[frb(dart_metadata=("freezed"))]
pub struct EdgeAgentStep {
    pub step: u32,
    pub action_type: String,
    pub observation: String,
}

#[frb(dart_metadata=("freezed"))]
pub struct EdgeCitation {
    pub source_id: String,
    pub excerpt: String,
}

#[frb(dart_metadata=("freezed"))]
pub struct EdgeCommandMeta {
    pub name: String,
    pub description: String,
    pub category: String,
}

#[frb(dart_metadata=("freezed"))]
pub struct EdgeMemoryEntry {
    pub id: String,
    pub key: String,
    pub value_json: String,
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// Runtime lifecycle
// ---------------------------------------------------------------------------

/// Initialize the EdgeChain runtime. Call once at app startup.
///
/// - `db_path`: path to the SQLite memory database file.
///   Pass `None` to use in-process session memory only.
#[frb(sync)]
pub fn edge_init(db_path: Option<String>) -> anyhow::Result<()> {
    runtime::init(db_path)
}

/// Returns `true` if the runtime has been initialized.
#[frb(sync)]
pub fn edge_is_initialized() -> bool {
    // OnceLock::get returns None if not set
    std::panic::catch_unwind(|| runtime::state()).is_ok()
}

// ---------------------------------------------------------------------------
// Agent management
// ---------------------------------------------------------------------------

/// Register a named agent with a system prompt and optional command allowlist.
#[frb(sync)]
pub fn edge_register_agent(
    id: String,
    system_prompt: String,
    max_steps: u32,
    allowed_commands: Option<Vec<String>>,
) {
    let state = runtime::state();
    let mut config = edgechain_core::AgentConfig::new(&id, system_prompt)
        .with_max_steps(max_steps as usize);
    if let Some(cmds) = allowed_commands {
        config = config.with_allowed_commands(cmds);
    }
    state.runtime.register_agent(config);
}

/// Run an agent with user input. Returns the full result including steps and citations.
pub async fn edge_agent_run(agent_id: String, user_input: String) -> anyhow::Result<EdgeAgentResult> {
    let state = runtime::state();
    let agent = state.runtime.agent(&agent_id)?;
    let result = agent.run(&user_input).await?;

    Ok(EdgeAgentResult {
        text: result.text,
        total_tokens: result.total_tokens as u32,
        steps: result.steps.iter().map(|s| EdgeAgentStep {
            step: s.step as u32,
            action_type: format!("{:?}", s.action).split('{').next().unwrap_or("unknown").trim().to_lowercase(),
            observation: s.observation.clone(),
        }).collect(),
        citations: result.citations.iter().map(|c| EdgeCitation {
            source_id: c.source_id.clone(),
            excerpt: c.excerpt.clone(),
        }).collect(),
    })
}

// ---------------------------------------------------------------------------
// Command registration (Dart-side handlers)
// ---------------------------------------------------------------------------

/// Register a command whose handler lives in Dart.
///
/// When the agent calls this command, EdgeChain will invoke the Dart callback
/// registered via `edge_set_command_handler`.
///
/// `input_schema_json` is a JSON Schema string describing the command's args.
#[frb(sync)]
pub fn edge_register_command(
    name: String,
    description: String,
    category: String,
    input_schema_json: String,
) {
    use edgechain_core::{Command, CommandCategory, FnCommandHandler};
    use std::sync::Arc;

    let state = runtime::state();
    let cmd_name = name.clone();

    let category = match category.as_str() {
        "write" => CommandCategory::Write,
        "external" => CommandCategory::External,
        _ => CommandCategory::ReadOnly,
    };

    let schema: serde_json::Value = serde_json::from_str(&input_schema_json)
        .unwrap_or(serde_json::json!({}));

    // The handler looks up the Dart callback by name at call time.
    // This allows Dart to register the handler after the command is declared.
    let name_for_handler = cmd_name.clone();
    let command = Command::new(
        cmd_name,
        description,
        category,
        schema,
        FnCommandHandler(move |args: serde_json::Value| {
            let name = name_for_handler.clone();
            async move {
                let state = runtime::state();
                let handler = {
                    let guard = state.dart_handlers.read().unwrap();
                    guard.get(&name).map(|h| Arc::clone(h))
                };
                match handler {
                    Some(h) => {
                        let args_str = serde_json::to_string(&args)
                            .unwrap_or_else(|_| "{}".to_string());
                        let result_str = h(args_str);
                        serde_json::from_str(&result_str)
                            .map_err(|e| edgechain_core::CoreError::Other(e.into()))
                    }
                    None => Ok(serde_json::json!({ "error": "No Dart handler registered for this command" })),
                }
            }
        }),
    );

    state.runtime.register_command(command);
}

/// Register a Dart-side handler for a command.
/// The handler receives JSON args as a string and returns a JSON result string.
///
/// This is called from Dart after `edge_register_command`.
pub fn edge_set_command_handler(
    name: String,
    handler: impl Fn(String) -> String + Send + Sync + 'static,
) {
    let state = runtime::state();
    state.dart_handlers
        .write()
        .unwrap()
        .insert(name, std::sync::Arc::new(Box::new(handler)));
}

// ---------------------------------------------------------------------------
// Memory API
// ---------------------------------------------------------------------------

/// Store a value in long-term memory.
pub async fn edge_memory_set(key: String, value_json: String) -> anyhow::Result<()> {
    let state = runtime::state();
    let value: serde_json::Value = serde_json::from_str(&value_json)?;
    state.runtime.memory().set(&key, value).await?;
    Ok(())
}

/// Retrieve a value from memory. Returns `None` if the key doesn't exist.
pub async fn edge_memory_get(key: String) -> anyhow::Result<Option<EdgeMemoryEntry>> {
    let state = runtime::state();
    let entry = state.runtime.memory().get(&key).await?;
    Ok(entry.map(|e| EdgeMemoryEntry {
        id: e.id,
        key: e.key,
        value_json: serde_json::to_string(&e.value).unwrap_or_default(),
        tags: e.tags,
    }))
}

/// Delete a key from memory.
pub async fn edge_memory_delete(key: String) -> anyhow::Result<()> {
    let state = runtime::state();
    state.runtime.memory().delete(&key).await?;
    Ok(())
}

/// List all memory keys, optionally filtered by prefix.
pub async fn edge_memory_list_keys(prefix: Option<String>) -> anyhow::Result<Vec<String>> {
    let state = runtime::state();
    let keys = state.runtime.memory()
        .list_keys(prefix.as_deref())
        .await?;
    Ok(keys)
}

// ---------------------------------------------------------------------------
// RAG API
// ---------------------------------------------------------------------------

/// Index a document for local retrieval.
pub async fn edge_rag_index(id: String, text: String, metadata_json: Option<String>) -> anyhow::Result<()> {
    use edgechain_rag::{Document, Retriever as _};
    let state = runtime::state();

    let mut doc = Document::new(id, text);
    if let Some(json) = metadata_json {
        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(&json) {
            for (k, v) in map {
                doc = doc.with_metadata(k, v);
            }
        }
    }

    state.retriever.index_document(doc).await?;
    Ok(())
}

/// Search indexed documents. Returns JSON strings of RetrievedChunk objects.
pub async fn edge_rag_search(query: String, top_k: u32) -> anyhow::Result<Vec<String>> {
    use edgechain_rag::Retriever as _;
    let state = runtime::state();
    let chunks = state.retriever.search(&query, top_k as usize).await?;
    chunks
        .iter()
        .map(|c| serde_json::to_string(c).map_err(|e| anyhow::anyhow!(e)))
        .collect()
}
