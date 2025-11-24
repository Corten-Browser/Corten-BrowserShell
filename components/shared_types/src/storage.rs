//! SQLite-based storage layer for persistent data
//!
//! This module provides a generic storage layer using SQLite with:
//! - Typed key-value storage
//! - JSON serialization for structured data
//! - Migration system for schema updates
//! - Thread-safe concurrent access
//! - Platform-specific data directory resolution

use crate::StorageError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "storage")]
use directories::ProjectDirs;
#[cfg(feature = "storage")]
use rusqlite::{params, Connection, OptionalExtension};
#[cfg(feature = "storage")]
use std::path::PathBuf;
#[cfg(feature = "storage")]
use std::sync::Mutex;

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Metadata about a stored value
#[derive(Debug, Clone)]
pub struct StorageMetadata {
    /// When the value was created
    pub created_at: DateTime<Utc>,
    /// When the value was last updated
    pub updated_at: DateTime<Utc>,
    /// Optional time-to-live (expiration)
    pub expires_at: Option<DateTime<Utc>>,
    /// Type hint for the stored value
    pub value_type: String,
}

/// A migration defines a schema change
#[derive(Debug, Clone)]
pub struct Migration {
    /// Unique version number (must be monotonically increasing)
    pub version: u32,
    /// Description of what this migration does
    pub description: String,
    /// SQL statements to execute for this migration
    pub up_sql: Vec<String>,
}

impl Migration {
    /// Create a new migration
    pub fn new(version: u32, description: impl Into<String>, up_sql: Vec<String>) -> Self {
        Self {
            version,
            description: description.into(),
            up_sql,
        }
    }
}

/// Trait defining the storage interface
///
/// This trait provides async-compatible methods for storing and retrieving
/// typed data. Implementations handle serialization and persistence.
#[async_trait]
pub trait Storage: Send + Sync {
    /// Get a value by key, deserializing from JSON
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> StorageResult<Option<T>>;

    /// Set a value by key, serializing to JSON
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> StorageResult<()>;

    /// Set a value with expiration time
    async fn set_with_expiry<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        expires_at: DateTime<Utc>,
    ) -> StorageResult<()>;

    /// Delete a key
    async fn delete(&self, key: &str) -> StorageResult<bool>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> StorageResult<bool>;

    /// List all keys matching a prefix
    async fn list_keys(&self, prefix: &str) -> StorageResult<Vec<String>>;

    /// Get metadata for a key
    async fn get_metadata(&self, key: &str) -> StorageResult<Option<StorageMetadata>>;

    /// Clear all data (use with caution!)
    async fn clear(&self) -> StorageResult<()>;

    /// Get multiple values at once
    async fn get_many<T: DeserializeOwned + Send>(
        &self,
        keys: &[&str],
    ) -> StorageResult<HashMap<String, T>>;

    /// Set multiple values at once
    async fn set_many<T: Serialize + Send + Sync>(
        &self,
        items: &HashMap<String, T>,
    ) -> StorageResult<()>;

    /// Delete expired entries
    async fn cleanup_expired(&self) -> StorageResult<u64>;
}

/// SQLite-based storage implementation
///
/// Provides thread-safe storage using SQLite with automatic schema migrations.
/// Uses `std::sync::Mutex` for thread-safe access to the connection.
#[cfg(feature = "storage")]
pub struct SqliteStorage {
    /// Database connection wrapped for thread-safe access
    conn: Arc<Mutex<Connection>>,
    /// Path to the database file
    db_path: PathBuf,
    /// Namespace for this storage instance (allows multiple stores in one DB)
    namespace: String,
}

#[cfg(feature = "storage")]
impl SqliteStorage {
    /// Create a new SqliteStorage with default data directory
    ///
    /// Uses platform-specific directories:
    /// - Linux: ~/.local/share/CortenBrowser/BrowserShell/
    /// - macOS: ~/Library/Application Support/com.CortenBrowser.BrowserShell/
    /// - Windows: C:\Users\<User>\AppData\Roaming\CortenBrowser\BrowserShell\
    pub fn new(db_name: &str, namespace: &str) -> StorageResult<Self> {
        let db_path = Self::default_data_dir()?.join(format!("{}.db", db_name));
        Self::with_path(db_path, namespace)
    }

    /// Create a new SqliteStorage with a specific path
    pub fn with_path(db_path: PathBuf, namespace: &str) -> StorageResult<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                StorageError::InitializationFailed(format!(
                    "Failed to create data directory: {}",
                    e
                ))
            })?;
        }

        let conn =
            Connection::open(&db_path).map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
            namespace: namespace.to_string(),
        };

        // Run schema initialization
        storage.initialize_schema()?;

        Ok(storage)
    }

    /// Get the default data directory for the application
    pub fn default_data_dir() -> StorageResult<PathBuf> {
        ProjectDirs::from("com", "CortenBrowser", "BrowserShell")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .ok_or_else(|| {
                StorageError::InitializationFailed(
                    "Could not determine platform data directory".to_string(),
                )
            })
    }

    /// Initialize the database schema
    fn initialize_schema(&self) -> StorageResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::InitializationFailed(format!("Could not acquire lock: {}", e))
        })?;

        // Create the migrations table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                applied_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // Create the main key-value store table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv_store (
                namespace TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                value_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                expires_at TEXT,
                PRIMARY KEY (namespace, key)
            )",
            [],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // Create indexes for efficient lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_kv_namespace ON kv_store(namespace)",
            [],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_kv_expires ON kv_store(expires_at) WHERE expires_at IS NOT NULL",
            [],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Run migrations to update the schema
    pub fn run_migrations(&self, migrations: &[Migration]) -> StorageResult<u32> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;
        let mut applied = 0;

        // Get current version
        let current_version: u32 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM _migrations",
                [],
                |row| row.get(0),
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // Apply pending migrations in order
        let mut pending: Vec<_> = migrations
            .iter()
            .filter(|m| m.version > current_version)
            .collect();
        pending.sort_by_key(|m| m.version);

        for migration in pending {
            // Start a transaction for each migration
            conn.execute("BEGIN TRANSACTION", [])
                .map_err(|e| StorageError::TransactionError(e.to_string()))?;

            // Execute all SQL statements for this migration
            for sql in &migration.up_sql {
                conn.execute(sql, []).map_err(|e| {
                    // Rollback on error
                    let _ = conn.execute("ROLLBACK", []);
                    StorageError::MigrationError(format!(
                        "Migration {} failed: {}",
                        migration.version, e
                    ))
                })?;
            }

            // Record the migration
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO _migrations (version, description, applied_at) VALUES (?, ?, ?)",
                params![migration.version, migration.description, now],
            )
            .map_err(|e| {
                let _ = conn.execute("ROLLBACK", []);
                StorageError::MigrationError(format!(
                    "Failed to record migration {}: {}",
                    migration.version, e
                ))
            })?;

            conn.execute("COMMIT", [])
                .map_err(|e| StorageError::TransactionError(e.to_string()))?;

            applied += 1;
        }

        Ok(applied)
    }

    /// Get the current schema version
    pub fn schema_version(&self) -> StorageResult<u32> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;
        conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _migrations",
            [],
            |row| row.get(0),
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))
    }

    /// Get the database file path
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Get the namespace
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Execute a raw SQL query (for advanced use cases)
    pub fn execute_raw(&self, sql: &str) -> StorageResult<usize> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;
        conn.execute(sql, [])
            .map_err(|e| StorageError::DatabaseError(e.to_string()))
    }
}

#[cfg(feature = "storage")]
#[async_trait]
impl Storage for SqliteStorage {
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> StorageResult<Option<T>> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;
        let now = Utc::now().to_rfc3339();

        let result: Option<String> = conn
            .query_row(
                "SELECT value FROM kv_store
                 WHERE namespace = ? AND key = ?
                 AND (expires_at IS NULL OR expires_at > ?)",
                params![self.namespace, key, now],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        match result {
            Some(json) => {
                let value: T = serde_json::from_str(&json)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> StorageResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;
        let json = serde_json::to_string(value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let now = Utc::now().to_rfc3339();
        let value_type = std::any::type_name::<T>().to_string();

        conn.execute(
            "INSERT INTO kv_store (namespace, key, value, value_type, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(namespace, key) DO UPDATE SET
                value = excluded.value,
                value_type = excluded.value_type,
                updated_at = excluded.updated_at,
                expires_at = NULL",
            params![self.namespace, key, json, value_type, now, now],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn set_with_expiry<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        expires_at: DateTime<Utc>,
    ) -> StorageResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;
        let json = serde_json::to_string(value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let now = Utc::now().to_rfc3339();
        let expires = expires_at.to_rfc3339();
        let value_type = std::any::type_name::<T>().to_string();

        conn.execute(
            "INSERT INTO kv_store (namespace, key, value, value_type, created_at, updated_at, expires_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(namespace, key) DO UPDATE SET
                value = excluded.value,
                value_type = excluded.value_type,
                updated_at = excluded.updated_at,
                expires_at = excluded.expires_at",
            params![self.namespace, key, json, value_type, now, now, expires],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<bool> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;
        let rows = conn
            .execute(
                "DELETE FROM kv_store WHERE namespace = ? AND key = ?",
                params![self.namespace, key],
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows > 0)
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;
        let now = Utc::now().to_rfc3339();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM kv_store
                 WHERE namespace = ? AND key = ?
                 AND (expires_at IS NULL OR expires_at > ?)",
                params![self.namespace, key, now],
                |row| row.get(0),
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(count > 0)
    }

    async fn list_keys(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;
        let now = Utc::now().to_rfc3339();
        let pattern = format!("{}%", prefix);

        let mut stmt = conn
            .prepare(
                "SELECT key FROM kv_store
                 WHERE namespace = ? AND key LIKE ?
                 AND (expires_at IS NULL OR expires_at > ?)
                 ORDER BY key",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let keys = stmt
            .query_map(params![self.namespace, pattern, now], |row| row.get(0))
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(keys)
    }

    async fn get_metadata(&self, key: &str) -> StorageResult<Option<StorageMetadata>> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;

        let result: Option<(String, String, Option<String>, String)> = conn
            .query_row(
                "SELECT created_at, updated_at, expires_at, value_type
                 FROM kv_store WHERE namespace = ? AND key = ?",
                params![self.namespace, key],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        match result {
            Some((created_at, updated_at, expires_at, value_type)) => {
                let created = DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?
                    .with_timezone(&Utc);
                let updated = DateTime::parse_from_rfc3339(&updated_at)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?
                    .with_timezone(&Utc);
                let expires = expires_at
                    .map(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)))
                    .transpose()
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;

                Ok(Some(StorageMetadata {
                    created_at: created,
                    updated_at: updated,
                    expires_at: expires,
                    value_type,
                }))
            }
            None => Ok(None),
        }
    }

    async fn clear(&self) -> StorageResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;
        conn.execute(
            "DELETE FROM kv_store WHERE namespace = ?",
            params![self.namespace],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_many<T: DeserializeOwned + Send>(
        &self,
        keys: &[&str],
    ) -> StorageResult<HashMap<String, T>> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;
        let now = Utc::now().to_rfc3339();
        let mut result = HashMap::new();

        for key in keys {
            let value: Option<String> = conn
                .query_row(
                    "SELECT value FROM kv_store
                     WHERE namespace = ? AND key = ?
                     AND (expires_at IS NULL OR expires_at > ?)",
                    params![self.namespace, *key, now],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

            if let Some(json) = value {
                let parsed: T = serde_json::from_str(&json)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                result.insert((*key).to_string(), parsed);
            }
        }

        Ok(result)
    }

    async fn set_many<T: Serialize + Send + Sync>(
        &self,
        items: &HashMap<String, T>,
    ) -> StorageResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;
        let now = Utc::now().to_rfc3339();
        let value_type = std::any::type_name::<T>().to_string();

        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| StorageError::TransactionError(e.to_string()))?;

        for (key, value) in items {
            let json = serde_json::to_string(value)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;

            if let Err(e) = conn.execute(
                "INSERT INTO kv_store (namespace, key, value, value_type, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?)
                 ON CONFLICT(namespace, key) DO UPDATE SET
                    value = excluded.value,
                    value_type = excluded.value_type,
                    updated_at = excluded.updated_at,
                    expires_at = NULL",
                params![self.namespace, key, json, value_type, now, now],
            ) {
                let _ = conn.execute("ROLLBACK", []);
                return Err(StorageError::DatabaseError(e.to_string()));
            }
        }

        conn.execute("COMMIT", [])
            .map_err(|e| StorageError::TransactionError(e.to_string()))?;

        Ok(())
    }

    async fn cleanup_expired(&self) -> StorageResult<u64> {
        let conn = self.conn.lock().map_err(|e| {
            StorageError::DatabaseError(format!("Could not acquire lock: {}", e))
        })?;
        let now = Utc::now().to_rfc3339();

        let rows = conn
            .execute(
                "DELETE FROM kv_store WHERE namespace = ? AND expires_at IS NOT NULL AND expires_at <= ?",
                params![self.namespace, now],
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows as u64)
    }
}

/// In-memory storage implementation for testing
///
/// Uses a HashMap internally, useful for unit tests that don't
/// need persistence.
pub struct InMemoryStorage {
    data: Arc<RwLock<HashMap<String, StoredValue>>>,
    namespace: String,
}

struct StoredValue {
    json: String,
    value_type: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

impl InMemoryStorage {
    /// Create a new in-memory storage
    pub fn new(namespace: &str) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            namespace: namespace.to_string(),
        }
    }

    fn make_key(&self, key: &str) -> String {
        format!("{}:{}", self.namespace, key)
    }
}

#[async_trait]
impl Storage for InMemoryStorage {
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> StorageResult<Option<T>> {
        let data = self.data.read().await;
        let full_key = self.make_key(key);
        let now = Utc::now();

        match data.get(&full_key) {
            Some(stored) => {
                // Check expiration
                if let Some(expires) = stored.expires_at {
                    if expires <= now {
                        return Ok(None);
                    }
                }
                let value: T = serde_json::from_str(&stored.json)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> StorageResult<()> {
        let mut data = self.data.write().await;
        let full_key = self.make_key(key);
        let json = serde_json::to_string(value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        let now = Utc::now();

        let stored = data.entry(full_key).or_insert_with(|| StoredValue {
            json: String::new(),
            value_type: String::new(),
            created_at: now,
            updated_at: now,
            expires_at: None,
        });

        stored.json = json;
        stored.value_type = std::any::type_name::<T>().to_string();
        stored.updated_at = now;
        stored.expires_at = None;

        Ok(())
    }

    async fn set_with_expiry<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        expires_at: DateTime<Utc>,
    ) -> StorageResult<()> {
        let mut data = self.data.write().await;
        let full_key = self.make_key(key);
        let json = serde_json::to_string(value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        let now = Utc::now();

        let stored = data.entry(full_key).or_insert_with(|| StoredValue {
            json: String::new(),
            value_type: String::new(),
            created_at: now,
            updated_at: now,
            expires_at: None,
        });

        stored.json = json;
        stored.value_type = std::any::type_name::<T>().to_string();
        stored.updated_at = now;
        stored.expires_at = Some(expires_at);

        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<bool> {
        let mut data = self.data.write().await;
        let full_key = self.make_key(key);
        Ok(data.remove(&full_key).is_some())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let data = self.data.read().await;
        let full_key = self.make_key(key);
        let now = Utc::now();

        match data.get(&full_key) {
            Some(stored) => {
                if let Some(expires) = stored.expires_at {
                    Ok(expires > now)
                } else {
                    Ok(true)
                }
            }
            None => Ok(false),
        }
    }

    async fn list_keys(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let data = self.data.read().await;
        let full_prefix = self.make_key(prefix);
        let namespace_prefix = self.make_key("");
        let now = Utc::now();

        let mut keys: Vec<String> = data
            .iter()
            .filter(|(k, v)| {
                k.starts_with(&full_prefix) && v.expires_at.map(|e| e > now).unwrap_or(true)
            })
            .map(|(k, _)| k.strip_prefix(&namespace_prefix).unwrap_or(k).to_string())
            .collect();

        keys.sort();
        Ok(keys)
    }

    async fn get_metadata(&self, key: &str) -> StorageResult<Option<StorageMetadata>> {
        let data = self.data.read().await;
        let full_key = self.make_key(key);

        match data.get(&full_key) {
            Some(stored) => Ok(Some(StorageMetadata {
                created_at: stored.created_at,
                updated_at: stored.updated_at,
                expires_at: stored.expires_at,
                value_type: stored.value_type.clone(),
            })),
            None => Ok(None),
        }
    }

    async fn clear(&self) -> StorageResult<()> {
        let mut data = self.data.write().await;
        let prefix = self.make_key("");
        data.retain(|k, _| !k.starts_with(&prefix));
        Ok(())
    }

    async fn get_many<T: DeserializeOwned + Send>(
        &self,
        keys: &[&str],
    ) -> StorageResult<HashMap<String, T>> {
        let data = self.data.read().await;
        let now = Utc::now();
        let mut result = HashMap::new();

        for key in keys {
            let full_key = self.make_key(key);
            if let Some(stored) = data.get(&full_key) {
                // Skip expired
                if stored.expires_at.map(|e| e <= now).unwrap_or(false) {
                    continue;
                }
                let value: T = serde_json::from_str(&stored.json)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                result.insert((*key).to_string(), value);
            }
        }

        Ok(result)
    }

    async fn set_many<T: Serialize + Send + Sync>(
        &self,
        items: &HashMap<String, T>,
    ) -> StorageResult<()> {
        for (key, value) in items {
            self.set(key, value).await?;
        }
        Ok(())
    }

    async fn cleanup_expired(&self) -> StorageResult<u64> {
        let mut data = self.data.write().await;
        let now = Utc::now();
        let prefix = self.make_key("");
        let initial_len = data.len();

        data.retain(|k, v| {
            if !k.starts_with(&prefix) {
                return true;
            }
            v.expires_at.map(|e| e > now).unwrap_or(true)
        });

        Ok((initial_len - data.len()) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        name: String,
        value: i32,
    }

    // InMemoryStorage tests (always available)
    #[tokio::test]
    async fn test_in_memory_get_set() {
        let storage = InMemoryStorage::new("test");

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        storage.set("key1", &data).await.unwrap();
        let retrieved: Option<TestData> = storage.get("key1").await.unwrap();

        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn test_in_memory_get_nonexistent() {
        let storage = InMemoryStorage::new("test");
        let result: Option<TestData> = storage.get("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_delete() {
        let storage = InMemoryStorage::new("test");

        storage.set("key1", &"value1").await.unwrap();
        assert!(storage.exists("key1").await.unwrap());

        let deleted = storage.delete("key1").await.unwrap();
        assert!(deleted);
        assert!(!storage.exists("key1").await.unwrap());

        let deleted_again = storage.delete("key1").await.unwrap();
        assert!(!deleted_again);
    }

    #[tokio::test]
    async fn test_in_memory_list_keys() {
        let storage = InMemoryStorage::new("test");

        storage.set("user:1", &"Alice").await.unwrap();
        storage.set("user:2", &"Bob").await.unwrap();
        storage.set("config:debug", &true).await.unwrap();

        let user_keys = storage.list_keys("user:").await.unwrap();
        assert_eq!(user_keys.len(), 2);
        assert!(user_keys.contains(&"user:1".to_string()));
        assert!(user_keys.contains(&"user:2".to_string()));

        let config_keys = storage.list_keys("config:").await.unwrap();
        assert_eq!(config_keys.len(), 1);
    }

    #[tokio::test]
    async fn test_in_memory_expiry() {
        let storage = InMemoryStorage::new("test");

        // Set with expiry in the past
        let past = Utc::now() - chrono::Duration::seconds(10);
        storage
            .set_with_expiry("expired_key", &"value", past)
            .await
            .unwrap();

        // Should not be retrievable
        let result: Option<String> = storage.get("expired_key").await.unwrap();
        assert!(result.is_none());

        // Set with expiry in the future
        let future = Utc::now() + chrono::Duration::seconds(3600);
        storage
            .set_with_expiry("valid_key", &"value", future)
            .await
            .unwrap();

        // Should be retrievable
        let result: Option<String> = storage.get("valid_key").await.unwrap();
        assert_eq!(result, Some("value".to_string()));
    }

    #[tokio::test]
    async fn test_in_memory_cleanup_expired() {
        let storage = InMemoryStorage::new("test");

        let past = Utc::now() - chrono::Duration::seconds(10);
        let future = Utc::now() + chrono::Duration::seconds(3600);

        storage
            .set_with_expiry("expired1", &"value1", past)
            .await
            .unwrap();
        storage
            .set_with_expiry("expired2", &"value2", past)
            .await
            .unwrap();
        storage
            .set_with_expiry("valid", &"value3", future)
            .await
            .unwrap();
        storage.set("permanent", &"value4").await.unwrap();

        let cleaned = storage.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 2);

        assert!(!storage.exists("expired1").await.unwrap());
        assert!(!storage.exists("expired2").await.unwrap());
        assert!(storage.exists("valid").await.unwrap());
        assert!(storage.exists("permanent").await.unwrap());
    }

    #[tokio::test]
    async fn test_in_memory_get_metadata() {
        let storage = InMemoryStorage::new("test");

        storage.set("key1", &"value1").await.unwrap();

        let metadata = storage.get_metadata("key1").await.unwrap();
        assert!(metadata.is_some());

        let meta = metadata.unwrap();
        assert!(meta.value_type.contains("str"));
        assert!(meta.expires_at.is_none());
        assert!(meta.created_at <= Utc::now());
        assert!(meta.updated_at <= Utc::now());
    }

    #[tokio::test]
    async fn test_in_memory_clear() {
        let storage = InMemoryStorage::new("test");

        storage.set("key1", &"value1").await.unwrap();
        storage.set("key2", &"value2").await.unwrap();

        storage.clear().await.unwrap();

        assert!(!storage.exists("key1").await.unwrap());
        assert!(!storage.exists("key2").await.unwrap());
    }

    #[tokio::test]
    async fn test_in_memory_get_set_many() {
        let storage = InMemoryStorage::new("test");

        let mut items = HashMap::new();
        items.insert(
            "key1".to_string(),
            TestData {
                name: "one".to_string(),
                value: 1,
            },
        );
        items.insert(
            "key2".to_string(),
            TestData {
                name: "two".to_string(),
                value: 2,
            },
        );

        storage.set_many(&items).await.unwrap();

        let retrieved: HashMap<String, TestData> =
            storage.get_many(&["key1", "key2", "key3"]).await.unwrap();

        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved.get("key1").unwrap().value, 1);
        assert_eq!(retrieved.get("key2").unwrap().value, 2);
    }

    #[tokio::test]
    async fn test_in_memory_namespace_isolation() {
        let storage1 = InMemoryStorage::new("ns1");
        let storage2 = InMemoryStorage::new("ns2");

        storage1.set("key", &"value1").await.unwrap();
        storage2.set("key", &"value2").await.unwrap();

        let v1: Option<String> = storage1.get("key").await.unwrap();
        let v2: Option<String> = storage2.get("key").await.unwrap();

        assert_eq!(v1, Some("value1".to_string()));
        assert_eq!(v2, Some("value2".to_string()));
    }

    // SQLite storage tests (only when "storage" feature is enabled)
    #[cfg(feature = "storage")]
    mod sqlite_tests {
        use super::*;
        use tempfile::TempDir;

        fn create_temp_storage(namespace: &str) -> (SqliteStorage, TempDir) {
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let storage = SqliteStorage::with_path(db_path, namespace).unwrap();
            (storage, temp_dir)
        }

        #[tokio::test]
        async fn test_sqlite_get_set() {
            let (storage, _temp) = create_temp_storage("test");

            let data = TestData {
                name: "test".to_string(),
                value: 42,
            };

            storage.set("key1", &data).await.unwrap();
            let retrieved: Option<TestData> = storage.get("key1").await.unwrap();

            assert_eq!(retrieved, Some(data));
        }

        #[tokio::test]
        async fn test_sqlite_persistence() {
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");

            // Create storage, write data, and drop it
            {
                let storage = SqliteStorage::with_path(db_path.clone(), "test").unwrap();
                storage
                    .set("persistent_key", &"persistent_value")
                    .await
                    .unwrap();
            }

            // Create new storage instance with same path
            {
                let storage = SqliteStorage::with_path(db_path, "test").unwrap();
                let value: Option<String> = storage.get("persistent_key").await.unwrap();
                assert_eq!(value, Some("persistent_value".to_string()));
            }
        }

        #[tokio::test]
        async fn test_sqlite_migrations() {
            let (storage, _temp) = create_temp_storage("test");

            let migrations = vec![
                Migration::new(
                    1,
                    "Create users table",
                    vec!["CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)".to_string()],
                ),
                Migration::new(
                    2,
                    "Add email column",
                    vec!["ALTER TABLE users ADD COLUMN email TEXT".to_string()],
                ),
            ];

            let applied = storage.run_migrations(&migrations).unwrap();
            assert_eq!(applied, 2);

            let version = storage.schema_version().unwrap();
            assert_eq!(version, 2);

            // Running migrations again should apply 0
            let applied_again = storage.run_migrations(&migrations).unwrap();
            assert_eq!(applied_again, 0);
        }

        #[tokio::test]
        async fn test_sqlite_expiry() {
            let (storage, _temp) = create_temp_storage("test");

            let past = Utc::now() - chrono::Duration::seconds(10);
            storage
                .set_with_expiry("expired_key", &"value", past)
                .await
                .unwrap();

            let result: Option<String> = storage.get("expired_key").await.unwrap();
            assert!(result.is_none());

            let future = Utc::now() + chrono::Duration::seconds(3600);
            storage
                .set_with_expiry("valid_key", &"value", future)
                .await
                .unwrap();

            let result: Option<String> = storage.get("valid_key").await.unwrap();
            assert_eq!(result, Some("value".to_string()));
        }

        #[tokio::test]
        async fn test_sqlite_namespace_isolation() {
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");

            let storage1 = SqliteStorage::with_path(db_path.clone(), "ns1").unwrap();
            let storage2 = SqliteStorage::with_path(db_path, "ns2").unwrap();

            storage1.set("key", &"value1").await.unwrap();
            storage2.set("key", &"value2").await.unwrap();

            let v1: Option<String> = storage1.get("key").await.unwrap();
            let v2: Option<String> = storage2.get("key").await.unwrap();

            assert_eq!(v1, Some("value1".to_string()));
            assert_eq!(v2, Some("value2".to_string()));
        }

        #[tokio::test]
        async fn test_sqlite_list_keys() {
            let (storage, _temp) = create_temp_storage("test");

            storage.set("user:1", &"Alice").await.unwrap();
            storage.set("user:2", &"Bob").await.unwrap();
            storage.set("config:debug", &true).await.unwrap();

            let user_keys = storage.list_keys("user:").await.unwrap();
            assert_eq!(user_keys.len(), 2);
        }

        #[tokio::test]
        async fn test_sqlite_cleanup_expired() {
            let (storage, _temp) = create_temp_storage("test");

            let past = Utc::now() - chrono::Duration::seconds(10);
            let future = Utc::now() + chrono::Duration::seconds(3600);

            storage
                .set_with_expiry("expired1", &"v1", past)
                .await
                .unwrap();
            storage
                .set_with_expiry("expired2", &"v2", past)
                .await
                .unwrap();
            storage
                .set_with_expiry("valid", &"v3", future)
                .await
                .unwrap();

            let cleaned = storage.cleanup_expired().await.unwrap();
            assert_eq!(cleaned, 2);
        }

        #[tokio::test]
        async fn test_sqlite_get_set_many() {
            let (storage, _temp) = create_temp_storage("test");

            let mut items = HashMap::new();
            items.insert("key1".to_string(), 100i32);
            items.insert("key2".to_string(), 200i32);

            storage.set_many(&items).await.unwrap();

            let retrieved: HashMap<String, i32> =
                storage.get_many(&["key1", "key2", "key3"]).await.unwrap();

            assert_eq!(retrieved.len(), 2);
            assert_eq!(retrieved.get("key1"), Some(&100));
            assert_eq!(retrieved.get("key2"), Some(&200));
        }

        #[tokio::test]
        async fn test_sqlite_update_existing() {
            let (storage, _temp) = create_temp_storage("test");

            storage.set("key", &"initial").await.unwrap();
            storage.set("key", &"updated").await.unwrap();

            let value: Option<String> = storage.get("key").await.unwrap();
            assert_eq!(value, Some("updated".to_string()));

            // Should only have one entry
            let keys = storage.list_keys("").await.unwrap();
            assert_eq!(keys.len(), 1);
        }
    }
}
