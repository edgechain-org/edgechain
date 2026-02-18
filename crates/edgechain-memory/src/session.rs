use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;
use chrono::Utc;
use crate::{
    error::MemoryError,
    store::{MemoryEntry, MemoryStore},
};

/// In-process session memory. Cleared when the session ends.
/// Thread-safe via RwLock.
pub struct SessionMemory {
    store: RwLock<HashMap<String, MemoryEntry>>,
}

impl SessionMemory {
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }

    pub fn clear(&self) {
        self.store.write().unwrap().clear();
    }

    pub fn len(&self) -> usize {
        self.store.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for SessionMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemoryStore for SessionMemory {
    async fn set(&self, key: &str, value: serde_json::Value) -> Result<(), MemoryError> {
        let mut store = self.store.write().unwrap();
        let entry = store.entry(key.to_string()).or_insert_with(|| MemoryEntry::new(key, value.clone()));
        entry.value = value;
        entry.updated_at = Utc::now();
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<MemoryEntry>, MemoryError> {
        Ok(self.store.read().unwrap().get(key).cloned())
    }

    async fn delete(&self, key: &str) -> Result<(), MemoryError> {
        self.store.write().unwrap().remove(key);
        Ok(())
    }

    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, MemoryError> {
        let store = self.store.read().unwrap();
        let keys = store
            .keys()
            .filter(|k| prefix.is_none_or(|p| k.starts_with(p)))
            .cloned()
            .collect();
        Ok(keys)
    }

    async fn search_by_tag(&self, tag: &str) -> Result<Vec<MemoryEntry>, MemoryError> {
        let store = self.store.read().unwrap();
        let results = store
            .values()
            .filter(|e| e.tags.iter().any(|t| t == tag))
            .cloned()
            .collect();
        Ok(results)
    }

    async fn set_tagged(
        &self,
        key: &str,
        value: serde_json::Value,
        tags: Vec<String>,
    ) -> Result<(), MemoryError> {
        let mut store = self.store.write().unwrap();
        let entry = store.entry(key.to_string()).or_insert_with(|| MemoryEntry::new(key, value.clone()));
        entry.value = value;
        entry.tags = tags;
        entry.updated_at = Utc::now();
        Ok(())
    }
}
