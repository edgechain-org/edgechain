# edgechain-core

Core agent loop, command registry, and hook system for the [EdgeChain](https://github.com/edgechain-org/edgechain) SDK.

## What's in this crate

- **`Agent`** — deterministic ReAct-style loop: model call → parse action → execute command or retrieve → repeat
- **`CommandRegistry`** — thread-safe (`RwLock`) registry of callable actions
- **`HookRegistry`** — lifecycle event bus (`BeforeModelCall`, `AfterCommand`, `MemoryWrite`, `Error`, …)
- **`AgentContext`** — conversation history + memory integration
- **`EdgeChain`** — top-level runtime builder

## Quick start

```toml
[dependencies]
edgechain-core = "0.1"
edgechain-model = "0.1"
edgechain-memory = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde_json = "1"
```

```rust
use edgechain_core::{EdgeChain, Command, CommandCategory, FnCommandHandler};
use edgechain_model::MockModelProvider;
use serde_json::json;

#[tokio::main]
async fn main() {
    let edge = EdgeChain::builder(MockModelProvider::new("mock"))
        .with_command(Command::new(
            "get_weather",
            "Get current weather for a city.",
            CommandCategory::ReadOnly,
            json!({ "type": "object", "properties": { "city": { "type": "string" } } }),
            FnCommandHandler(|args: serde_json::Value| async move {
                Ok(json!({ "city": args["city"], "condition": "sunny" }))
            }),
        ))
        .build();

    let result = edge.agent("assistant").unwrap()
        .run("What is the weather in London?")
        .await
        .unwrap();

    println!("{}", result.text);
}
```

## Feature flags

| Flag | Description |
|---|---|
| *(none)* | Default build — no native dependencies |

## License

MIT OR Apache-2.0
