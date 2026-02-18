# Contributing to EdgeChain

Thanks for your interest in contributing. EdgeChain is a local-first agent runtime for mobile and edge devices — contributions that respect the offline-first, privacy-by-design principles are most welcome.

---

## Where to Start

Good first contributions:

- **Model providers** — add an ONNX, CoreML, or MediaPipe backend implementing `ModelProvider`
- **Retriever connectors** — SQLite row indexer, file-system crawler, Drift/Isar adapters
- **Example plugins** — CRM, inventory, field-service (see `crates/edgechain-plugin`)
- **Safety policies** — audit hooks, redaction policies, command confirmation flows
- **Flutter binding** — help wire `flutter_rust_bridge` once the Rust API stabilises

---

## Repository Layout

```
EdgeChain/
  Cargo.toml                  # Workspace root
  crates/
    edgechain-core/           # Agent loop, command registry, hooks
    edgechain-model/          # ModelProvider trait + mock
    edgechain-memory/         # Session + SQLite memory
    edgechain-rag/            # Embeddings, vector index, retriever
    edgechain-plugin/         # Plugin trait + manager
  bindings/
    flutter/                  # Dart API (flutter_rust_bridge)
  examples/
    flutter_notes_assistant/  # Reference Flutter app
  docs/
    roadmap.md
    architecture.md
```

---

## Development Setup

### Prerequisites

- Rust stable (`rustup update stable`)
- For Flutter binding: Flutter ≥ 3.10 and `flutter_rust_bridge_codegen`

### Build & check

```bash
cargo check          # fast type-check all crates
cargo build          # full build
cargo test           # run all tests
cargo clippy         # lint
```

### Run a specific crate's tests

```bash
cargo test -p edgechain-core
cargo test -p edgechain-memory
```

---

## Adding a Model Provider

1. Add a new file in `crates/edgechain-model/src/` (e.g. `onnx.rs`)
2. Implement the `ModelProvider` trait:

```rust
use async_trait::async_trait;
use edgechain_model::{ModelProvider, ModelRequest, ModelResponse, ModelError};

pub struct OnnxModelProvider { /* ... */ }

#[async_trait]
impl ModelProvider for OnnxModelProvider {
    fn name(&self) -> &str { "onnx" }
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> { todo!() }
    fn context_window(&self) -> usize { 2048 }
    fn is_available(&self) -> bool { true }
}
```

3. Export it from `crates/edgechain-model/src/lib.rs`
4. Add a test in the same file

---

## Adding a Command

Commands are the actions agents can take. Register them at runtime:

```rust
use edgechain_core::{Command, CommandCategory, FnCommandHandler};
use serde_json::json;

let cmd = Command::new(
    "search_notes",
    "Search notes stored locally on device.",
    CommandCategory::ReadOnly,
    json!({
        "type": "object",
        "properties": { "query": { "type": "string" } },
        "required": ["query"]
    }),
    FnCommandHandler(|args| async move {
        let query = args["query"].as_str().unwrap_or("");
        Ok(json!({ "results": [] }))
    }),
);
```

---

## Adding a Plugin

1. Create a new crate or module implementing `EdgePlugin`
2. Implement `manifest()` and `register()`
3. See `crates/edgechain-plugin/src/plugin.rs` for the trait definition

---

## Pull Request Guidelines

- Keep PRs focused — one feature or fix per PR
- Add or update tests for any changed behaviour
- Run `cargo clippy` and fix all warnings before submitting
- Update `docs/roadmap.md` if your PR completes a roadmap item

---

## Code Style

- Follow standard `rustfmt` formatting (`cargo fmt`)
- Prefer `thiserror` for error types, `anyhow` for ad-hoc errors
- Use `tracing` (not `println!`) for all logging
- Async code uses `tokio`; avoid blocking the executor

---

## License

By contributing, you agree your contributions will be licensed under MIT OR Apache-2.0.
