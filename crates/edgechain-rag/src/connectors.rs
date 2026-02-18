//! Data source connectors for bulk-indexing documents into a `Retriever`.
//!
//! # Connectors
//! - [`SqliteConnector`] — index rows from a local SQLite table
//! - [`FileConnector`]  — index text files from a directory

use std::path::{Path, PathBuf};
use crate::{document::Document, error::RagError, retriever::Retriever};

// ---------------------------------------------------------------------------
// SQLite row connector
// ---------------------------------------------------------------------------

/// Indexes rows from a SQLite table into a retriever.
///
/// Each row becomes one `Document`. You specify which column holds the text
/// to embed and which column holds the unique ID.
///
/// # Example
/// ```no_run
/// # use edgechain_rag::{LocalRetriever, StubEmbedder};
/// # use edgechain_rag::connectors::SqliteConnector;
/// let retriever = LocalRetriever::new(StubEmbedder::default());
/// // SqliteConnector::new("notes.db", "notes", "id", "body")
/// //     .index_all(&retriever).await.unwrap();
/// ```
pub struct SqliteConnector {
    db_path: PathBuf,
    table: String,
    id_column: String,
    text_column: String,
    extra_columns: Vec<String>,
    where_clause: Option<String>,
}

impl SqliteConnector {
    pub fn new(
        db_path: impl Into<PathBuf>,
        table: impl Into<String>,
        id_column: impl Into<String>,
        text_column: impl Into<String>,
    ) -> Self {
        Self {
            db_path: db_path.into(),
            table: table.into(),
            id_column: id_column.into(),
            text_column: text_column.into(),
            extra_columns: vec![],
            where_clause: None,
        }
    }

    /// Include extra columns as document metadata.
    pub fn with_metadata_columns(mut self, columns: Vec<impl Into<String>>) -> Self {
        self.extra_columns = columns.into_iter().map(|c| c.into()).collect();
        self
    }

    /// Filter rows with a SQL WHERE clause (e.g. `"deleted = 0"`).
    pub fn with_filter(mut self, where_clause: impl Into<String>) -> Self {
        self.where_clause = Some(where_clause.into());
        self
    }

    /// Index all matching rows into the retriever.
    pub async fn index_all(&self, retriever: &dyn Retriever) -> Result<usize, RagError> {
        use rusqlite::{Connection, params};

        let conn = Connection::open(&self.db_path)
            .map_err(|e| RagError::IndexError(e.to_string()))?;

        let extra = if self.extra_columns.is_empty() {
            String::new()
        } else {
            format!(", {}", self.extra_columns.join(", "))
        };

        let sql = match &self.where_clause {
            Some(w) => format!(
                "SELECT {id}, {text}{extra} FROM {table} WHERE {w}",
                id = self.id_column,
                text = self.text_column,
                table = self.table,
                w = w,
            ),
            None => format!(
                "SELECT {id}, {text}{extra} FROM {table}",
                id = self.id_column,
                text = self.text_column,
                table = self.table,
            ),
        };

        let mut stmt = conn.prepare(&sql)
            .map_err(|e| RagError::IndexError(e.to_string()))?;

        let col_count = stmt.column_count();
        let rows: Vec<(String, String, Vec<(String, String)>)> = stmt
            .query_map(params![], |row| {
                let id: String = row.get(0)?;
                let text: String = row.get(1)?;
                let mut meta = vec![];
                for (i, col_name) in self.extra_columns.iter().enumerate() {
                    if 2 + i < col_count {
                        let val: String = row.get(2 + i).unwrap_or_default();
                        meta.push((col_name.clone(), val));
                    }
                }
                Ok((id, text, meta))
            })
            .map_err(|e| RagError::IndexError(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        let count = rows.len();
        for (id, text, meta) in rows {
            if text.trim().is_empty() {
                continue;
            }
            let mut doc = Document::new(
                format!("sqlite:{}:{}", self.table, id),
                text,
            );
            for (k, v) in meta {
                doc = doc.with_metadata(k, serde_json::Value::String(v));
            }
            retriever.index_document(doc).await?;
        }

        Ok(count)
    }
}

// ---------------------------------------------------------------------------
// File connector
// ---------------------------------------------------------------------------

/// Indexes text files from a directory (or a single file) into a retriever.
///
/// Each file becomes one `Document`. The document ID is the file path.
///
/// # Example
/// ```no_run
/// # use edgechain_rag::{LocalRetriever, StubEmbedder};
/// # use edgechain_rag::connectors::FileConnector;
/// let retriever = LocalRetriever::new(StubEmbedder::default());
/// // FileConnector::new("notes/")
/// //     .with_extensions(vec!["txt", "md"])
/// //     .index_all(&retriever).await.unwrap();
/// ```
pub struct FileConnector {
    root: PathBuf,
    extensions: Option<Vec<String>>,
    recursive: bool,
    chunk_size: Option<usize>,
}

impl FileConnector {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            extensions: None,
            recursive: true,
            chunk_size: None,
        }
    }

    /// Only index files with these extensions (e.g. `["txt", "md"]`).
    pub fn with_extensions(mut self, exts: Vec<impl Into<String>>) -> Self {
        self.extensions = Some(exts.into_iter().map(|e| e.into()).collect());
        self
    }

    pub fn non_recursive(mut self) -> Self {
        self.recursive = false;
        self
    }

    /// Split large files into chunks of approximately `size` characters.
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }

    /// Index all matching files into the retriever.
    pub async fn index_all(&self, retriever: &dyn Retriever) -> Result<usize, RagError> {
        let files = self.collect_files()?;
        let mut count = 0usize;

        for path in files {
            let text = tokio::fs::read_to_string(&path).await
                .map_err(|e| RagError::IndexError(format!("{}: {e}", path.display())))?;

            if text.trim().is_empty() {
                continue;
            }

            let id_base = path.to_string_lossy().to_string();

            if let Some(chunk_size) = self.chunk_size {
                for (i, chunk) in chunk_text(&text, chunk_size).iter().enumerate() {
                    let doc = Document::new(
                        format!("file:{id_base}:chunk{i}"),
                        chunk.clone(),
                    )
                    .with_metadata("source_file", serde_json::Value::String(id_base.clone()))
                    .with_metadata("chunk_index", serde_json::Value::Number(i.into()));
                    retriever.index_document(doc).await?;
                    count += 1;
                }
            } else {
                let doc = Document::new(format!("file:{id_base}"), text)
                    .with_metadata("source_file", serde_json::Value::String(id_base));
                retriever.index_document(doc).await?;
                count += 1;
            }
        }

        Ok(count)
    }

    fn collect_files(&self) -> Result<Vec<PathBuf>, RagError> {
        let mut files = vec![];
        self.walk(&self.root, &mut files)?;
        Ok(files)
    }

    fn walk(&self, dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), RagError> {
        if dir.is_file() {
            if self.matches(dir) {
                out.push(dir.to_path_buf());
            }
            return Ok(());
        }

        let entries = std::fs::read_dir(dir)
            .map_err(|e| RagError::IndexError(e.to_string()))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() && self.recursive {
                self.walk(&path, out)?;
            } else if path.is_file() && self.matches(&path) {
                out.push(path);
            }
        }
        Ok(())
    }

    fn matches(&self, path: &Path) -> bool {
        match &self.extensions {
            None => true,
            Some(exts) => path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| exts.iter().any(|x| x == e))
                .unwrap_or(false),
        }
    }
}

/// Split text into chunks of approximately `size` characters, breaking on whitespace.
fn chunk_text(text: &str, size: usize) -> Vec<String> {
    let mut chunks = vec![];
    let mut current = String::new();

    for word in text.split_whitespace() {
        if !current.is_empty() && current.len() + word.len() + 1 > size {
            chunks.push(current.trim().to_string());
            current = String::new();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    chunks
}
