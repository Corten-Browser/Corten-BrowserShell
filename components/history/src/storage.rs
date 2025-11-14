use crate::types::{HistoryVisit, PageStats, SearchQuery, TransitionType, VisitId};
use anyhow::{Context, Result};
use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;

/// SQLite-based history storage
pub struct HistoryStorage {
    conn: Arc<Mutex<Connection>>,
}

impl HistoryStorage {
    /// Create a new history storage with the given database path
    pub async fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open database connection")?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        storage.init_schema().await?;
        Ok(storage)
    }

    /// Initialize database schema
    async fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS history_visits (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                title TEXT NOT NULL,
                visit_time INTEGER NOT NULL,
                visit_duration INTEGER,
                from_url TEXT,
                transition_type TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history_url ON history_visits(url)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history_time ON history_visits(visit_time DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history_title ON history_visits(title)",
            [],
        )?;

        Ok(())
    }

    /// Insert a visit into the database
    pub async fn insert(&mut self, visit: &HistoryVisit) -> Result<()> {
        let conn = self.conn.lock();

        conn.execute(
            "INSERT INTO history_visits (id, url, title, visit_time, visit_duration, from_url, transition_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                visit.id,
                visit.url,
                visit.title,
                visit.visit_time,
                visit.visit_duration,
                visit.from_url,
                visit.transition_type.to_str(),
            ],
        )?;

        Ok(())
    }

    /// Get a visit by ID
    pub async fn get(&self, id: &VisitId) -> Result<Option<HistoryVisit>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, url, title, visit_time, visit_duration, from_url, transition_type
             FROM history_visits WHERE id = ?1"
        )?;

        let result = stmt.query_row([id], |row| {
            Ok(HistoryVisit {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visit_time: row.get(3)?,
                visit_duration: row.get(4)?,
                from_url: row.get(5)?,
                transition_type: TransitionType::from_str(row.get::<_, String>(6)?.as_str())
                    .unwrap_or(TransitionType::Link),
            })
        });

        match result {
            Ok(visit) => Ok(Some(visit)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Delete a visit by ID
    pub async fn delete(&mut self, id: &VisitId) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM history_visits WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Get all visits for a specific URL
    pub async fn get_visits_for_url(&self, url: &str) -> Result<Vec<HistoryVisit>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, url, title, visit_time, visit_duration, from_url, transition_type
             FROM history_visits WHERE url = ?1 ORDER BY visit_time DESC"
        )?;

        let visits = stmt.query_map([url], |row| {
            Ok(HistoryVisit {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visit_time: row.get(3)?,
                visit_duration: row.get(4)?,
                from_url: row.get(5)?,
                transition_type: TransitionType::from_str(row.get::<_, String>(6)?.as_str())
                    .unwrap_or(TransitionType::Link),
            })
        })?;

        let mut result = Vec::new();
        for visit in visits {
            result.push(visit?);
        }

        Ok(result)
    }

    /// Search history based on query parameters
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<HistoryVisit>> {
        let conn = self.conn.lock();

        let mut sql = String::from(
            "SELECT id, url, title, visit_time, visit_duration, from_url, transition_type
             FROM history_visits WHERE 1=1"
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref text) = query.text {
            sql.push_str(" AND (url LIKE ?1 OR title LIKE ?1)");
            params.push(Box::new(format!("%{}%", text)));
        }

        if let Some(start_time) = query.start_time {
            let param_num = params.len() + 1;
            sql.push_str(&format!(" AND visit_time >= ?{}", param_num));
            params.push(Box::new(start_time));
        }

        if let Some(end_time) = query.end_time {
            let param_num = params.len() + 1;
            sql.push_str(&format!(" AND visit_time <= ?{}", param_num));
            params.push(Box::new(end_time));
        }

        sql.push_str(" ORDER BY visit_time DESC LIMIT ?");
        params.push(Box::new(query.limit as i64));

        let mut stmt = conn.prepare(&sql)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let visits = stmt.query_map(&param_refs[..], |row| {
            Ok(HistoryVisit {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visit_time: row.get(3)?,
                visit_duration: row.get(4)?,
                from_url: row.get(5)?,
                transition_type: TransitionType::from_str(row.get::<_, String>(6)?.as_str())
                    .unwrap_or(TransitionType::Link),
            })
        })?;

        let mut result = Vec::new();
        for visit in visits {
            result.push(visit?);
        }

        Ok(result)
    }

    /// Get most visited pages
    pub async fn get_most_visited(&self, limit: usize) -> Result<Vec<PageStats>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT url, MAX(title) as title, COUNT(*) as visit_count, MAX(visit_time) as last_visit
             FROM history_visits
             GROUP BY url
             ORDER BY visit_count DESC
             LIMIT ?1"
        )?;

        let stats = stmt.query_map([limit], |row| {
            Ok(PageStats {
                url: row.get(0)?,
                title: row.get(1)?,
                visit_count: row.get(2)?,
                last_visit: row.get(3)?,
                frecency_score: 0.0, // Will be calculated separately
            })
        })?;

        let mut result = Vec::new();
        for stat in stats {
            result.push(stat?);
        }

        Ok(result)
    }

    /// Get recent visits
    pub async fn get_recent(&self, limit: usize) -> Result<Vec<HistoryVisit>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, url, title, visit_time, visit_duration, from_url, transition_type
             FROM history_visits
             ORDER BY visit_time DESC
             LIMIT ?1"
        )?;

        let visits = stmt.query_map([limit], |row| {
            Ok(HistoryVisit {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visit_time: row.get(3)?,
                visit_duration: row.get(4)?,
                from_url: row.get(5)?,
                transition_type: TransitionType::from_str(row.get::<_, String>(6)?.as_str())
                    .unwrap_or(TransitionType::Link),
            })
        })?;

        let mut result = Vec::new();
        for visit in visits {
            result.push(visit?);
        }

        Ok(result)
    }

    /// Clear history older than the given timestamp
    pub async fn clear_older_than(&mut self, timestamp: i64) -> Result<usize> {
        let conn = self.conn.lock();
        let deleted = conn.execute("DELETE FROM history_visits WHERE visit_time < ?1", [timestamp])?;
        Ok(deleted)
    }

    /// Clear all history
    pub async fn clear_all(&mut self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM history_visits", [])?;
        Ok(())
    }

    /// Count total visits
    pub async fn count_visits(&self) -> Result<usize> {
        let conn = self.conn.lock();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM history_visits", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Count visits for a specific URL
    pub async fn count_visits_for_url(&self, url: &str) -> Result<usize> {
        let conn = self.conn.lock();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM history_visits WHERE url = ?1",
            [url],
            |row| row.get(0)
        )?;
        Ok(count as usize)
    }

    /// Update visit duration
    pub async fn update_visit_duration(&mut self, id: &VisitId, duration: i64) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE history_visits SET visit_duration = ?1 WHERE id = ?2",
            rusqlite::params![duration, id],
        )?;
        Ok(())
    }

    /// Get frecency-scored pages
    pub async fn get_frecent(&self, limit: usize, current_time: i64) -> Result<Vec<PageStats>> {
        let conn = self.conn.lock();

        // Calculate frecency scores using SQLite
        let mut stmt = conn.prepare(
            "SELECT
                url,
                MAX(title) as title,
                COUNT(*) as visit_count,
                MAX(visit_time) as last_visit,
                SUM(
                    CASE
                        WHEN visit_time > ?1 - 86400 THEN 100
                        WHEN visit_time > ?1 - 604800 THEN 70
                        WHEN visit_time > ?1 - 2592000 THEN 50
                        WHEN visit_time > ?1 - 7776000 THEN 30
                        ELSE 10
                    END
                ) as frecency_score
             FROM history_visits
             GROUP BY url
             ORDER BY frecency_score DESC
             LIMIT ?2"
        )?;

        let stats = stmt.query_map(rusqlite::params![current_time, limit], |row| {
            Ok(PageStats {
                url: row.get(0)?,
                title: row.get(1)?,
                visit_count: row.get(2)?,
                last_visit: row.get(3)?,
                frecency_score: row.get::<_, f64>(4)?,
            })
        })?;

        let mut result = Vec::new();
        for stat in stats {
            result.push(stat?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_creation() {
        let storage = HistoryStorage::new(":memory:").await;
        assert!(storage.is_ok());
    }
}
