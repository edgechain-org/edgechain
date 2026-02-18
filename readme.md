# EdgeChain
===============

**Local-first agent runtime for offline devices.**
Think: LangChain-style agents, but built for mobile and edge constraints.

EdgeChain lets developers build **on-device assistants** that:
- run with **local models** (Llama, Mistral, Qwen, Gemma, any GGUF)
- use **tools/commands** to act on **local app data**
- support **RAG** over local files and databases
- keep **memory** locally (optionally encrypted)
- work **offline** with optional **cloud fallback**

### Goal
Enable "personal assistant on your device" experiences, without shipping user data to servers by default.

## Why EdgeChain exists
------------------------

Server-first agent frameworks assume:
- stable internet
- large models
- high RAM/CPU
- no battery constraints

Mobile and edge reality:
- offline and low-connectivity
- limited RAM
- battery and thermal throttling
- strict background execution
- privacy and compliance requirements

EdgeChain is built for those constraints from day one.

---

## Key Concepts

### Agents
An **Agent** is the decision-maker. It receives user input and decides what to do:
- answer directly
- retrieve from local data
- call commands/tools
- store memory
- run multi-step loops

### Commands (Tools)
A **Command** is an executable capability exposed by your app:
- `search_notes`
- `create_task`
- `update_inventory`
- `get_order_status`

Commands are typed using JSON schema-like metadata so models can call them safely.

### Skills
A **Skill** bundles one or more commands plus prompts, policies, and routing rules.  
Example skills:
- `crm_assistant`
- `inventory_manager`
- `offline_support_agent`
- `field_report_summarizer`

### Plugins
A **Plugin** is a distributable unit that can register:
- commands
- skills
- agents
- hooks
- UI/debug panels (optional)

Plugins make EdgeChain extensible like a marketplace.

### Hooks
**Hooks** let you intercept and observe the agent lifecycle:
- before/after model call
- before/after command execution
- on memory write
- on retrieval results
- on errors and retries

Hooks enable logging, cost tracking, guardrails, analytics, and replay.

---

## High-level Architecture

- **Core Runtime (Rust)**
  - Agent loop + router
  - Tool calling / command dispatcher
  - Memory store
  - Local RAG (retriever + embeddings)
  - Plugin manager
  - Hooks + event bus

- **Model Providers**
  - `llama.cpp` GGUF provider (recommended v1)
  - Optional providers later: ONNX, CoreML, MediaPipe, custom

- **Bindings**
  - Flutter (via `flutter_rust_bridge`)
  - Android (JNI)
  - iOS (Swift FFI)
  - Desktop (Rust native)

---

## Quick Start (Conceptual)

### 1) Create a runtime

```dart
final edge = EdgeChain(
  model: LocalModel.gguf(
    path: "models/llama-3.2-3b-instruct-q4_k_m.gguf",
  ),
  memory: Memory.encrypted(
    storePath: "edgechain/memory.db",
    keyProvider: MyKeyProvider(),
  ),
  retriever: Retriever.local(
    indexPath: "edgechain/index",
  ),
);
```
### 2) Register commands

```dart
edge.registerCommand(Command(
  name: "search_notes",
  description: "Search notes stored locally on device.",
  inputSchema: {
    "type": "object",
    "properties": {
      "query": {"type": "string"},
      "limit": {"type": "integer", "default": 5}
    },
    "required": ["query"]
  },
  handler: (args) async {
    return await notesDb.search(args["query"], limit: args["limit"] ?? 5);
  },
));
```

### 3) Load a plugin (optional)

```dart
edge.loadPlugin(CrmPlugin());
```
### 4) Run an agent

```dart
final agent = edge.agent("personal_assistant");
final result = await agent.run("What did I discuss with Rajesh last week?");
print(result.text);
print(result.citations);
```
## Agent Loop (How it works)

EdgeChain runs a deterministic loop:

1. **Build prompt** (system + user + memory + retrieved context)
2. **Ask local model** for next action:
   - respond
   - call command
   - retrieve more context
3. **Execute the action**
4. **Feed observations** back into the loop
5. **Stop** on completion or step limit

This makes agents debuggable and safe for mobile.

## Built-in Features

### Local RAG

Index local content and answer with citations:

- files in app sandbox
- SQLite tables
- in-app entities

```dart
await edge.retriever.indexText(
  id: "note:123",
  text: "Met Rajesh, wants 200 cement bags next Friday...",
  metadata: {"type": "note", "createdAt": "2026-02-18"}
);

final answer = await agent.run("Summarize Rajesh requirements.");
```

### Memory

- session memory (in-memory)
- long-term memory (local DB)
- optional encryption at rest

### Safety Guardrails

- command allowlist per agent
- confirmation required for risky commands
- audit log of command calls
- redaction policy before any cloud fallback (optional)

### Offline-first

Everything works without internet:

- local inference
- local retrieval
- local command execution

Cloud can be added as an opt-in fallback.

## Plugin System

### What a plugin can contribute

- **Commands**
- **Skills**
- **Agents**
- **Hooks**
- Optional debug UI panels

### Plugin example

```dart
class InventoryPlugin extends EdgePlugin {
  @override
  String get id => "inventory";

  @override
  void register(EdgeRegistry r) {
    r.command(updateStockCommand);
    r.skill(InventorySkill());
    r.agent("inventory_assistant", InventoryAgent());
    r.hook(InventoryAuditHook());
  }
}
```

## Skills and Commands

### Skill example

```dart
class InventorySkill extends Skill {
  @override
  String get name => "inventory_manager";

  @override
  List<String> get allowedCommands => [
    "search_inventory",
    "update_stock",
    "create_purchase_order",
  ];

  @override
  String systemPrompt() => """
You are an inventory assistant.
Always confirm before updating stock.
Prefer local retrieval before asking the user questions.
""";
}
```

### Command categories (recommended)

- **Read-only**: safe to execute automatically
- **Write**: require confirmation by default
- **External**: network calls, always gated

## Hooks (Lifecycle Events)

### Hook points:

- `onBeforeModelCall(ctx)`
- `onAfterModelCall(ctx, output)`
- `onBeforeCommand(cmd, args)`
- `onAfterCommand(cmd, result)`
- `onMemoryWrite(key, value)`
- `onError(err, ctx)`

### Use hooks for:

- logs
- observability
- replay/debug
- cost stats (local vs cloud)
- policy enforcement

## Commands vs Agents vs Skills

- **Command**: an action callable by the model
- **Skill**: policies and prompt rules that control an agent's behavior
- **Agent**: executes the reasoning loop and decides what to do next
- **Plugin**: a packaging mechanism to ship commands/skills/agents/hooks

## Roadmap

### v0.1 (Foundation)

- Rust core runtime
- GGUF local model provider (llama.cpp)
- Command dispatcher
- Basic agent loop
- Memory store (SQLite)
- Hooks + audit log
- Flutter binding + example app

### v0.2 (RAG)

- Local embeddings + vector index
- Retriever with citations
- File + SQLite connectors

### v0.3 (Plugins + Skills)

- Plugin manager
- Skill policies + command gating
- Agent registry

### v0.4 (Production hardening)

- background-friendly scheduling hooks (platform dependent)
- retries, dead-letter queue for commands
- better tracing and debugging tools

## Repository Layout (suggested)

```
edgechain/
  crates/
    edgechain-core/        # runtime, agent loop, hooks, registry
    edgechain-model/       # model provider traits + llama.cpp provider
    edgechain-rag/         # embeddings + vector index + retriever
    edgechain-memory/      # memory store + encryption helpers
    edgechain-plugin/      # plugin system, manifests
  bindings/
    flutter/               # flutter_rust_bridge package
    android/               # JNI wrapper (optional early)
    ios/                   # Swift wrapper (optional early)
  examples/
    flutter_notes_assistant/
    flutter_inventory_assistant/
  docs/
    architecture.md
    safety.md
    plugin-api.md
```

## Non-goals (for clarity)

EdgeChain does not aim to:

- control the entire phone UI across apps
- bypass OS permissions
- read external app data without explicit user permission and app integration

It is an in-app agent runtime that developers embed into their applications.

## License

TBD (MIT/Apache-2.0 recommended for adoption).

## Contributing

PRs welcome. Start with:

- model providers
- retriever connectors (SQLite, file, drift, isar)
- example plugins (CRM, inventory, field-service)
- safety policies and audit tooling

