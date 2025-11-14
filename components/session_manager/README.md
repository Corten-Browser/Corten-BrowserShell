# Session Manager

Session management system for the Corten Browser Shell, providing persistent browser state storage, crash recovery, and recently closed tabs tracking.

## Features

- **Session Persistence**: Save and restore complete browser sessions
- **Crash Recovery**: Automatic session backup for recovery after unexpected shutdowns
- **Recently Closed Tabs**: Track and restore recently closed tabs
- **Import/Export**: Backup sessions to JSON for portability
- **SQLite Storage**: Reliable, lightweight persistent storage
- **Async Operations**: Non-blocking I/O for smooth performance

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
session_manager = { path = "../session_manager" }
tokio = { version = "1.35", features = ["full"] }
```

## Quick Start

```rust
use session_manager::{SessionManager, SessionState, WindowState, TabState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create session manager
    let mut manager = SessionManager::new("browser_sessions.db").await?;

    // Create a session
    let mut state = SessionState::new(chrono::Utc::now().timestamp());

    // Add windows and tabs
    let mut window = WindowState::new("win-1".to_string(), 1920, 1080);
    window.tabs.push(TabState::new(
        "tab-1".to_string(),
        "https://example.com".to_string(),
        "Example".to_string(),
        0,
    ));
    state.windows.push(window);

    // Save session
    let session_id = manager.save_session(&state).await?;
    println!("Session saved with ID: {}", session_id);

    // Restore session
    if let Some(restored) = manager.restore_session().await? {
        println!("Restored {} windows", restored.windows.len());
    }

    Ok(())
}
```

## Usage Examples

### Saving Browser State

```rust
use session_manager::{SessionManager, SessionState, WindowState, TabState};

async fn save_current_session(
    manager: &mut SessionManager,
    windows: Vec<WindowState>,
) -> anyhow::Result<i64> {
    let mut state = SessionState::new(chrono::Utc::now().timestamp());
    state.windows = windows;

    let session_id = manager.save_session(&state).await?;
    println!("Session {} saved successfully", session_id);

    Ok(session_id)
}
```

### Crash Recovery

```rust
async fn recover_from_crash() -> anyhow::Result<()> {
    let manager = SessionManager::new("browser_sessions.db").await?;

    // Restore most recent session
    match manager.restore_session().await? {
        Some(session) => {
            println!("Recovering session from {}", session.timestamp);
            for window in session.windows {
                println!("  Window {}: {} tabs", window.id, window.tabs.len());
            }
            Ok(())
        }
        None => {
            println!("No previous session found");
            Ok(())
        }
    }
}
```

### Managing Window State

```rust
use session_manager::WindowState;

fn create_window_state() -> WindowState {
    let mut window = WindowState::new("main-window".to_string(), 1920, 1080);

    // Set position
    window.x = Some(100);
    window.y = Some(50);

    // Set maximized state
    window.maximized = true;

    // Set active tab
    window.active_tab_index = Some(0);

    window
}
```

### Recently Closed Tabs

```rust
use session_manager::{SessionManager, ClosedTab};

async fn manage_closed_tabs(manager: &mut SessionManager) -> anyhow::Result<()> {
    // Record a closed tab
    let closed = ClosedTab::new(
        "tab-123".to_string(),
        "https://example.com".to_string(),
        "Example Site".to_string(),
        chrono::Utc::now().timestamp(),
    );
    manager.record_closed_tab(closed).await?;

    // Get recently closed tabs
    let recent = manager.get_recently_closed(10).await?;
    println!("Recently closed tabs:");
    for tab in recent {
        println!("  - {} ({})", tab.title, tab.url);
    }

    // Clear history
    let cleared = manager.clear_recently_closed().await?;
    println!("Cleared {} closed tabs", cleared);

    Ok(())
}
```

### Session List and Cleanup

```rust
async fn manage_sessions(manager: &mut SessionManager) -> anyhow::Result<()> {
    // List all sessions
    let sessions = manager.list_sessions().await?;
    println!("Found {} sessions:", sessions.len());
    for (id, timestamp) in &sessions {
        println!("  Session {}: {}", id, timestamp);
    }

    // Keep only last 10 sessions
    let deleted = manager.cleanup_old_sessions(10).await?;
    println!("Deleted {} old sessions", deleted);

    Ok(())
}
```

### Import/Export

```rust
async fn backup_and_restore(manager: &mut SessionManager) -> anyhow::Result<()> {
    // Export current session
    let session_id = 123; // Example session ID
    let json_backup = manager.export_session(Some(session_id)).await?;

    // Save to file
    std::fs::write("session_backup.json", &json_backup)?;
    println!("Session exported to session_backup.json");

    // Later, import from backup
    let json_data = std::fs::read_to_string("session_backup.json")?;
    let restored_id = manager.import_session(&json_data).await?;
    println!("Session restored with ID: {}", restored_id);

    Ok(())
}
```

### Complete Example: Browser Session Manager

```rust
use session_manager::{SessionManager, SessionState, WindowState, TabState, ClosedTab};
use chrono::Utc;

struct BrowserSession {
    manager: SessionManager,
}

impl BrowserSession {
    async fn new(db_path: &str) -> anyhow::Result<Self> {
        let manager = SessionManager::new(db_path).await?;
        Ok(Self { manager })
    }

    async fn save_current_state(&mut self, windows: Vec<WindowState>) -> anyhow::Result<()> {
        let mut state = SessionState::new(Utc::now().timestamp());
        state.windows = windows;

        let session_id = self.manager.save_session(&state).await?;
        println!("Session saved: {}", session_id);
        Ok(())
    }

    async fn restore_last_session(&self) -> anyhow::Result<Option<SessionState>> {
        self.manager.restore_session().await
    }

    async fn close_tab(&mut self, tab_id: String, url: String, title: String) -> anyhow::Result<()> {
        let closed = ClosedTab::new(tab_id, url, title, Utc::now().timestamp());
        self.manager.record_closed_tab(closed).await
    }

    async fn get_recently_closed(&self) -> anyhow::Result<Vec<ClosedTab>> {
        self.manager.get_recently_closed(20).await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut browser = BrowserSession::new("my_browser.db").await?;

    // Create some windows with tabs
    let mut window1 = WindowState::new("win-1".to_string(), 1920, 1080);
    window1.tabs.push(TabState::new(
        "tab-1".to_string(),
        "https://github.com".to_string(),
        "GitHub".to_string(),
        0,
    ));
    window1.tabs.push(TabState::new(
        "tab-2".to_string(),
        "https://rust-lang.org".to_string(),
        "Rust".to_string(),
        1,
    ));
    window1.active_tab_index = Some(0);

    // Save session
    browser.save_current_state(vec![window1]).await?;

    // Simulate closing a tab
    browser.close_tab(
        "tab-2".to_string(),
        "https://rust-lang.org".to_string(),
        "Rust".to_string(),
    ).await?;

    // Get recently closed tabs
    let closed = browser.get_recently_closed().await?;
    println!("Recently closed: {:?}", closed);

    Ok(())
}
```

## Architecture

### Components

- **SessionManager**: High-level API for session operations
- **SessionStorage**: SQLite backend for data persistence
- **Types**: Data structures for sessions, windows, tabs
- **Validation**: Input validation utilities

### Data Model

```
Session
├── timestamp
└── windows[]
    ├── id, dimensions, position
    ├── maximized state
    ├── active tab index
    └── tabs[]
        ├── id, url, title
        └── position
```

### Database Schema

The component uses SQLite with the following schema:

- **sessions**: Session records with timestamps
- **session_windows**: Window states within sessions
- **session_tabs**: Tab states within windows
- **closed_tabs**: Recently closed tabs history

All tables use foreign keys with CASCADE delete for data integrity.

## Testing

Run the test suite:

```bash
cargo test
```

The component includes:
- **31 unit tests**: Core functionality testing
- **10 integration tests**: End-to-end scenarios
- **7 validation tests**: Input validation
- **13 doc tests**: Documentation examples

Total: **61 tests** with **100% pass rate**

## Performance

Typical performance metrics:

- Session save: < 100ms (5 windows, 50 tabs)
- Session restore: < 50ms
- Recently closed lookup: < 10ms
- Database initialization: < 20ms

## Error Handling

All public methods return `anyhow::Result<T>` for comprehensive error handling:

```rust
match manager.save_session(&state).await {
    Ok(session_id) => println!("Saved: {}", session_id),
    Err(e) => eprintln!("Failed to save session: {}", e),
}
```

## Security

- **SQL Injection Prevention**: All queries use prepared statements
- **Path Validation**: Database paths are validated before use
- **No Secrets Logged**: Sensitive data never appears in logs

## Dependencies

- `tokio`: Async runtime
- `rusqlite`: SQLite database
- `serde`: Serialization
- `anyhow`: Error handling
- `chrono`: Timestamp management

## Version

Current version: **0.19.0**

Compatible with Corten Browser Shell v0.17.0+

## License

Part of the Corten Browser Shell project.

## Contributing

See the main Corten-BrowserShell repository for contribution guidelines.
