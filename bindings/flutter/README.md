# edgechain_flutter

Flutter binding for [EdgeChain](https://github.com/edgechain-org/edgechain) — a local-first, offline-capable agent runtime for mobile and edge devices.

Run LLM agents entirely on-device. No cloud required. No data leaves the phone.

---

## Features

- **On-device inference** — GGUF models via llama.cpp (Llama 3, Mistral, Phi-3, etc.)
- **Command system** — register Dart functions as agent-callable tools
- **SQLite memory** — persistent long-term memory, session memory for ephemeral state
- **RAG** — index local notes, SQLite rows, or files; agent retrieves and cites them
- **Plugin system** — bundle commands + agents + hooks into distributable plugins
- **Privacy by design** — all data stays on device

---

## Installation

```yaml
dependencies:
  edgechain_flutter: ^0.1.0
```

---

## Quick start

```dart
import 'package:edgechain_flutter/edgechain_flutter.dart';

final edge = EdgeChainFlutter(
  modelPath: 'assets/models/llama-3.2-3b-q4_k_m.gguf',
  dbPath: 'edgechain.db',
);

// Register a command the agent can call
edge.registerCommand(Command.simple(
  name: 'search_notes',
  description: 'Search notes stored locally on device.',
  handler: (args) async {
    final results = await myNotesDb.search(args['query']);
    return {'results': results};
  },
));

// Register a named agent
edge.registerAgent(AgentConfig(
  id: 'assistant',
  systemPrompt: 'You are a helpful personal assistant with access to local notes.',
  maxSteps: 5,
  allowedCommands: ['search_notes'],
));

// Initialize the Rust runtime (call once at app startup)
await edge.init();

// Run the agent
final result = await edge.agent('assistant').run(
  'What did I discuss with Rajesh last week?',
);

print(result.text);
for (final citation in result.citations) {
  print('[${citation.sourceId}] ${citation.excerpt}');
}
```

---

## Memory

```dart
// Store a value
await edge.memory.set('user:name', 'Praveen');

// Retrieve it
final name = await edge.memory.get('user:name');

// List keys by prefix
final keys = await edge.memory.listKeys(prefix: 'user:');
```

---

## RAG — Index local data

```dart
// Index a document
await edgeRagIndex(
  id: 'note:42',
  text: 'Meeting with Rajesh. He needs 200 cement bags by Friday.',
  metadataJson: '{"type":"note","date":"2026-02-18"}',
);

// Search
final chunks = await edgeRagSearch(query: 'cement bags Rajesh', topK: 5);
```

---

## Architecture

```
Flutter App (Dart)
      │
      │  flutter_rust_bridge FFI
      ▼
EdgeChain Rust Runtime
  ├── Agent loop (ReAct: model → command → retrieve → respond)
  ├── CommandRegistry
  ├── HookRegistry (audit, logging)
  ├── MemoryStore (SQLite)
  └── LocalRetriever (vector index + embedder)
            │
            ▼
      .gguf model file
      (on-device, app sandbox)
```

---

## Platform support

| Platform | Status |
|---|---|
| Android | ✅ Supported |
| iOS | ✅ Supported |
| macOS | 🔜 Planned |
| Linux | 🔜 Planned |

---

## Generating the FFI bridge

After cloning, run codegen to produce the real FFI glue:

```bash
cargo install flutter_rust_bridge_codegen
cd bindings/flutter
flutter_rust_bridge_codegen generate
```

This replaces the stub `lib/src/frb_generated.dart` with the real generated bridge.

---

## License

MIT
