// @implements: REQ-007
//! Settings Storage - SQLite backend for settings

use rusqlite::{Connection, params};
use std::collections::HashMap;
use super::SettingsResult;

/// SQLite storage backend for settings
pub struct SettingsStorage {
    conn: Connection,
}

impl SettingsStorage {
    /// Create a new SettingsStorage and initialize the database schema
    pub fn new(mut conn: Connection) -> SettingsResult<Self> {
        Self::init_schema(&mut conn)?;
        Ok(Self { conn })
    }

    /// Initialize the database schema
    fn init_schema(conn: &mut Connection) -> SettingsResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Create index for faster queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_settings_updated_at
            ON settings(updated_at DESC)",
            [],
        )?;

        Ok(())
    }

    /// Set a setting value (insert or update)
    pub fn set(&mut self, key: &str, value: &str) -> SettingsResult<()> {
        let now = current_timestamp();

        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at)
             VALUES (?1, ?2, ?3)",
            params![key, value, now],
        )?;

        Ok(())
    }

    /// Get a setting value
    pub fn get(&self, key: &str) -> SettingsResult<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT value FROM settings WHERE key = ?1"
        )?;

        let mut rows = stmt.query(params![key])?;

        if let Some(row) = rows.next()? {
            let value: String = row.get(0)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Delete a setting
    /// Returns true if a row was deleted, false otherwise
    pub fn delete(&mut self, key: &str) -> SettingsResult<bool> {
        let rows_affected = self.conn.execute(
            "DELETE FROM settings WHERE key = ?1",
            params![key],
        )?;

        Ok(rows_affected > 0)
    }

    /// List all settings
    pub fn list_all(&self) -> SettingsResult<HashMap<String, String>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, value FROM settings ORDER BY key ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut result = HashMap::new();
        for row in rows {
            let (key, value) = row?;
            result.insert(key, value);
        }

        Ok(result)
    }

    /// Clear all settings
    pub fn clear_all(&mut self) -> SettingsResult<()> {
        self.conn.execute("DELETE FROM settings", [])?;
        Ok(())
    }
}

/// Get current Unix timestamp in seconds
fn current_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before Unix epoch")
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_creation() {
        let conn = Connection::open_in_memory().unwrap();
        let _storage = SettingsStorage::new(conn).unwrap();
    }

    #[test]
    fn test_set_and_get() {
        let conn = Connection::open_in_memory().unwrap();
        let mut storage = SettingsStorage::new(conn).unwrap();

        storage.set("key1", "value1").unwrap();
        let value = storage.get("key1").unwrap();

        assert_eq!(value, Some("value1".to_string()));
    }

    #[test]
    fn test_delete() {
        let conn = Connection::open_in_memory().unwrap();
        let mut storage = SettingsStorage::new(conn).unwrap();

        storage.set("key1", "value1").unwrap();
        let deleted = storage.delete("key1").unwrap();

        assert!(deleted);

        let value = storage.get("key1").unwrap();
        assert_eq!(value, None);
    }
}
