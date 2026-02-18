# EdgeChain Roadmap

> This document tracks the planned milestones for the EdgeChain SDK — a local-first agent runtime for mobile and edge devices.

---

## Guiding Principles

- **Offline-first**: every feature must work without internet by default
- **Mobile-safe**: no unbounded memory, no blocking the main thread, battery-aware
- **Developer-friendly**: clear APIs, good error messages, debuggable agent loops
- **Privacy by design**: no data leaves the device unless the developer explicitly opts in
- **Incrementally adoptable**: each version should be usable on its own

---

## v0.1 — Foundation

**Theme**: Get a working agent loop running on-device with Flutter.

**Target**: First working demo — notes assistant on Android/iOS.

### Core Runtime (Rust)
- [ ] `edgechain-core` crate with agent loop trait
- [ ] Deterministic step-by-step agent loop (build prompt → model call → parse action → execute → observe → repeat)
- [ ] Command dispatcher with typed JSON schema input/output
- [ ] Command registry (register, lookup, validate)
- [ ] Step limit and timeout safety guards

### Model Provider
- [ ] `edgechain-model` crate with `ModelProvider` trait
- [ ] `llama.cpp` GGUF provider via `llama-cpp-rs` or direct FFI
- [ ] Streaming token output support
- [ ] Context window management (truncation strategy)

### Memory
- [ ] `edgechain-memory` crate
- [ ] Session memory (in-process, cleared on session end)
- [ ] Long-term memory store backed by SQLite
- [ ] Basic key-value and conversation history APIs

### Hooks & Observability
- [ ] `HookRegistry` with lifecycle event bus
- [ ] Hook points: `onBeforeModelCall`, `onAfterModelCall`, `onBeforeCommand`, `onAfterCommand`, `onMemoryWrite`, `onError`
- [ ] Built-in audit log hook (append-only SQLite log)

### Flutter Binding
- [ ] `flutter_rust_bridge` integration
- [ ] Dart API: `EdgeChain`, `Agent`, `Command`, `AgentResult`
- [ ] Async streaming result support in Dart

### Example App
- [ ] `flutter_notes_assistant` — search and summarize local notes using a GGUF model

---

## v0.2 — RAG (Retrieval-Augmented Generation)

**Theme**: Let agents answer questions grounded in local content with citations.

**Target**: Agents that can search and cite local files, notes, and database rows.

### Embeddings
- [ ] `edgechain-rag` crate
- [ ] Local embedding model support (GGUF embedding models or ONNX)
- [ ] Batch indexing API
- [ ] Incremental index updates (add/remove/update documents)

### Vector Index
- [ ] In-process vector store (HNSW or flat index for small corpora)
- [ ] Persistent index (saved to disk, loaded on startup)
- [ ] Similarity search with top-k and metadata filtering

### Retriever
- [ ] `Retriever` trait with pluggable backends
- [ ] File connector (index text files from app sandbox)
- [ ] SQLite connector (index rows from local database tables)
- [ ] Citation metadata attached to retrieved chunks

### Agent Integration
- [ ] Automatic retrieval step in agent loop when relevant
- [ ] `AgentResult.citations` — list of source references
- [ ] Configurable retrieval strategy per agent (always, on-demand, disabled)

### Example App
- [ ] `flutter_inventory_assistant` — query and summarize inventory records with citations

---

## v0.3 — Plugins & Skills

**Theme**: Make EdgeChain extensible so teams can package and share domain-specific agents.

**Target**: A plugin can be dropped into any EdgeChain app and contribute commands, skills, and agents.

### Plugin System
- [ ] `edgechain-plugin` crate
- [ ] `EdgePlugin` trait: `id`, `version`, `register(EdgeRegistry)`
- [ ] `EdgeRegistry` API: register commands, skills, agents, hooks
- [ ] Plugin manifest (metadata: name, version, required permissions)
- [ ] Plugin load/unload lifecycle

### Skills
- [ ] `Skill` trait: `name`, `allowedCommands`, `systemPrompt()`
- [ ] Skill-level command allowlist enforcement
- [ ] Skill-level prompt injection into agent context
- [ ] Skill policies: confirmation required, read-only mode, external-gated

### Agent Registry
- [ ] Named agent registry (`edge.agent("inventory_assistant")`)
- [ ] Agent-to-skill binding
- [ ] Multi-agent routing (route user input to the right agent by intent)

### Example Plugins
- [ ] `CrmPlugin` — contacts, meeting notes, follow-up tasks
- [ ] `InventoryPlugin` — stock lookup, purchase orders, low-stock alerts
- [ ] `FieldServicePlugin` — job reports, site checklists, offline form submission

---

## v0.4 — Production Hardening

**Theme**: Make EdgeChain reliable and observable in real production apps.

**Target**: SDK ready for beta developer adoption.

### Reliability
- [ ] Command retry logic with configurable backoff
- [ ] Dead-letter queue for failed commands (inspect and replay)
- [ ] Agent loop error recovery (continue after a failed step)
- [ ] Graceful degradation when model is unavailable

### Background Execution
- [ ] Background-friendly scheduling hooks (platform-aware)
- [ ] Android: WorkManager integration hints
- [ ] iOS: BGTask integration hints
- [ ] Thermal and battery state awareness (pause/throttle inference)

### Tracing & Debugging
- [ ] Structured trace log per agent run (full step-by-step record)
- [ ] Replay tool: re-run a recorded agent trace with a different model or prompt
- [ ] Debug panel plugin (optional Flutter widget showing live agent state)
- [ ] Export trace as JSON for offline analysis

### Security
- [ ] Memory encryption at rest (SQLite + SQLCipher or AES key wrapping)
- [ ] Redaction policy: strip PII fields before any cloud fallback
- [ ] Command permission model: declare required permissions in plugin manifest
- [ ] Sandboxed command execution (no filesystem access beyond declared paths)

---

## v0.5 — Cloud Fallback & Sync (Optional)

**Theme**: Let developers opt in to cloud capabilities without breaking offline-first guarantees.

**Target**: Hybrid apps that work offline and optionally enhance with cloud.

### Cloud Model Fallback
- [ ] `CloudModelProvider` trait (OpenAI-compatible API)
- [ ] Automatic fallback: use cloud when local model is too slow or unavailable
- [ ] Fallback policy: always local, prefer local, prefer cloud, always cloud
- [ ] Cost tracking hook (tokens used, estimated cost per run)

### Cloud Sync (Optional)
- [ ] Memory sync: push/pull long-term memory to a developer-controlled backend
- [ ] Index sync: upload embeddings to a remote vector store
- [ ] Conflict resolution strategy for offline-first sync

### Privacy Controls
- [ ] Per-field opt-in for cloud sync (annotate sensitive fields as local-only)
- [ ] Audit log of every cloud call made
- [ ] User-facing consent API (prompt user before first cloud call)

---

## v1.0 — Stable SDK

**Theme**: Stable public API, comprehensive docs, and a plugin marketplace foundation.

**Target**: Production-ready for general developer adoption.

### API Stability
- [ ] Finalize and freeze public Dart and Rust APIs
- [ ] Semantic versioning commitment
- [ ] Deprecation policy documented

### Documentation
- [ ] `architecture.md` — Rust traits, agent loop pseudocode, data flow diagrams
- [ ] `plugin-api.md` — full plugin authoring guide with examples
- [ ] `safety.md` — security model, permission system, redaction policies
- [ ] `getting-started.md` — 15-minute tutorial from zero to working agent

### Testing & Quality
- [ ] Integration test suite for agent loop, RAG, and plugin system
- [ ] Benchmark suite: inference latency, memory usage, index build time
- [ ] CI for Android, iOS, and desktop targets

### Plugin Ecosystem
- [ ] Plugin registry spec (manifest format, versioning, discovery)
- [ ] At least 3 published reference plugins (CRM, Inventory, Field Service)
- [ ] Plugin authoring template repository

---

## Future / Backlog

Ideas being tracked but not yet scheduled:

- **Multi-modal support** — image input for vision-capable GGUF models
- **Voice input/output** — Whisper for STT, local TTS integration
- **ONNX / CoreML providers** — alternative runtimes for Apple Silicon and Qualcomm NPUs
- **MediaPipe provider** — for on-device ML pipelines
- **Agent-to-agent communication** — orchestrator agents delegating to sub-agents
- **Federated memory** — share anonymized memory patterns across a fleet of devices (privacy-preserving)
- **Edge server mode** — run EdgeChain as a local HTTP server for browser or desktop clients
- **Drift / Isar connectors** — RAG over popular Flutter database packages
- **LangChain / LlamaIndex compatibility layer** — import existing chains as EdgeChain skills

---

## Distribution & Adoption Strategy

> Where you publish determines adoption quality. This section tracks the publishing and community-building plan alongside technical milestones.

### Phase 1 — Technical Credibility

*Do this before any public announcement.*

#### GitHub (Non-negotiable)
- [ ] Create `github.com/edgechain-org/edgechain` (public repo)
- [ ] Clean `README.md` with quick start, architecture overview, and badges
- [ ] `docs/architecture.md` — Rust traits, agent loop diagram, data flow
- [ ] `docs/roadmap.md` (this file)
- [ ] `CONTRIBUTING.md` — how to add model providers, connectors, plugins
- [ ] Working Flutter example app in `examples/`
- [ ] GitHub Actions CI (build + test on push)
- [ ] GitHub Releases with changelogs per version

#### crates.io (Rust Registry)
- [ ] Publish `edgechain-core`
- [ ] Publish `edgechain-model`
- [ ] Publish `edgechain-rag`
- [ ] Publish `edgechain-memory`
- [ ] Publish `edgechain-plugin`
- [ ] Each crate: proper `Cargo.toml` metadata (description, license, keywords, categories)
- [ ] Docs auto-published to `docs.rs`

#### pub.dev (Flutter/Dart Registry)
- [ ] Publish `edgechain_flutter` package
- [ ] Dart API docs complete and passing `dart doc`
- [ ] Example tab on pub.dev with minimal working snippet
- [ ] Achieve pub.dev "likes" baseline via early adopter outreach

---

### Phase 2 — Community Positioning

*After v0.1 or v0.2 is solid and demo-able.*

#### Hacker News
- [ ] Post: `Show HN: EdgeChain – Offline Agent Runtime for Mobile Apps`
- [ ] Have architecture doc and demo app ready before posting
- [ ] Engage seriously with technical feedback in comments

#### Developer Writing
- [ ] Medium or Dev.to article: *"Why I built an agent runtime for offline mobile"*
- [ ] Deep-dive post: *"How EdgeChain's agent loop works on a 3B GGUF model"*
- [ ] Comparison post: *"EdgeChain vs LangChain — different constraints, different design"*

#### Reddit
- [ ] Post in `r/rust` — focus on the Rust core and FFI design
- [ ] Post in `r/FlutterDev` — focus on the Flutter binding and mobile UX
- [ ] Post in `r/LocalLLaMA` — focus on GGUF on-device inference angle

---

### Phase 3 — Ecosystem Building

*After community traction is established.*

- [ ] Plugin contribution guide and template repository
- [ ] Skill marketplace concept doc (how plugins will be discovered and shared)
- [ ] Cloud adapter beta (opt-in, for hybrid apps)
- [ ] `edgechain.dev` documentation site (GitHub Pages or Docusaurus)
- [ ] Product Hunt launch (awareness, not primary traction driver)

---

### What NOT to Do

- **No paid tier** until the core is proven and adopted
- **No marketing** before architecture is solid and documented
- **No incomplete architecture docs** — a weak first impression kills infra projects
- **No website over-investment** early — GitHub Pages is sufficient until v1.0

---

## Release Philosophy

| Stage | Criteria |
|---|---|
| **Alpha** (v0.x) | Core features working, API may change, not for production |
| **Beta** (v0.9+) | API stabilizing, suitable for early adopters and pilot apps |
| **Stable** (v1.0) | Frozen public API, production-ready, full docs |

---

*Last updated: February 2026*
