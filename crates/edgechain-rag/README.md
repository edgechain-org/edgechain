# edgechain-rag

Local embeddings, vector index, and retrieval-augmented generation for the [EdgeChain](https://github.com/edgechain-org/edgechain) SDK.

## What's in this crate

- **`Embedder`** trait — async text → vector embedding
- **`StubEmbedder`** — deterministic pseudo-embeddings for testing
- **`GgufEmbedder`** — GGUF embedding models via llama.cpp (`--features gguf`)
- **`VectorIndex`** — flat cosine-similarity index (suitable for ≤10k docs)
- **`LocalRetriever`** — combines embedder + index, implements `Retriever`
- **`SqliteConnector`** — bulk-index rows from a SQLite table
- **`FileConnector`** — bulk-index text/markdown files from a directory

## Quick start

```toml
[dependencies]
edgechain-rag = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

```rust
use edgechain_rag::{LocalRetriever, StubEmbedder, Document, Retriever};

#[tokio::main]
async fn main() {
    let retriever = LocalRetriever::new(StubEmbedder::default());

    retriever.index_document(
        Document::new("note:1", "Rajesh needs 200 cement bags by Friday.")
    ).await.unwrap();

    let results = retriever.search("cement order Rajesh", 3).await.unwrap();
    for r in &results {
        println!("[{}] {:.3} — {}", r.id, r.score, r.text);
    }
}
```

## Connectors

### SQLite connector

```rust
use edgechain_rag::connectors::SqliteConnector;

let n = SqliteConnector::new("app.db", "notes", "id", "body")
    .with_metadata_columns(vec!["created_at", "author"])
    .with_filter("deleted = 0")
    .index_all(&retriever).await.unwrap();
println!("Indexed {n} rows");
```

### File connector

```rust
use edgechain_rag::connectors::FileConnector;

let n = FileConnector::new("documents/")
    .with_extensions(vec!["txt", "md"])
    .with_chunk_size(512)
    .index_all(&retriever).await.unwrap();
println!("Indexed {n} chunks");
```

## Feature flags

| Flag | Description |
|---|---|
| `gguf` | Enable `GgufEmbedder` via `llama-cpp-2` (requires cmake) |

## License

MIT OR Apache-2.0
