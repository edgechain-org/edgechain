/// Global EdgeChain runtime singleton.
/// Initialized once via `edge_init` and reused across all Dart calls.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use anyhow::Result;
use tokio::runtime::Runtime as TokioRuntime;

use edgechain_core::EdgeChain;
use edgechain_memory::{SqliteMemoryStore, SessionMemory};
use edgechain_model::MockModelProvider;
use edgechain_rag::{LocalRetriever, Retriever, StubEmbedder};

/// Dart-registered command handler: takes JSON args string, returns JSON result string.
pub type DartCommandHandler = Box<dyn Fn(String) -> String + Send + Sync>;

pub struct BridgeState {
    pub runtime: EdgeChain,
    pub retriever: Arc<LocalRetriever>,
    pub tokio: Arc<TokioRuntime>,
    pub dart_handlers: RwLock<HashMap<String, Arc<DartCommandHandler>>>,
}

static STATE: OnceLock<BridgeState> = OnceLock::new();

pub fn state() -> &'static BridgeState {
    STATE.get().expect("EdgeChain bridge not initialized — call edge_init() first")
}

/// Initialize the global runtime. Called once from Dart at app startup.
pub fn init(db_path: Option<String>) -> Result<()> {
    if STATE.get().is_some() {
        return Ok(()); // already initialized
    }

    let tokio = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("edgechain-worker")
            .enable_all()
            .build()?,
    );

    let model = MockModelProvider::new("mock-v0.1");

    let edge = match db_path {
        Some(path) => EdgeChain::builder(model)
            .with_memory(SqliteMemoryStore::open(&path)?)
            .build(),
        None => EdgeChain::builder(model)
            .with_memory(SessionMemory::new())
            .build(),
    };

    let retriever = Arc::new(LocalRetriever::new(StubEmbedder::default()));

    let state = BridgeState {
        runtime: edge,
        retriever,
        tokio,
        dart_handlers: RwLock::new(HashMap::new()),
    };

    STATE.set(state).map_err(|_| anyhow::anyhow!("EdgeChain already initialized"))?;
    Ok(())
}
