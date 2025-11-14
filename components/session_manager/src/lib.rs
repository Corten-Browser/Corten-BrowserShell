//! Session Manager - Browser state persistence and crash recovery
//!
//! This crate provides session management functionality for the Corten Browser Shell,
//! including:
//! - Save and restore complete browser sessions
//! - Crash recovery with auto-save
//! - Recently closed tabs tracking
//! - Session import/export for backup
//!
//! # Example
//!
//! ```no_run
//! use session_manager::{SessionManager, SessionState};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create session manager
//!     let mut manager = SessionManager::new("session.db").await?;
//!
//!     // Save current session
//!     let state = SessionState::new(chrono::Utc::now().timestamp());
//!     let session_id = manager.save_session(&state).await?;
//!
//!     // Restore session
//!     if let Some(restored) = manager.restore_session().await? {
//!         println!("Restored {} windows", restored.windows.len());
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod storage;
pub mod types;
pub mod validation;

pub use types::{ClosedTab, SessionState, TabState, WindowState};

use anyhow::{Context, Result};
use storage::SessionStorage;

/// Session manager for browser state persistence
pub struct SessionManager {
    storage: SessionStorage,
    #[allow(dead_code)]
    auto_save_enabled: bool,
    max_closed_tabs: usize,
}

impl SessionManager {
    /// Create new session manager
    ///
    /// # Arguments
    /// * `db_path` - Path to SQLite database file
    ///
    /// # Example
    /// ```no_run
    /// # use session_manager::SessionManager;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let manager = SessionManager::new("session.db").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(db_path: &str) -> Result<Self> {
        validation::validate_db_path(db_path)?;

        let storage = tokio::task::spawn_blocking({
            let db_path = db_path.to_string();
            move || SessionStorage::new(&db_path)
        })
        .await
        .context("Failed to spawn storage task")??;

        Ok(Self {
            storage,
            auto_save_enabled: true,
            max_closed_tabs: 100,
        })
    }

    /// Save current session state
    ///
    /// # Arguments
    /// * `state` - Current browser session state
    ///
    /// # Returns
    /// Session ID of saved session
    ///
    /// # Example
    /// ```no_run
    /// # use session_manager::{SessionManager, SessionState};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let mut manager = SessionManager::new("session.db").await?;
    /// let state = SessionState::new(chrono::Utc::now().timestamp());
    /// let session_id = manager.save_session(&state).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_session(&mut self, state: &SessionState) -> Result<i64> {
        let state = state.clone();
        let storage = self.storage.clone();

        tokio::task::spawn_blocking(move || storage.save_session(&state))
            .await
            .context("Failed to spawn save task")?
    }

    /// Restore most recent session
    ///
    /// # Returns
    /// Session state if found, None if no sessions exist
    ///
    /// # Example
    /// ```no_run
    /// # use session_manager::SessionManager;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let manager = SessionManager::new("session.db").await?;
    /// if let Some(state) = manager.restore_session().await? {
    ///     println!("Restored {} windows", state.windows.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn restore_session(&self) -> Result<Option<SessionState>> {
        let storage = self.storage.clone();

        tokio::task::spawn_blocking(move || storage.get_most_recent_session())
            .await
            .context("Failed to spawn restore task")?
    }

    /// Get session by ID
    ///
    /// # Arguments
    /// * `session_id` - ID of session to retrieve
    ///
    /// # Returns
    /// Session state if found
    ///
    /// # Example
    /// ```no_run
    /// # use session_manager::SessionManager;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let manager = SessionManager::new("session.db").await?;
    /// let state = manager.get_session(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_session(&self, session_id: i64) -> Result<Option<SessionState>> {
        validation::validate_session_id(session_id)?;

        let storage = self.storage.clone();

        tokio::task::spawn_blocking(move || storage.get_session(session_id))
            .await
            .context("Failed to spawn get session task")?
    }

    /// List all saved sessions
    ///
    /// # Returns
    /// Vector of (session_id, timestamp) tuples
    ///
    /// # Example
    /// ```no_run
    /// # use session_manager::SessionManager;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let manager = SessionManager::new("session.db").await?;
    /// let sessions = manager.list_sessions().await?;
    /// for (id, timestamp) in sessions {
    ///     println!("Session {} at {}", id, timestamp);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_sessions(&self) -> Result<Vec<(i64, i64)>> {
        let storage = self.storage.clone();

        tokio::task::spawn_blocking(move || storage.list_sessions())
            .await
            .context("Failed to spawn list sessions task")?
    }

    /// Delete old sessions
    ///
    /// # Arguments
    /// * `keep_count` - Number of recent sessions to keep
    ///
    /// # Returns
    /// Number of sessions deleted
    ///
    /// # Example
    /// ```no_run
    /// # use session_manager::SessionManager;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let mut manager = SessionManager::new("session.db").await?;
    /// let deleted = manager.cleanup_old_sessions(10).await?;
    /// println!("Deleted {} old sessions", deleted);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cleanup_old_sessions(&mut self, keep_count: usize) -> Result<usize> {
        let storage = self.storage.clone();

        tokio::task::spawn_blocking(move || storage.cleanup_old_sessions(keep_count))
            .await
            .context("Failed to spawn cleanup task")?
    }

    /// Record a closed tab
    ///
    /// # Arguments
    /// * `tab` - Closed tab information
    ///
    /// # Example
    /// ```no_run
    /// # use session_manager::{SessionManager, ClosedTab};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let mut manager = SessionManager::new("session.db").await?;
    /// let tab = ClosedTab::new(
    ///     "tab-1".to_string(),
    ///     "https://example.com".to_string(),
    ///     "Example".to_string(),
    ///     chrono::Utc::now().timestamp()
    /// );
    /// manager.record_closed_tab(tab).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn record_closed_tab(&mut self, tab: ClosedTab) -> Result<()> {
        validation::validate_url(&tab.url)?;

        let storage = self.storage.clone();

        tokio::task::spawn_blocking(move || storage.record_closed_tab(&tab))
            .await
            .context("Failed to spawn record closed tab task")?
    }

    /// Get recently closed tabs
    ///
    /// # Arguments
    /// * `limit` - Maximum number of tabs to return
    ///
    /// # Returns
    /// Vector of recently closed tabs (most recent first)
    ///
    /// # Example
    /// ```no_run
    /// # use session_manager::SessionManager;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let manager = SessionManager::new("session.db").await?;
    /// let closed_tabs = manager.get_recently_closed(10).await?;
    /// for tab in closed_tabs {
    ///     println!("Recently closed: {}", tab.title);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_recently_closed(&self, limit: usize) -> Result<Vec<ClosedTab>> {
        let actual_limit = std::cmp::min(limit, self.max_closed_tabs);
        let storage = self.storage.clone();

        tokio::task::spawn_blocking(move || storage.get_recently_closed(actual_limit))
            .await
            .context("Failed to spawn get recently closed task")?
    }

    /// Clear recently closed tabs
    ///
    /// # Returns
    /// Number of tabs cleared
    ///
    /// # Example
    /// ```no_run
    /// # use session_manager::SessionManager;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let mut manager = SessionManager::new("session.db").await?;
    /// let cleared = manager.clear_recently_closed().await?;
    /// println!("Cleared {} tabs", cleared);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn clear_recently_closed(&mut self) -> Result<usize> {
        let storage = self.storage.clone();

        tokio::task::spawn_blocking(move || storage.clear_recently_closed())
            .await
            .context("Failed to spawn clear recently closed task")?
    }

    /// Export session to JSON
    ///
    /// # Arguments
    /// * `session_id` - ID of session to export, or None for most recent
    ///
    /// # Returns
    /// JSON string representation of session
    ///
    /// # Example
    /// ```no_run
    /// # use session_manager::SessionManager;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let manager = SessionManager::new("session.db").await?;
    /// let json = manager.export_session(Some(123)).await?;
    /// std::fs::write("session_backup.json", json)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn export_session(&self, session_id: Option<i64>) -> Result<String> {
        let state = if let Some(id) = session_id {
            validation::validate_session_id(id)?;
            self.get_session(id).await?
        } else {
            self.restore_session().await?
        };

        match state {
            Some(s) => serde_json::to_string_pretty(&s).context("Failed to serialize session"),
            None => Err(anyhow::anyhow!("Session not found")),
        }
    }

    /// Import session from JSON
    ///
    /// # Arguments
    /// * `json` - JSON string representation of session
    ///
    /// # Returns
    /// Session ID of imported session
    ///
    /// # Example
    /// ```no_run
    /// # use session_manager::SessionManager;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let mut manager = SessionManager::new("session.db").await?;
    /// let json = std::fs::read_to_string("session_backup.json")?;
    /// let session_id = manager.import_session(&json).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn import_session(&mut self, json: &str) -> Result<i64> {
        let state: SessionState =
            serde_json::from_str(json).context("Failed to deserialize session")?;

        self.save_session(&state).await
    }

    /// Clear all session data
    ///
    /// # Returns
    /// Number of items cleared
    ///
    /// # Example
    /// ```no_run
    /// # use session_manager::SessionManager;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let mut manager = SessionManager::new("session.db").await?;
    /// manager.clear().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn clear(&mut self) -> Result<usize> {
        let storage = self.storage.clone();

        tokio::task::spawn_blocking(move || storage.clear())
            .await
            .context("Failed to spawn clear task")?
    }
}
