## 0.1.0

* Initial release.
* Dart API scaffold: `EdgeChainFlutter`, `Agent`, `AgentConfig`, `Command`, `MemoryStore`.
* `SessionMemory` — in-process Dart-side key-value store.
* `BridgeMemoryStore` — delegates to Rust SQLite store via `flutter_rust_bridge` FFI.
* `frb_generated.dart` stub — replaced by codegen output once `flutter_rust_bridge_codegen generate` is run.
* Supports Android and iOS via native Rust library compiled with Cargo.
