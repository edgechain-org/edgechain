# edgechain-memory

Session and persistent memory stores for the [EdgeChain](https://github.com/edgechain-org/edgechain) SDK.

## What's in this crate

- **`MemoryStore`** trait — async key-value store with tag-based search
- **`SessionMemory`** — in-process `RwLock<HashMap>`, cleared on drop
- **`SqliteMemoryStore`** — persistent SQLite store via `rusqlite` (bundled)

## Quick start

```toml
[dependencies]
edgechain-memory = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde_json = "1"
```

```rust
use edgechain_memory::{MemoryStore, SqliteMemoryStore};
use serde_json::json;

#[tokio::main]
async fn main() {
    let store = SqliteMemoryStore::open("app.db").unwrap();

    store.set("user:name", json!("Praveen")).await.unwrap();
    store.set_tagged("order:42", json!({ "item": "cement", "qty": 200 }),
        vec!["order".into(), "pending".into()]).await.unwrap();

    let entry = store.get("user:name").await.unwrap().unwrap();
    println!("{}", entry.value); // "Praveen"

    let orders = store.search_by_tag("pending").await.unwrap();
    println!("{} pending orders", orders.len());
}
```

## Memory key conventions

| Prefix | Purpose |
|---|---|
| `session:<id>:<key>` | Per-session ephemeral data |
| `user:<key>` | Long-term user preferences |
| `entity:<type>:<id>` | Domain entities |
| `plugin:<id>:<key>` | Plugin-private storage |

## License

MIT OR Apache-2.0
