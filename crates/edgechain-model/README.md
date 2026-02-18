# edgechain-model

Model provider trait and implementations for the [EdgeChain](https://github.com/edgechain-org/edgechain) SDK.

## What's in this crate

- **`ModelProvider`** trait — async `complete()` over any inference backend
- **`MockModelProvider`** — scripted responses for testing
- **`GgufModelProvider`** — on-device GGUF inference via llama.cpp (`--features gguf`)

## Quick start

```toml
[dependencies]
edgechain-model = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

```rust
use edgechain_model::{MockModelProvider, ModelProvider, ModelRequest, ModelMessage, Role};

#[tokio::main]
async fn main() {
    let model = MockModelProvider::new("mock")
        .with_responses(vec!["Hello from EdgeChain!"]);

    let req = ModelRequest::new(vec![
        ModelMessage { role: Role::User, content: "Hi".into() },
    ]);
    let resp = model.complete(req).await.unwrap();
    println!("{}", resp.content);
}
```

## Feature flags

| Flag | Description |
|---|---|
| `gguf` | Enable `GgufModelProvider` via `llama-cpp-2` (requires cmake) |

### GGUF usage

```toml
edgechain-model = { version = "0.1", features = ["gguf"] }
```

```rust
use edgechain_model::{GgufModelProvider, GgufConfig};

let provider = GgufModelProvider::load(
    GgufConfig::new("models/llama-3.2-3b-q4_k_m.gguf")
        .with_gpu_layers(32)
        .with_context_size(4096),
).unwrap();
```

## License

MIT OR Apache-2.0
