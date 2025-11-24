//! History Manager Component
//!
//! Tracks browsing history with timestamps, provides search API,
//! and manages visit frequency statistics.
//!
//! Supports private browsing mode where history is not recorded.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use shared_types::TabId;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("History entry not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, HistoryError>;

/// A single history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub visit_time: DateTime<Utc>,
    pub visit_count: u32,
}

/// History search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryQuery {
    pub text: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub max_results: Option<usize>,
}

/// History Manager for tracking browsing history
pub struct HistoryManager {
    db_path: PathBuf,
    conn: Arc<RwLock<Connection>>,
}

impl HistoryManager {
    /// Create a new HistoryManager with default database path
    pub fn new() -> Result<Self> {
        let db_path = Self::default_db_path()?;
        Self::with_path(db_path)
    }

    /// Create a new HistoryManager with custom database path
    pub fn with_path(db_path: PathBuf) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        Self::initialize_database(&conn)?;

        Ok(Self {
            db_path,
            conn: Arc::new(RwLock::new(conn)),
        })
    }

    fn default_db_path() -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "CortenBrowser", "BrowserShell")
            .ok_or_else(|| {
                HistoryError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not determine project directories",
                ))
            })?;

        Ok(dirs.data_dir().join("history.db"))
    }

    fn initialize_database(conn: &Connection) -> SqliteResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                title TEXT NOT NULL,
                visit_time TEXT NOT NULL,
                visit_count INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history_url ON history(url)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history_time ON history(visit_time)",
            [],
        )?;

        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS history_fts USING fts5(
                url, title, content=history, content_rowid=id
            )",
            [],
        )?;

        Ok(())
    }

    /// Check if history should be recorded for a given tab.
    ///
    /// Returns `false` for private/incognito tabs, `true` otherwise.
    /// This method should be called before recording any history entry.
    ///
    /// # Arguments
    /// * `_tab_id` - The ID of the tab (reserved for future use)
    /// * `is_private` - Whether the tab is in private browsing mode
    ///
    /// # Returns
    /// `true` if history should be recorded, `false` for private tabs
    pub fn should_record(&self, _tab_id: TabId, is_private: bool) -> bool {
        !is_private
    }

    /// Add a URL visit to history, respecting private browsing mode.
    ///
    /// This is the preferred method when the caller has tab context.
    /// For private tabs, this method returns Ok(0) without recording anything.
    ///
    /// # Arguments
    /// * `url` - The URL to record
    /// * `title` - The page title
    /// * `tab_id` - The ID of the tab
    /// * `is_private` - Whether the tab is in private browsing mode
    ///
    /// # Returns
    /// The history entry ID, or 0 if not recorded (private tab)
    pub async fn add_visit_with_privacy_check(
        &self,
        url: String,
        title: String,
        tab_id: TabId,
        is_private: bool,
    ) -> Result<i64> {
        if !self.should_record(tab_id, is_private) {
            return Ok(0);
        }
        self.add_visit(url, title).await
    }

    /// Add a URL visit to history
    pub async fn add_visit(&self, url: String, title: String) -> Result<i64> {
        let conn = self.conn.write().await;
        let visit_time = Utc::now().to_rfc3339();

        // Check if URL already exists
        let existing: Option<(i64, u32)> = conn
            .query_row(
                "SELECT id, visit_count FROM history WHERE url = ?",
                [&url],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        if let Some((id, count)) = existing {
            // Update existing entry
            conn.execute(
                "UPDATE history SET visit_time = ?, visit_count = ?, title = ? WHERE id = ?",
                params![visit_time, count + 1, title, id],
            )?;

            // Update FTS
            conn.execute(
                "INSERT INTO history_fts(history_fts, rowid, url, title) VALUES('delete', ?, ?, ?)",
                params![id, url, title],
            )?;
            conn.execute(
                "INSERT INTO history_fts(rowid, url, title) VALUES(?, ?, ?)",
                params![id, url, title],
            )?;

            Ok(id)
        } else {
            // Insert new entry
            conn.execute(
                "INSERT INTO history (url, title, visit_time, visit_count) VALUES (?, ?, ?, 1)",
                params![url, title, visit_time],
            )?;

            let id = conn.last_insert_rowid();

            // Add to FTS
            conn.execute(
                "INSERT INTO history_fts(rowid, url, title) VALUES(?, ?, ?)",
                params![id, url, title],
            )?;

            Ok(id)
        }
    }

    /// Search history entries
    pub async fn search(&self, query: HistoryQuery) -> Result<Vec<HistoryEntry>> {
        let conn = self.conn.read().await;
        let mut results = Vec::new();
        let limit = query.max_results.unwrap_or(100);

        let sql = if let Some(text) = &query.text {
            format!(
                "SELECT h.id, h.url, h.title, h.visit_time, h.visit_count
                 FROM history h
                 INNER JOIN history_fts ON h.id = history_fts.rowid
                 WHERE history_fts MATCH ?
                 ORDER BY h.visit_time DESC
                 LIMIT {}",
                limit
            )
        } else {
            format!(
                "SELECT id, url, title, visit_time, visit_count
                 FROM history
                 ORDER BY visit_time DESC
                 LIMIT {}",
                limit
            )
        };

        let mut stmt = conn.prepare(&sql)?;

        let map_row = |row: &rusqlite::Row| -> rusqlite::Result<HistoryEntry> {
            Ok(HistoryEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visit_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?
                    .with_timezone(&Utc),
                visit_count: row.get(4)?,
            })
        };

        if let Some(text) = &query.text {
            let rows = stmt.query_map([text], map_row)?;
            for row in rows {
                results.push(row?);
            }
        } else {
            let rows = stmt.query_map([], map_row)?;
            for row in rows {
                results.push(row?);
            }
        };

        Ok(results)
    }

    /// Get recent history entries
    pub async fn get_recent(&self, count: usize) -> Result<Vec<HistoryEntry>> {
        self.search(HistoryQuery {
            text: None,
            start_time: None,
            end_time: None,
            max_results: Some(count),
        })
        .await
    }

    /// Delete a history entry by ID
    pub async fn delete_entry(&self, id: i64) -> Result<()> {
        let conn = self.conn.write().await;

        // Get entry for FTS deletion
        let (url, title): (String, String) = conn
            .query_row(
                "SELECT url, title FROM history WHERE id = ?",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| HistoryError::NotFound)?;

        // Delete from FTS
        conn.execute(
            "INSERT INTO history_fts(history_fts, rowid, url, title) VALUES('delete', ?, ?, ?)",
            params![id, url, title],
        )?;

        // Delete from main table
        let rows_affected = conn.execute("DELETE FROM history WHERE id = ?", [id])?;

        if rows_affected == 0 {
            return Err(HistoryError::NotFound);
        }

        Ok(())
    }

    /// Clear all history
    pub async fn clear_all(&self) -> Result<()> {
        let conn = self.conn.write().await;
        conn.execute("DELETE FROM history_fts", [])?;
        conn.execute("DELETE FROM history", [])?;
        Ok(())
    }

    /// Get history statistics
    pub async fn get_stats(&self) -> Result<HistoryStats> {
        let conn = self.conn.read().await;

        let total_entries: i64 =
            conn.query_row("SELECT COUNT(*) FROM history", [], |row| row.get(0))?;

        let total_visits: i64 = conn.query_row(
            "SELECT COALESCE(SUM(visit_count), 0) FROM history",
            [],
            |row| row.get(0),
        )?;

        Ok(HistoryStats {
            total_entries: total_entries as u64,
            total_visits: total_visits as u64,
        })
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default HistoryManager")
    }
}

/// History statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStats {
    pub total_entries: u64,
    pub total_visits: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_add_visit() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_history.db");
        let manager = HistoryManager::with_path(db_path).unwrap();

        let id = manager
            .add_visit(
                "https://example.com".to_string(),
                "Example Site".to_string(),
            )
            .await
            .unwrap();

        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_search_history() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_history.db");
        let manager = HistoryManager::with_path(db_path).unwrap();

        manager
            .add_visit("https://rust-lang.org".to_string(), "Rust".to_string())
            .await
            .unwrap();
        manager
            .add_visit("https://example.com".to_string(), "Example".to_string())
            .await
            .unwrap();

        let results = manager
            .search(HistoryQuery {
                text: Some("rust".to_string()),
                start_time: None,
                end_time: None,
                max_results: Some(10),
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].url.contains("rust"));
    }

    #[tokio::test]
    async fn test_get_recent() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_history.db");
        let manager = HistoryManager::with_path(db_path).unwrap();

        manager
            .add_visit("https://example1.com".to_string(), "One".to_string())
            .await
            .unwrap();
        manager
            .add_visit("https://example2.com".to_string(), "Two".to_string())
            .await
            .unwrap();

        let recent = manager.get_recent(10).await.unwrap();
        assert_eq!(recent.len(), 2);
    }

    #[tokio::test]
    async fn test_visit_count_increments() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_history.db");
        let manager = HistoryManager::with_path(db_path).unwrap();

        manager
            .add_visit("https://example.com".to_string(), "Example".to_string())
            .await
            .unwrap();
        manager
            .add_visit("https://example.com".to_string(), "Example".to_string())
            .await
            .unwrap();
        manager
            .add_visit("https://example.com".to_string(), "Example".to_string())
            .await
            .unwrap();

        let results = manager.get_recent(10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].visit_count, 3);
    }

    #[tokio::test]
    async fn test_delete_entry() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_history.db");
        let manager = HistoryManager::with_path(db_path).unwrap();

        let id = manager
            .add_visit("https://example.com".to_string(), "Example".to_string())
            .await
            .unwrap();

        manager.delete_entry(id).await.unwrap();

        let results = manager.get_recent(10).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_clear_all() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_history.db");
        let manager = HistoryManager::with_path(db_path).unwrap();

        manager
            .add_visit("https://example1.com".to_string(), "One".to_string())
            .await
            .unwrap();
        manager
            .add_visit("https://example2.com".to_string(), "Two".to_string())
            .await
            .unwrap();

        manager.clear_all().await.unwrap();

        let stats = manager.get_stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_history.db");
        let manager = HistoryManager::with_path(db_path).unwrap();

        manager
            .add_visit("https://example.com".to_string(), "Example".to_string())
            .await
            .unwrap();
        manager
            .add_visit("https://example.com".to_string(), "Example".to_string())
            .await
            .unwrap();

        let stats = manager.get_stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.total_visits, 2);
    }

    #[test]
    fn test_should_record_regular_tab() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_history.db");
        let manager = HistoryManager::with_path(db_path).unwrap();
        let tab_id = TabId::new();

        // Regular tabs should record history
        assert!(manager.should_record(tab_id, false));
    }

    #[test]
    fn test_should_record_private_tab() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_history.db");
        let manager = HistoryManager::with_path(db_path).unwrap();
        let tab_id = TabId::new();

        // Private tabs should NOT record history
        assert!(!manager.should_record(tab_id, true));
    }

    #[tokio::test]
    async fn test_add_visit_with_privacy_check_regular() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_history.db");
        let manager = HistoryManager::with_path(db_path).unwrap();
        let tab_id = TabId::new();

        // Regular tab should record history
        let id = manager
            .add_visit_with_privacy_check(
                "https://example.com".to_string(),
                "Example".to_string(),
                tab_id,
                false,
            )
            .await
            .unwrap();

        assert!(id > 0);

        // Verify it was recorded
        let results = manager.get_recent(10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://example.com");
    }

    #[tokio::test]
    async fn test_add_visit_with_privacy_check_private() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_history.db");
        let manager = HistoryManager::with_path(db_path).unwrap();
        let tab_id = TabId::new();

        // Private tab should NOT record history
        let id = manager
            .add_visit_with_privacy_check(
                "https://secret-site.com".to_string(),
                "Secret".to_string(),
                tab_id,
                true,
            )
            .await
            .unwrap();

        // Should return 0 for skipped entry
        assert_eq!(id, 0);

        // Verify nothing was recorded
        let results = manager.get_recent(10).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_mixed_private_and_regular_visits() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_history.db");
        let manager = HistoryManager::with_path(db_path).unwrap();
        let regular_tab = TabId::new();
        let private_tab = TabId::new();

        // Add visits from both regular and private tabs
        manager
            .add_visit_with_privacy_check(
                "https://public1.com".to_string(),
                "Public 1".to_string(),
                regular_tab,
                false,
            )
            .await
            .unwrap();

        manager
            .add_visit_with_privacy_check(
                "https://private1.com".to_string(),
                "Private 1".to_string(),
                private_tab,
                true,
            )
            .await
            .unwrap();

        manager
            .add_visit_with_privacy_check(
                "https://public2.com".to_string(),
                "Public 2".to_string(),
                regular_tab,
                false,
            )
            .await
            .unwrap();

        manager
            .add_visit_with_privacy_check(
                "https://private2.com".to_string(),
                "Private 2".to_string(),
                private_tab,
                true,
            )
            .await
            .unwrap();

        // Only public visits should be recorded
        let results = manager.get_recent(10).await.unwrap();
        assert_eq!(results.len(), 2);

        let urls: Vec<&str> = results.iter().map(|r| r.url.as_str()).collect();
        assert!(urls.contains(&"https://public1.com"));
        assert!(urls.contains(&"https://public2.com"));
        assert!(!urls.contains(&"https://private1.com"));
        assert!(!urls.contains(&"https://private2.com"));
    }
}
