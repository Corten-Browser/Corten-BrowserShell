//! SQLite storage backend for session persistence

use crate::types::{ClosedTab, SessionState, TabState, WindowState};
use anyhow::{anyhow, Context, Result};
use rusqlite::{params, Connection, Transaction};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Session storage using SQLite
#[derive(Clone)]
pub struct SessionStorage {
    conn: Arc<Mutex<Connection>>,
}

impl SessionStorage {
    /// Create new storage with database at given path
    pub fn new(db_path: &str) -> Result<Self> {
        // Create parent directory if needed
        if let Some(parent) = Path::new(db_path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {:?}", parent))?;
            }
        }

        let conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open database: {}", db_path))?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        storage.init_schema()?;

        Ok(storage)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                is_active BOOLEAN DEFAULT 1
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS session_windows (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                window_id TEXT NOT NULL,
                x INTEGER,
                y INTEGER,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                maximized BOOLEAN DEFAULT 0,
                active_tab_index INTEGER,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS session_tabs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_window_id INTEGER NOT NULL,
                tab_id TEXT NOT NULL,
                url TEXT NOT NULL,
                title TEXT,
                position INTEGER NOT NULL,
                FOREIGN KEY (session_window_id) REFERENCES session_windows(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS closed_tabs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tab_id TEXT NOT NULL,
                url TEXT NOT NULL,
                title TEXT,
                closed_at INTEGER NOT NULL,
                window_id TEXT,
                position INTEGER
            )",
            [],
        )?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_timestamp ON sessions(timestamp DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_closed_tabs_timestamp ON closed_tabs(closed_at DESC)",
            [],
        )?;

        Ok(())
    }

    /// Save session state to database
    pub fn save_session(&self, state: &SessionState) -> Result<i64> {
        let mut conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let tx = conn.transaction()?;

        // Insert session
        tx.execute(
            "INSERT INTO sessions (timestamp, is_active) VALUES (?1, 1)",
            params![state.timestamp],
        )?;

        let session_id = tx.last_insert_rowid();

        // Insert windows and tabs
        for window in &state.windows {
            self.save_window(&tx, session_id, window)?;
        }

        tx.commit()?;

        Ok(session_id)
    }

    /// Save window within a transaction
    fn save_window(&self, tx: &Transaction, session_id: i64, window: &WindowState) -> Result<()> {
        tx.execute(
            "INSERT INTO session_windows
             (session_id, window_id, x, y, width, height, maximized, active_tab_index)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                session_id,
                &window.id,
                window.x,
                window.y,
                window.width,
                window.height,
                window.maximized,
                window.active_tab_index,
            ],
        )?;

        let window_db_id = tx.last_insert_rowid();

        // Insert tabs for this window
        for tab in &window.tabs {
            tx.execute(
                "INSERT INTO session_tabs
                 (session_window_id, tab_id, url, title, position)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![window_db_id, &tab.id, &tab.url, &tab.title, tab.position,],
            )?;
        }

        Ok(())
    }

    /// Get most recent session
    pub fn get_most_recent_session(&self) -> Result<Option<SessionState>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT id, timestamp FROM sessions
             ORDER BY timestamp DESC LIMIT 1",
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            let session_id: i64 = row.get(0)?;
            let timestamp: i64 = row.get(1)?;

            drop(rows);
            drop(stmt);

            let windows = self.load_windows(&conn, session_id)?;

            Ok(Some(SessionState { windows, timestamp }))
        } else {
            Ok(None)
        }
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: i64) -> Result<Option<SessionState>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn.prepare("SELECT timestamp FROM sessions WHERE id = ?1")?;

        let mut rows = stmt.query(params![session_id])?;

        if let Some(row) = rows.next()? {
            let timestamp: i64 = row.get(0)?;

            drop(rows);
            drop(stmt);

            let windows = self.load_windows(&conn, session_id)?;

            Ok(Some(SessionState { windows, timestamp }))
        } else {
            Ok(None)
        }
    }

    /// Load windows for a session
    fn load_windows(&self, conn: &Connection, session_id: i64) -> Result<Vec<WindowState>> {
        let mut stmt = conn.prepare(
            "SELECT id, window_id, x, y, width, height, maximized, active_tab_index
             FROM session_windows
             WHERE session_id = ?1",
        )?;

        let window_rows = stmt.query_map(params![session_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,      // id
                row.get::<_, String>(1)?,   // window_id
                row.get::<_, Option<i32>>(2)?, // x
                row.get::<_, Option<i32>>(3)?, // y
                row.get::<_, u32>(4)?,      // width
                row.get::<_, u32>(5)?,      // height
                row.get::<_, bool>(6)?,     // maximized
                row.get::<_, Option<usize>>(7)?, // active_tab_index
            ))
        })?;

        let mut windows = Vec::new();

        for window_row in window_rows {
            let (window_db_id, window_id, x, y, width, height, maximized, active_tab_index) =
                window_row?;

            let tabs = self.load_tabs(conn, window_db_id)?;

            windows.push(WindowState {
                id: window_id,
                x,
                y,
                width,
                height,
                maximized,
                tabs,
                active_tab_index,
            });
        }

        Ok(windows)
    }

    /// Load tabs for a window
    fn load_tabs(&self, conn: &Connection, window_db_id: i64) -> Result<Vec<TabState>> {
        let mut stmt = conn.prepare(
            "SELECT tab_id, url, title, position
             FROM session_tabs
             WHERE session_window_id = ?1
             ORDER BY position",
        )?;

        let tab_rows = stmt.query_map(params![window_db_id], |row| {
            Ok(TabState {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                position: row.get(3)?,
            })
        })?;

        let mut tabs = Vec::new();
        for tab_row in tab_rows {
            tabs.push(tab_row?);
        }

        Ok(tabs)
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Result<Vec<(i64, i64)>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn.prepare("SELECT id, timestamp FROM sessions ORDER BY timestamp DESC")?;

        let session_rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

        let mut sessions = Vec::new();
        for session_row in session_rows {
            sessions.push(session_row?);
        }

        Ok(sessions)
    }

    /// Delete old sessions, keeping only the most recent `keep_count`
    pub fn cleanup_old_sessions(&self, keep_count: usize) -> Result<usize> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        // Get IDs to keep
        let mut stmt = conn.prepare(
            "SELECT id FROM sessions ORDER BY timestamp DESC LIMIT ?1",
        )?;

        let ids_to_keep: Vec<i64> = stmt
            .query_map(params![keep_count], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        if ids_to_keep.is_empty() {
            return Ok(0);
        }

        // Delete old sessions
        let placeholders = ids_to_keep.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!("DELETE FROM sessions WHERE id NOT IN ({})", placeholders);

        let params: Vec<&dyn rusqlite::ToSql> =
            ids_to_keep.iter().map(|id| id as &dyn rusqlite::ToSql).collect();

        let deleted = conn.execute(&query, params.as_slice())?;

        Ok(deleted)
    }

    /// Record a closed tab
    pub fn record_closed_tab(&self, tab: &ClosedTab) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        conn.execute(
            "INSERT INTO closed_tabs (tab_id, url, title, closed_at, window_id, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &tab.id,
                &tab.url,
                &tab.title,
                tab.closed_at,
                &tab.window_id,
                tab.position,
            ],
        )?;

        Ok(())
    }

    /// Get recently closed tabs
    pub fn get_recently_closed(&self, limit: usize) -> Result<Vec<ClosedTab>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT tab_id, url, title, closed_at, window_id, position
             FROM closed_tabs
             ORDER BY closed_at DESC
             LIMIT ?1",
        )?;

        let tab_rows = stmt.query_map(params![limit], |row| {
            Ok(ClosedTab {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                closed_at: row.get(3)?,
                window_id: row.get(4)?,
                position: row.get(5)?,
            })
        })?;

        let mut tabs = Vec::new();
        for tab_row in tab_rows {
            tabs.push(tab_row?);
        }

        Ok(tabs)
    }

    /// Clear recently closed tabs
    pub fn clear_recently_closed(&self) -> Result<usize> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let deleted = conn.execute("DELETE FROM closed_tabs", [])?;

        Ok(deleted)
    }

    /// Clear all data
    pub fn clear(&self) -> Result<usize> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut total = 0;
        total += conn.execute("DELETE FROM session_tabs", [])?;
        total += conn.execute("DELETE FROM session_windows", [])?;
        total += conn.execute("DELETE FROM sessions", [])?;
        total += conn.execute("DELETE FROM closed_tabs", [])?;

        Ok(total)
    }
}
