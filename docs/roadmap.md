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

## v0.1 — Foundation ✅ Released

**Theme**: Get a working agent loop running on-device with Flutter.

**Status**: Complete. All 5 crates published to crates.io. Flutter package prepared for pub.dev.

### Core Runtime (Rust)
- [x] `edgechain-core` crate with agent loop trait
- [x] Deterministic step-by-step agent loop (build prompt → model call → parse action → execute → observe → repeat)
- [x] Command dispatcher with typed JSON schema input/output
- [x] Command registry (`RwLock`-backed, register, lookup, validate)
- [x] Step limit and timeout safety guards

### Model Provider
- [x] `edgechain-model` crate with `ModelProvider` trait
- [x] `GgufModelProvider` via `llama-cpp-2` (`--features gguf`)
- [x] `MockModelProvider` for testing with scripted responses
- [x] Context window overflow guard
- [x] Configurable GPU layers, threads, context size

### Memory
- [x] `edgechain-memory` crate
- [x] `SessionMemory` — in-process `RwLock<HashMap>`, cleared on drop
- [x] `SqliteMemoryStore` — persistent SQLite via `rusqlite` (bundled)
- [x] Key-value, tag-based search, prefix listing APIs

### Hooks & Observability
- [x] `HookRegistry` with lifecycle event bus
- [x] Hook points: `BeforeModelCall`, `AfterModelCall`, `BeforeCommand`, `AfterCommand`, `MemoryWrite`, `Error`
- [ ] Built-in audit log hook (append-only SQLite log) — deferred to v0.4

### Flutter Binding
- [x] `flutter_rust_bridge` FFI scaffold (`api.rs`, `runtime.rs`)
- [x] Dart API: `EdgeChainFlutter`, `Agent`, `AgentConfig`, `Command`, `MemoryStore`
- [x] `frb_generated.dart` stub (replaced by codegen output after `flutter_rust_bridge_codegen generate`)
- [x] Android + iOS plugin stubs, podspec, build.gradle
- [ ] Async streaming result support in Dart — deferred to v0.2

### Plugin System
- [x] `edgechain-plugin` crate
- [x] `EdgePlugin` trait + `PluginManifest` + `PluginManager`
- [x] `EdgeRegistry` — commands, agents, hooks

### CI & Distribution
- [x] GitHub Actions CI (test, gguf feature check, Android cross-compile)
- [x] `CONTRIBUTING.md`, `docs/architecture.md`, `docs/roadmap.md`
- [x] All 5 crates published to crates.io (`edgechain-core`, `edgechain-model`, `edgechain-memory`, `edgechain-rag`, `edgechain-plugin`)
- [x] `edgechain_flutter` pub.dev package prepared (pending `flutter pub publish`)

### Example App
- [x] `flutter_notes_assistant` — search and summarize local notes using a GGUF model

---

## v0.2 — RAG (Retrieval-Augmented Generation) ✅ Released

**Theme**: Let agents answer questions grounded in local content with citations.

**Status**: Complete. RAG fully wired into the agent loop. Connectors shipped.

### Embeddings
- [x] `edgechain-rag` crate
- [x] `Embedder` trait with `StubEmbedder` (testing) and `GgufEmbedder` (`--features gguf`)
- [x] `GgufEmbedder` — GGUF embedding models via `llama-cpp-2`, L2-normalized, `spawn_blocking`
- [ ] ONNX embedding support — deferred to v0.3
- [ ] Persistent index (saved to disk, loaded on startup) — deferred to v0.3

### Vector Index
- [x] `VectorIndex` — flat cosine-similarity in-process index (suitable for ≤10k docs)
- [x] Similarity search with top-k results
- [ ] HNSW index for larger corpora — deferred to v0.3
- [ ] Metadata filtering on search — deferred to v0.3

### Retriever
- [x] `Retriever` trait + `LocalRetriever` (embedder + vector index)
- [x] `FileConnector` — index text/markdown files from a directory, with chunking and extension filter
- [x] `SqliteConnector` — index rows from a local SQLite table, with metadata columns and WHERE filter
- [x] `RetrievedChunk` with `id`, `text`, `score`, `metadata`

### Agent Integration
- [x] `ParsedAction::Retrieve` wired in agent loop — calls `LocalRetriever::search`
- [x] `AgentResult.citations` — populated from retrieved chunks (source ID + 200-char excerpt)
- [x] `Agent::with_retriever()` — attach a retriever to any agent
- [x] Bridge: `edge_rag_index` and `edge_rag_search` wired in `api.rs`
- [ ] Configurable retrieval strategy per agent (always, on-demand, disabled) — deferred to v0.3

### Example App
- [ ] `flutter_inventory_assistant` — query and summarize inventory records with citations

---

## v0.3 — Plugins & Skills ✅ Completed

**Theme**: Make EdgeChain extensible so teams can package and share domain-specific agents.

**Target**: A plugin can be dropped into any EdgeChain app and contribute commands, skills, and agents.

### Plugin System
- [x] `edgechain-plugin` crate
- [x] `EdgePlugin` trait: `id`, `version`, `register(EdgeRegistry)`
- [x] `EdgeRegistry` API: register commands, skills, agents, hooks
- [x] Plugin manifest (metadata: name, version, required permissions)
- [x] Plugin load/unload lifecycle

### Skills
- [x] `Skill` trait: `name`, `allowedCommands`, `systemPrompt()`
- [x] Skill-level command allowlist enforcement
- [x] Skill-level prompt injection into agent context
- [x] Skill policies: confirmation required, read-only mode, external-gated

### Agent Registry
- [x] Named agent registry (`edge.agent("inventory_assistant")`)
- [x] Agent-to-skill binding
- [x] Multi-agent routing (route user input to the right agent by intent)

### Example Plugins
- [x] `CrmPlugin` — contacts, meeting notes, follow-up tasks
- [x] `InventoryPlugin` — stock lookup, purchase orders, low-stock alerts
- [x] `FieldServicePlugin` — job reports, site checklists, offline form submission

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
- [ ] Create `github.com/edgechain-org/edgechain` (public repo) — repo initialized locally, push pending
- [x] Clean `README.md` with quick start, architecture overview, and badges
- [x] `docs/architecture.md` — Rust traits, agent loop diagram, data flow
- [x] `docs/roadmap.md` (this file)
- [x] `CONTRIBUTING.md` — how to add model providers, connectors, plugins
- [x] Working Flutter example app in `examples/flutter_notes_assistant/`
- [x] GitHub Actions CI (build + test on push, gguf feature check, Android cross-compile)
- [ ] GitHub Releases with changelogs per version — pending first push

#### crates.io (Rust Registry)
- [x] Publish `edgechain-core` v0.1.0
- [x] Publish `edgechain-model` v0.1.0
- [x] Publish `edgechain-rag` v0.1.0
- [x] Publish `edgechain-memory` v0.1.0
- [x] Publish `edgechain-plugin` v0.1.0
- [x] Each crate: proper `Cargo.toml` metadata (description, readme, license, repository, homepage, keywords)
- [x] Per-crate `README.md` displayed on crates.io
- [ ] Docs auto-published to `docs.rs` — happens automatically after push

#### pub.dev (Flutter/Dart Registry)
- [x] `edgechain_flutter` package fully prepared (README, CHANGELOG, LICENSE, pubspec metadata, topics)
- [x] Android plugin stub (Kotlin + build.gradle)
- [x] iOS plugin stub (Swift + podspec)
- [x] Example app with `pubspec.yaml` for pub.dev example tab
- [ ] Run `flutter_rust_bridge_codegen generate` to replace stub `frb_generated.dart`
- [ ] Run `flutter pub publish` — requires Flutter SDK
- [ ] Dart API docs passing `dart doc`
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

*Last updated: 22 February 2026 — v0.1 + v0.2 complete, 5 crates live on crates.io*
