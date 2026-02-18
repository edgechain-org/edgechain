use async_trait::async_trait;
use rusqlite::{Connection, params};
use std::sync::Mutex;
use tracing::debug;
use crate::{
    error::MemoryError,
    store::{MemoryEntry, MemoryStore},
};

/// Persistent long-term memory backed by SQLite.
pub struct SqliteMemoryStore {
    conn: Mutex<Connection>,
}

impl SqliteMemoryStore {
    pub fn open(path: &str) -> Result<Self, MemoryError> {
        let conn = Connection::open(path)?;
        let store = Self { conn: Mutex::new(conn) };
        store.init_schema()?;
        Ok(store)
    }

    pub fn in_memory() -> Result<Self, MemoryError> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn: Mutex::new(conn) };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<(), MemoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memory (
                id          TEXT PRIMARY KEY,
                key         TEXT NOT NULL UNIQUE,
                value       TEXT NOT NULL,
                tags        TEXT NOT NULL DEFAULT '[]',
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_memory_key ON memory(key);",
        )?;
        Ok(())
    }

    fn row_to_entry(
        id: String,
        key: String,
        value_str: String,
        tags_str: String,
        created_at_str: String,
        updated_at_str: String,
    ) -> Result<MemoryEntry, MemoryError> {
        Ok(MemoryEntry {
            id,
            key,
            value: serde_json::from_str(&value_str)?,
            tags: serde_json::from_str(&tags_str)?,
            created_at: created_at_str.parse().unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: updated_at_str.parse().unwrap_or_else(|_| chrono::Utc::now()),
        })
    }
}

#[async_trait]
impl MemoryStore for SqliteMemoryStore {
    async fn set(&self, key: &str, value: serde_json::Value) -> Result<(), MemoryError> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        let value_str = serde_json::to_string(&value)?;
        conn.execute(
            "INSERT INTO memory (id, key, value, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, '[]', ?4, ?4)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![id, key, value_str, now],
        )?;
        debug!(key, "SqliteMemoryStore::set");
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<MemoryEntry>, MemoryError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, key, value, tags, created_at, updated_at FROM memory WHERE key = ?1",
        )?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            let entry = Self::row_to_entry(
                row.get(0)?, row.get(1)?, row.get(2)?,
                row.get(3)?, row.get(4)?, row.get(5)?,
            )?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, key: &str) -> Result<(), MemoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM memory WHERE key = ?1", params![key])?;
        Ok(())
    }

    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, MemoryError> {
        let conn = self.conn.lock().unwrap();
        let (sql, pattern): (&str, String) = match prefix {
            Some(p) => ("SELECT key FROM memory WHERE key LIKE ?1 ORDER BY key", format!("{p}%")),
            None => ("SELECT key FROM memory ORDER BY key", String::new()),
        };
        let mut stmt = conn.prepare(sql)?;
        let keys = if prefix.is_some() {
            stmt.query_map(params![pattern], |row| row.get(0))?
                .collect::<Result<Vec<String>, _>>()?
        } else {
            stmt.query_map([], |row| row.get(0))?
                .collect::<Result<Vec<String>, _>>()?
        };
        Ok(keys)
    }

    async fn search_by_tag(&self, tag: &str) -> Result<Vec<MemoryEntry>, MemoryError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, key, value, tags, created_at, updated_at FROM memory WHERE tags LIKE ?1",
        )?;
        let pattern = format!("%\"{tag}\"%" );
        let entries = stmt
            .query_map(params![pattern], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter_map(|(id, key, val, tags, ca, ua)| {
                Self::row_to_entry(id, key, val, tags, ca, ua).ok()
            })
            .collect();
        Ok(entries)
    }

    async fn set_tagged(
        &self,
        key: &str,
        value: serde_json::Value,
        tags: Vec<String>,
    ) -> Result<(), MemoryError> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        let value_str = serde_json::to_string(&value)?;
        let tags_str = serde_json::to_string(&tags)?;
        conn.execute(
            "INSERT INTO memory (id, key, value, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, tags = excluded.tags, updated_at = excluded.updated_at",
            params![id, key, value_str, tags_str, now],
        )?;
        Ok(())
    }
}
