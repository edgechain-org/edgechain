# EdgeChain Architecture

> Technical reference for the EdgeChain SDK internals. Covers Rust traits, the agent loop, data flow, and crate boundaries.

---

## Crate Dependency Graph

```
edgechain-plugin
    └── edgechain-core
            ├── edgechain-model
            └── edgechain-memory

edgechain-rag        (standalone, no core dependency)
```

Each crate has a single responsibility and can be used independently.

---

## Core Traits

### `ModelProvider` — `edgechain-model`

The single abstraction over any inference backend.

```rust
#[async_trait]
pub trait ModelProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError>;
    fn context_window(&self) -> usize;
    fn is_available(&self) -> bool;
}
```

**Implementations:**
- `MockModelProvider` — scripted responses for testing
- `GgufModelProvider` *(v0.1, planned)* — llama.cpp via FFI
- `OnnxModelProvider` *(v0.3, planned)* — ONNX Runtime
- `CoreMlModelProvider` *(v0.3, planned)* — Apple CoreML

**Request / Response:**

```
ModelRequest {
    messages: Vec<ModelMessage>,   // system + user + assistant + tool turns
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    stop_sequences: Vec<String>,
}

ModelResponse {
    content: String,
    tokens_used: usize,
    finish_reason: FinishReason,   // Stop | Length | ToolCall
}
```

---

### `MemoryStore` — `edgechain-memory`

Typed key-value store with tag-based search.

```rust
#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn set(&self, key: &str, value: Value) -> Result<(), MemoryError>;
    async fn get(&self, key: &str) -> Result<Option<MemoryEntry>, MemoryError>;
    async fn delete(&self, key: &str) -> Result<(), MemoryError>;
    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, MemoryError>;
    async fn search_by_tag(&self, tag: &str) -> Result<Vec<MemoryEntry>, MemoryError>;
    async fn set_tagged(&self, key: &str, value: Value, tags: Vec<String>) -> Result<(), MemoryError>;
}
```

**Implementations:**
- `SessionMemory` — `RwLock<HashMap>`, cleared on drop
- `SqliteMemoryStore` — persistent SQLite via `rusqlite` (bundled)

---

### `CommandHandler` — `edgechain-core`

Any callable action the model can invoke.

```rust
#[async_trait]
pub trait CommandHandler: Send + Sync {
    async fn execute(&self, args: Value) -> Result<Value, CoreError>;
}
```

Commands are wrapped in `Command` which carries typed metadata:

```rust
pub struct CommandMeta {
    pub name: String,
    pub description: String,
    pub category: CommandCategory,   // ReadOnly | Write | External
    pub input_schema: Value,         // JSON Schema for the args
}
```

The `FnCommandHandler` wrapper lets you register a plain async closure:

```rust
Command::new(
    "search_notes",
    "Search local notes.",
    CommandCategory::ReadOnly,
    json!({ "type": "object", "properties": { "query": { "type": "string" } } }),
    FnCommandHandler(|args: Value| async move {
        Ok(json!({ "results": [] }))
    }),
)
```

---

### `Hook` — `edgechain-core`

Intercept any point in the agent lifecycle.

```rust
#[async_trait]
pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
    async fn on_event(&self, event: &HookEvent);
}
```

**Hook events:**

```
HookEvent::BeforeModelCall(ModelCallContext)
HookEvent::AfterModelCall(ModelOutputContext)
HookEvent::BeforeCommand { name, args }
HookEvent::AfterCommand(CommandResult)
HookEvent::MemoryWrite { key, value }
HookEvent::Error(ErrorContext)
```

Built-in: `TracingHook` — logs all events at `DEBUG` level via `tracing`.

---

### `Embedder` — `edgechain-rag`

```rust
#[async_trait]
pub trait Embedder: Send + Sync {
    fn dimensions(&self) -> usize;
    async fn embed(&self, text: &str) -> Result<EmbeddingVector, RagError>;
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<EmbeddingVector>, RagError>;
}
```

**Implementations:**
- `StubEmbedder` — deterministic pseudo-embeddings for testing
- `GgufEmbedder` *(v0.2, planned)* — embedding models via llama.cpp
- `OnnxEmbedder` *(v0.2, planned)* — ONNX embedding models

---

### `Retriever` — `edgechain-rag`

```rust
#[async_trait]
pub trait Retriever: Send + Sync {
    async fn index_document(&self, doc: Document) -> Result<(), RagError>;
    async fn remove_document(&self, id: &str) -> Result<(), RagError>;
    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<RetrievedChunk>, RagError>;
    fn document_count(&self) -> usize;
}
```

`LocalRetriever` combines an `Embedder` with a `VectorIndex` (flat cosine similarity, suitable for ≤10k docs).

---

### `EdgePlugin` — `edgechain-plugin`

```rust
pub trait EdgePlugin: Send + Sync {
    fn manifest(&self) -> PluginManifest;
    fn register(&self, registry: &mut EdgeRegistry);
}
```

`EdgeRegistry` collects everything the plugin contributes:

```rust
pub struct EdgeRegistry {
    pub commands: Vec<Command>,
    pub agents:   Vec<AgentConfig>,
    pub hooks:    Vec<Box<dyn Hook>>,
}
```

---

## Agent Loop

The deterministic loop that runs inside `Agent::run()`:

```
┌─────────────────────────────────────────────────────────────┐
│  Input: user_input: &str                                    │
│                                                             │
│  1. Build AgentContext                                      │
│     └── system_prompt + conversation history + memory      │
│                                                             │
│  2. For step in 0..max_steps:                               │
│                                                             │
│     a. Emit HookEvent::BeforeModelCall                      │
│     b. model.complete(conversation) → raw_output           │
│     c. Emit HookEvent::AfterModelCall                       │
│                                                             │
│     d. parse_model_output(raw_output) → ParsedAction        │
│                                                             │
│        ┌── Respond { text }                                 │
│        │   └── return AgentResult ✓                         │
│        │                                                    │
│        ├── CallCommand { name, args }                       │
│        │   ├── check allowlist                              │
│        │   ├── Emit HookEvent::BeforeCommand                │
│        │   ├── commands.execute(name, args) → result        │
│        │   ├── Emit HookEvent::AfterCommand                 │
│        │   └── push tool result into conversation           │
│        │                                                    │
│        └── Retrieve { query }                               │
│            ├── retriever.search(query) → chunks  (v0.2)    │
│            └── push retrieved context into conversation     │
│                                                             │
│  3. If max_steps reached → Err(StepLimitReached)            │
└─────────────────────────────────────────────────────────────┘
```

### Model Output Format

The model is expected to respond with a JSON action object:

```json
{ "action": "respond", "text": "Here is your answer." }

{ "action": "call_command", "name": "search_notes", "args": { "query": "Rajesh" } }

{ "action": "retrieve", "query": "cement bag orders" }
```

If the model output is not valid JSON, it is treated as a plain `respond` action. This makes the loop robust to models that don't follow the format perfectly.

---

## Data Flow Diagram

```
Flutter / Dart App
        │
        │  FFI (flutter_rust_bridge)
        ▼
  EdgeChainFlutter (Dart)
        │
        │  calls
        ▼
  EdgeChain (Rust runtime)
    ├── CommandRegistry
    │       └── [Command, Command, ...]
    ├── HookRegistry
    │       └── [TracingHook, AuditHook, ...]
    ├── MemoryStore
    │       ├── SessionMemory
    │       └── SqliteMemoryStore
    └── ModelProvider
            └── GgufModelProvider (llama.cpp)
                        │
                        │  llama.cpp C FFI
                        ▼
                  .gguf model file
                  (on-device, app sandbox)
```

---

## Memory Key Conventions

Recommended key namespacing to avoid collisions:

| Prefix | Purpose |
|---|---|
| `session:<id>:<key>` | Per-session ephemeral data |
| `user:<key>` | Long-term user preferences |
| `entity:<type>:<id>` | Domain entities (contacts, orders) |
| `conv:<session_id>` | Conversation history snapshots |
| `plugin:<plugin_id>:<key>` | Plugin-private storage |

---

## Context Window Management

The agent loop currently uses a simple truncation strategy:

1. System prompt is always included (never truncated)
2. Most recent N turns are kept to fit within `model.context_window()`
3. Older turns are dropped from the front of the conversation

A smarter summarisation strategy is planned for v0.2.

---

## Error Hierarchy

```
CoreError
  ├── CommandNotFound(name)
  ├── CommandFailed { name, reason }
  ├── InvalidCommandInput(msg)
  ├── StepLimitReached(n)
  ├── Timeout(ms)
  ├── Model(ModelError)
  │       ├── NotLoaded
  │       ├── InferenceFailed
  │       └── ContextWindowExceeded
  ├── Memory(MemoryError)
  │       ├── NotFound
  │       └── Database
  └── Serialization

RagError
  ├── EmbeddingFailed
  ├── IndexError
  └── NotFound

PluginError
  ├── AlreadyRegistered
  ├── NotFound
  └── RegistrationFailed
```

---

## Thread Safety

All public types are `Send + Sync`:

- `EdgeChain` — wraps everything in `Arc<>`; safe to share across threads
- `SessionMemory` — `RwLock<HashMap>` internally
- `SqliteMemoryStore` — `Mutex<Connection>` internally
- `VectorIndex` — `RwLock<Vec>` internally
- `HookRegistry` — immutable after construction; hooks are `Arc<dyn Hook>`

---

*Last updated: February 2026*
