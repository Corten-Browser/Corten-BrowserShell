use crate::types::{Download, DownloadStatus};
use anyhow::{anyhow, Result};
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::sync::Arc;

/// SQLite-based storage for download metadata
#[derive(Clone)]
pub struct DownloadStorage {
    conn: Arc<Mutex<Connection>>,
}

impl DownloadStorage {
    /// Create new storage with database at given path
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Create schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS downloads (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                file_name TEXT NOT NULL,
                save_path TEXT NOT NULL,
                mime_type TEXT,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                completed_at INTEGER
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_downloads_created_at ON downloads(created_at DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_downloads_status ON downloads(status)",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Insert a new download
    pub fn insert(&self, download: &Download) -> Result<()> {
        let conn = self.conn.lock();
        let status_json = serde_json::to_string(&download.status)?;

        conn.execute(
            "INSERT INTO downloads (id, url, file_name, save_path, mime_type, status, created_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                download.id,
                download.url,
                download.file_name,
                download.save_path,
                download.mime_type,
                status_json,
                download.created_at,
                download.completed_at,
            ],
        )?;

        Ok(())
    }

    /// Get download by ID
    pub fn get(&self, id: &str) -> Result<Option<Download>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, url, file_name, save_path, mime_type, status, created_at, completed_at
             FROM downloads WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id], |row| {
            let status_json: String = row.get(5)?;
            let status: DownloadStatus = serde_json::from_str(&status_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            Ok(Download {
                id: row.get(0)?,
                url: row.get(1)?,
                file_name: row.get(2)?,
                save_path: row.get(3)?,
                mime_type: row.get(4)?,
                status,
                created_at: row.get(6)?,
                completed_at: row.get(7)?,
            })
        });

        match result {
            Ok(download) => Ok(Some(download)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Update existing download
    pub fn update(&self, download: &Download) -> Result<()> {
        let conn = self.conn.lock();
        let status_json = serde_json::to_string(&download.status)?;

        let rows_updated = conn.execute(
            "UPDATE downloads SET url = ?2, file_name = ?3, save_path = ?4, mime_type = ?5,
             status = ?6, created_at = ?7, completed_at = ?8 WHERE id = ?1",
            params![
                download.id,
                download.url,
                download.file_name,
                download.save_path,
                download.mime_type,
                status_json,
                download.created_at,
                download.completed_at,
            ],
        )?;

        if rows_updated == 0 {
            return Err(anyhow!("Download not found: {}", download.id));
        }

        Ok(())
    }

    /// Delete download by ID
    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM downloads WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// List all downloads
    pub fn list_all(&self) -> Result<Vec<Download>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, url, file_name, save_path, mime_type, status, created_at, completed_at
             FROM downloads ORDER BY created_at DESC",
        )?;

        let downloads = stmt.query_map([], |row| {
            let status_json: String = row.get(5)?;
            let status: DownloadStatus = serde_json::from_str(&status_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            Ok(Download {
                id: row.get(0)?,
                url: row.get(1)?,
                file_name: row.get(2)?,
                save_path: row.get(3)?,
                mime_type: row.get(4)?,
                status,
                created_at: row.get(6)?,
                completed_at: row.get(7)?,
            })
        })?;

        let mut result = Vec::new();
        for download in downloads {
            result.push(download?);
        }

        Ok(result)
    }

    /// List downloads by status prefix (e.g., "Pending", "Completed")
    pub fn list_by_status_prefix(&self, prefix: &str) -> Result<Vec<Download>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, url, file_name, save_path, mime_type, status, created_at, completed_at
             FROM downloads WHERE status LIKE ?1 ORDER BY created_at DESC",
        )?;

        let pattern = format!("%\"{}%", prefix);
        let downloads = stmt.query_map(params![pattern], |row| {
            let status_json: String = row.get(5)?;
            let status: DownloadStatus = serde_json::from_str(&status_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            Ok(Download {
                id: row.get(0)?,
                url: row.get(1)?,
                file_name: row.get(2)?,
                save_path: row.get(3)?,
                mime_type: row.get(4)?,
                status,
                created_at: row.get(6)?,
                completed_at: row.get(7)?,
            })
        })?;

        let mut result = Vec::new();
        for download in downloads {
            result.push(download?);
        }

        Ok(result)
    }

    /// Clear all completed downloads
    pub fn clear_completed(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM downloads WHERE status LIKE '%\"Completed%'",
            [],
        )?;
        Ok(())
    }
}
