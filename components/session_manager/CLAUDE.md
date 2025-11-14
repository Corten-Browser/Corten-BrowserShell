# Session Manager Component - TDD Specification

**Component**: session_manager
**Type**: Feature (Level 2)
**Version**: 0.19.0
**Tech Stack**: Rust, SQLite, Tokio

## Overview

Session management system for saving and restoring browser state, crash recovery, and recently closed tabs tracking.

## Responsibilities

- Save complete browser session state (windows, tabs, positions)
- Restore session from saved state
- Automatic crash recovery with periodic saves
- Track recently closed tabs with restore capability
- Export/import sessions for backup
- SQLite persistence

## Dependencies

- `shared_types` (^0.19.0) - WindowId, TabId, common types
- `user_data` (^0.19.0) - Storage utilities
- `tokio` (1.35) - Async runtime
- `rusqlite` (0.30) - SQLite database
- `serde` (1.0) - Serialization
- `anyhow` (1.0) - Error handling

## API Specification

### Types

```rust
use shared_types::{WindowId, TabId};
use serde::{Deserialize, Serialize};

/// Session manager for browser state persistence
pub struct SessionManager {
    storage: SessionStorage,
    auto_save_enabled: bool,
    max_closed_tabs: usize,
}

/// Complete session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub windows: Vec<WindowState>,
    pub timestamp: i64,
}

/// Window state within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub id: String,  // WindowId as string for serialization
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
    pub tabs: Vec<TabState>,
    pub active_tab_index: Option<usize>,
}

/// Tab state within a window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub id: String,  // TabId as string for serialization
    pub url: String,
    pub title: String,
    pub position: usize,
}

/// Recently closed tab information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedTab {
    pub id: String,
    pub url: String,
    pub title: String,
    pub closed_at: i64,
    pub window_id: Option<String>,
    pub position: Option<usize>,
}
```

### SessionManager Methods

```rust
impl SessionManager {
    /// Create new session manager
    ///
    /// # Arguments
    /// * `db_path` - Path to SQLite database file
    ///
    /// # Example
    /// ```
    /// let manager = SessionManager::new("session.db").await?;
    /// ```
    pub async fn new(db_path: &str) -> Result<Self>;

    /// Save current session state
    ///
    /// # Arguments
    /// * `state` - Current browser session state
    ///
    /// # Returns
    /// Session ID of saved session
    ///
    /// # Example
    /// ```
    /// let session_id = manager.save_session(&state).await?;
    /// ```
    pub async fn save_session(&mut self, state: &SessionState) -> Result<i64>;

    /// Restore most recent session
    ///
    /// # Returns
    /// Session state if found, None if no sessions exist
    ///
    /// # Example
    /// ```
    /// if let Some(state) = manager.restore_session().await? {
    ///     // Restore windows and tabs from state
    /// }
    /// ```
    pub async fn restore_session(&self) -> Result<Option<SessionState>>;

    /// Get session by ID
    ///
    /// # Arguments
    /// * `session_id` - ID of session to retrieve
    ///
    /// # Returns
    /// Session state if found
    ///
    /// # Example
    /// ```
    /// let state = manager.get_session(123).await?;
    /// ```
    pub async fn get_session(&self, session_id: i64) -> Result<Option<SessionState>>;

    /// List all saved sessions
    ///
    /// # Returns
    /// Vector of (session_id, timestamp) tuples
    ///
    /// # Example
    /// ```
    /// let sessions = manager.list_sessions().await?;
    /// for (id, timestamp) in sessions {
    ///     println!("Session {} at {}", id, timestamp);
    /// }
    /// ```
    pub async fn list_sessions(&self) -> Result<Vec<(i64, i64)>>;

    /// Delete old sessions
    ///
    /// # Arguments
    /// * `keep_count` - Number of recent sessions to keep
    ///
    /// # Returns
    /// Number of sessions deleted
    ///
    /// # Example
    /// ```
    /// let deleted = manager.cleanup_old_sessions(10).await?;
    /// ```
    pub async fn cleanup_old_sessions(&mut self, keep_count: usize) -> Result<usize>;

    /// Record a closed tab
    ///
    /// # Arguments
    /// * `tab` - Closed tab information
    ///
    /// # Example
    /// ```
    /// manager.record_closed_tab(closed_tab).await?;
    /// ```
    pub async fn record_closed_tab(&mut self, tab: ClosedTab) -> Result<()>;

    /// Get recently closed tabs
    ///
    /// # Arguments
    /// * `limit` - Maximum number of tabs to return
    ///
    /// # Returns
    /// Vector of recently closed tabs (most recent first)
    ///
    /// # Example
    /// ```
    /// let closed_tabs = manager.get_recently_closed(10).await?;
    /// for tab in closed_tabs {
    ///     println!("Recently closed: {}", tab.title);
    /// }
    /// ```
    pub async fn get_recently_closed(&self, limit: usize) -> Result<Vec<ClosedTab>>;

    /// Clear recently closed tabs
    ///
    /// # Returns
    /// Number of tabs cleared
    ///
    /// # Example
    /// ```
    /// let cleared = manager.clear_recently_closed().await?;
    /// ```
    pub async fn clear_recently_closed(&mut self) -> Result<usize>;

    /// Export session to JSON
    ///
    /// # Arguments
    /// * `session_id` - ID of session to export, or None for current
    ///
    /// # Returns
    /// JSON string representation of session
    ///
    /// # Example
    /// ```
    /// let json = manager.export_session(Some(123)).await?;
    /// ```
    pub async fn export_session(&self, session_id: Option<i64>) -> Result<String>;

    /// Import session from JSON
    ///
    /// # Arguments
    /// * `json` - JSON string representation of session
    ///
    /// # Returns
    /// Session ID of imported session
    ///
    /// # Example
    /// ```
    /// let session_id = manager.import_session(&json_data).await?;
    /// ```
    pub async fn import_session(&mut self, json: &str) -> Result<i64>;

    /// Clear all session data
    ///
    /// # Returns
    /// Number of items cleared
    ///
    /// # Example
    /// ```
    /// manager.clear().await?;
    /// ```
    pub async fn clear(&mut self) -> Result<usize>;
}
```

## Database Schema

```sql
-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    is_active BOOLEAN DEFAULT 1
);

-- Windows within sessions
CREATE TABLE IF NOT EXISTS session_windows (
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
);

-- Tabs within windows
CREATE TABLE IF NOT EXISTS session_tabs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_window_id INTEGER NOT NULL,
    tab_id TEXT NOT NULL,
    url TEXT NOT NULL,
    title TEXT,
    position INTEGER NOT NULL,
    FOREIGN KEY (session_window_id) REFERENCES session_windows(id) ON DELETE CASCADE
);

-- Recently closed tabs
CREATE TABLE IF NOT EXISTS closed_tabs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tab_id TEXT NOT NULL,
    url TEXT NOT NULL,
    title TEXT,
    closed_at INTEGER NOT NULL,
    window_id TEXT,
    position INTEGER
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_sessions_timestamp ON sessions(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_closed_tabs_timestamp ON closed_tabs(closed_at DESC);
```

## Test Requirements

### Unit Tests (>= 40 tests)

#### SessionManager Creation and Initialization
```rust
#[tokio::test]
async fn test_session_manager_creation() {
    // GIVEN: Database path
    // WHEN: Creating new SessionManager
    // THEN: Manager created successfully with empty state
}

#[tokio::test]
async fn test_session_manager_creates_schema() {
    // GIVEN: New database
    // WHEN: Creating SessionManager
    // THEN: All tables and indexes created
}
```

#### Save and Restore Session
```rust
#[tokio::test]
async fn test_save_empty_session() {
    // GIVEN: SessionManager and empty session state
    // WHEN: Saving session
    // THEN: Session saved successfully with ID
}

#[tokio::test]
async fn test_save_session_with_windows() {
    // GIVEN: Session state with 2 windows
    // WHEN: Saving session
    // THEN: Session and windows saved correctly
}

#[tokio::test]
async fn test_save_session_with_tabs() {
    // GIVEN: Session with window containing 3 tabs
    // WHEN: Saving session
    // THEN: All tabs saved with correct positions
}

#[tokio::test]
async fn test_restore_most_recent_session() {
    // GIVEN: Multiple saved sessions
    // WHEN: Restoring session
    // THEN: Most recent session returned
}

#[tokio::test]
async fn test_restore_session_by_id() {
    // GIVEN: Multiple sessions with specific IDs
    // WHEN: Getting session by ID
    // THEN: Correct session returned
}

#[tokio::test]
async fn test_restore_nonexistent_session() {
    // GIVEN: Empty database
    // WHEN: Restoring session
    // THEN: None returned
}

#[tokio::test]
async fn test_save_window_positions() {
    // GIVEN: Window with specific x, y, width, height
    // WHEN: Saving and restoring
    // THEN: Positions restored correctly
}

#[tokio::test]
async fn test_save_maximized_state() {
    // GIVEN: Maximized window
    // WHEN: Saving and restoring
    // THEN: Maximized state preserved
}

#[tokio::test]
async fn test_save_active_tab_index() {
    // GIVEN: Window with active tab at index 2
    // WHEN: Saving and restoring
    // THEN: Active tab index restored
}
```

#### Session Management
```rust
#[tokio::test]
async fn test_list_sessions() {
    // GIVEN: 5 saved sessions
    // WHEN: Listing sessions
    // THEN: All sessions returned with timestamps
}

#[tokio::test]
async fn test_cleanup_old_sessions() {
    // GIVEN: 10 saved sessions
    // WHEN: Cleanup keeping 3
    // THEN: 7 sessions deleted, 3 remain
}

#[tokio::test]
async fn test_cleanup_preserves_recent() {
    // GIVEN: Sessions with different timestamps
    // WHEN: Cleanup keeping 5
    // THEN: Most recent 5 preserved
}

#[tokio::test]
async fn test_multiple_saves_create_separate_sessions() {
    // GIVEN: SessionManager
    // WHEN: Saving 3 different states
    // THEN: 3 separate session records created
}
```

#### Recently Closed Tabs
```rust
#[tokio::test]
async fn test_record_closed_tab() {
    // GIVEN: SessionManager and closed tab
    // WHEN: Recording closed tab
    // THEN: Tab saved successfully
}

#[tokio::test]
async fn test_get_recently_closed_tabs() {
    // GIVEN: 5 closed tabs
    // WHEN: Getting recently closed (limit 10)
    // THEN: All 5 tabs returned in reverse chronological order
}

#[tokio::test]
async fn test_recently_closed_limit() {
    // GIVEN: 20 closed tabs
    // WHEN: Getting recently closed (limit 10)
    // THEN: Only 10 most recent tabs returned
}

#[tokio::test]
async fn test_clear_recently_closed() {
    // GIVEN: 10 closed tabs
    // WHEN: Clearing recently closed
    // THEN: All closed tabs removed
}

#[tokio::test]
async fn test_closed_tab_with_window_id() {
    // GIVEN: Closed tab with window ID
    // WHEN: Recording and retrieving
    // THEN: Window ID preserved
}

#[tokio::test]
async fn test_closed_tab_with_position() {
    // GIVEN: Closed tab at position 5
    // WHEN: Recording and retrieving
    // THEN: Position preserved
}
```

#### Import/Export
```rust
#[tokio::test]
async fn test_export_session_to_json() {
    // GIVEN: Saved session with windows and tabs
    // WHEN: Exporting to JSON
    // THEN: Valid JSON string returned
}

#[tokio::test]
async fn test_import_session_from_json() {
    // GIVEN: Valid session JSON
    // WHEN: Importing session
    // THEN: Session restored with correct data
}

#[tokio::test]
async fn test_export_import_roundtrip() {
    // GIVEN: Session state
    // WHEN: Exporting then importing
    // THEN: Restored state matches original
}

#[tokio::test]
async fn test_import_invalid_json() {
    // GIVEN: Invalid JSON string
    // WHEN: Importing
    // THEN: Error returned
}
```

#### Edge Cases and Error Handling
```rust
#[tokio::test]
async fn test_save_session_with_no_windows() {
    // GIVEN: Session state with empty windows vector
    // WHEN: Saving
    // THEN: Session saved successfully
}

#[tokio::test]
async fn test_save_window_with_no_tabs() {
    // GIVEN: Window with empty tabs vector
    // WHEN: Saving
    // THEN: Window saved successfully
}

#[tokio::test]
async fn test_restore_with_corrupt_data() {
    // GIVEN: Database with missing foreign key data
    // WHEN: Restoring
    // THEN: Error or partial data returned gracefully
}

#[tokio::test]
async fn test_clear_removes_all_data() {
    // GIVEN: Database with sessions and closed tabs
    // WHEN: Clearing
    // THEN: All tables emptied
}

#[tokio::test]
async fn test_concurrent_saves() {
    // GIVEN: SessionManager
    // WHEN: Multiple concurrent save operations
    // THEN: All saves complete successfully
}

#[tokio::test]
async fn test_url_validation_in_tabs() {
    // GIVEN: Tab with empty URL
    // WHEN: Saving
    // THEN: Still saved (URLs can be empty for new tabs)
}

#[tokio::test]
async fn test_large_session_state() {
    // GIVEN: Session with 100 windows, 10 tabs each
    // WHEN: Saving and restoring
    // THEN: All data preserved correctly
}

#[tokio::test]
async fn test_timestamp_accuracy() {
    // GIVEN: Two sessions saved 1 second apart
    // WHEN: Checking timestamps
    // THEN: Timestamps correctly ordered
}
```

### Integration Tests (>= 10 tests)

```rust
#[tokio::test]
async fn test_session_persistence_across_restarts() {
    // GIVEN: SessionManager saves state and is dropped
    // WHEN: Creating new SessionManager with same database
    // THEN: Previous session can be restored
}

#[tokio::test]
async fn test_crash_recovery_scenario() {
    // GIVEN: Session saved during normal operation
    // WHEN: Simulating crash and recovery
    // THEN: Session restored to last saved state
}

#[tokio::test]
async fn test_closed_tabs_persist_across_restarts() {
    // GIVEN: Recently closed tabs recorded
    // WHEN: Restarting SessionManager
    // THEN: Closed tabs still available
}

#[tokio::test]
async fn test_session_cleanup_doesnt_affect_closed_tabs() {
    // GIVEN: Old sessions and recent closed tabs
    // WHEN: Cleaning up old sessions
    // THEN: Closed tabs remain intact
}

#[tokio::test]
async fn test_multiple_windows_multiple_tabs_scenario() {
    // GIVEN: Complex session (3 windows, varying tab counts)
    // WHEN: Saving and restoring
    // THEN: All windows and tabs restored correctly
}

#[tokio::test]
async fn test_session_evolution_over_time() {
    // GIVEN: Session saved at T0, T1, T2
    // WHEN: Restoring at T3
    // THEN: Most recent (T2) session restored
}

#[tokio::test]
async fn test_export_import_backup_workflow() {
    // GIVEN: Active session
    // WHEN: Exporting to JSON, clearing DB, reimporting
    // THEN: Session fully restored from backup
}

#[tokio::test]
async fn test_partial_session_restore() {
    // GIVEN: Session with some invalid URLs
    // WHEN: Restoring
    // THEN: Valid tabs restored, invalid ones skipped or handled
}

#[tokio::test]
async fn test_session_versioning() {
    // GIVEN: Sessions saved at different times
    // WHEN: Listing all sessions
    // THEN: Sessions can be distinguished by ID and timestamp
}

#[tokio::test]
async fn test_max_closed_tabs_limit() {
    // GIVEN: Recording 1000 closed tabs
    // WHEN: Getting recently closed
    // THEN: Only reasonable number returned (e.g., 100 max)
}
```

## Quality Standards

### Coverage Target
- **Minimum**: 80%
- **Target**: 90%+

### Code Quality
- No `unwrap()` in production code
- All errors use `anyhow::Result`
- Input validation on all public methods
- SQLite prepared statements (no SQL injection)
- Proper transaction handling for multi-table operations

### Performance
- Session save: < 100ms for typical session (5 windows, 50 tabs)
- Session restore: < 50ms
- Recently closed lookup: < 10ms

### Security
- SQL injection prevention (prepared statements)
- Path validation for database file
- No sensitive data in logs

## Module Structure

```
components/session_manager/
├── src/
│   ├── lib.rs           # SessionManager public API
│   ├── types.rs         # SessionState, WindowState, TabState, ClosedTab
│   ├── storage.rs       # SessionStorage (SQLite operations)
│   └── validation.rs    # Input validation
├── tests/
│   ├── unit/
│   │   ├── session_manager_test.rs
│   │   ├── storage_test.rs
│   │   └── validation_test.rs
│   └── integration/
│       └── session_persistence_test.rs
├── Cargo.toml
├── component.yaml
├── CLAUDE.md           # This file
└── README.md           # User-facing documentation
```

## Implementation Notes

1. **SQLite Transactions**: Use transactions for multi-table operations (save_session involves 3 tables)
2. **Async Operations**: All I/O operations must be async
3. **Error Handling**: Use anyhow for error propagation
4. **Validation**: Validate all inputs (especially file paths, URLs)
5. **Testing**: Follow TDD - write tests first, implementation second
6. **Thread Safety**: Use RwLock/Mutex where needed for concurrent access

## Development Workflow

### TDD Cycle
1. **RED**: Write failing test
2. **GREEN**: Implement minimal code to pass test
3. **REFACTOR**: Clean up code while keeping tests passing

### Test First Order
1. SessionManager::new
2. save_session (empty)
3. save_session (with windows)
4. save_session (with tabs)
5. restore_session
6. get_session by ID
7. list_sessions
8. cleanup_old_sessions
9. record_closed_tab
10. get_recently_closed
11. clear_recently_closed
12. export_session
13. import_session
14. clear

## Success Criteria

- [ ] All 50+ tests passing
- [ ] >= 80% code coverage
- [ ] Zero linting errors
- [ ] No security vulnerabilities
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] Integration tests passing
