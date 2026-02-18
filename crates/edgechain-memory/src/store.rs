use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::error::MemoryError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub key: String,
    pub value: serde_json::Value,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MemoryEntry {
    pub fn new(key: impl Into<String>, value: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key: key.into(),
            value,
            tags: vec![],
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }
}

#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn set(&self, key: &str, value: serde_json::Value) -> Result<(), MemoryError>;

    async fn get(&self, key: &str) -> Result<Option<MemoryEntry>, MemoryError>;

    async fn delete(&self, key: &str) -> Result<(), MemoryError>;

    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, MemoryError>;

    async fn search_by_tag(&self, tag: &str) -> Result<Vec<MemoryEntry>, MemoryError>;

    async fn set_tagged(
        &self,
        key: &str,
        value: serde_json::Value,
        tags: Vec<String>,
    ) -> Result<(), MemoryError>;
}
