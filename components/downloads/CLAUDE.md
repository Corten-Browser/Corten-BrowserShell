# downloads Feature Component

## ⚠️ VERSION CONTROL RESTRICTIONS
**FORBIDDEN ACTIONS:**
- ❌ NEVER change project version to 1.0.0
- ❌ NEVER declare system "production ready"
- ❌ NEVER change lifecycle_state

**ALLOWED:**
- ✅ Report test coverage and quality metrics
- ✅ Complete your component work
- ✅ Suggest improvements

You are a specialized agent building ONLY the downloads feature component.

## Component Overview

**Purpose**: Download management system for tracking and controlling file downloads

**Responsibilities**:
- Download lifecycle management (start, pause, resume, cancel)
- Download progress tracking
- File system operations
- Download history and metadata
- Multiple simultaneous downloads
- SQLite-based persistence
- Async operations using Tokio

**Technology Stack**:
- Rust 2021 edition
- Tokio for async runtime
- rusqlite for SQLite database
- serde for serialization
- anyhow for error handling
- parking_lot for synchronization
- tokio::fs for async file operations

**Dependencies**:
- shared_types (Level 0)
- user_data (Level 2) - for storage patterns

## MANDATORY: Test-Driven Development (TDD)

### TDD is Not Optional

**ALL code in this component MUST be developed using TDD.** This is a strict requirement, not a suggestion.

### TDD Workflow (Red-Green-Refactor)

**You MUST follow this cycle for EVERY feature:**

1. **RED**: Write a failing test
   - Write the test FIRST before any implementation code
   - Run the test and verify it FAILS
   - The test defines the behavior you want

2. **GREEN**: Make the test pass
   - Write the MINIMUM code needed to pass the test
   - Don't add extra features
   - Run the test and verify it PASSES

3. **REFACTOR**: Improve the code
   - Clean up duplication
   - Improve naming and structure
   - Maintain passing tests throughout

### TDD Commit Pattern

Your git history MUST show TDD practice:

```bash
# Commit sequence example
git commit -m "[downloads] test: Add failing test for download start"
git commit -m "[downloads] feat: Implement download start to pass test"
git commit -m "[downloads] refactor: Extract download state management"
```

## API Specification

### DownloadManager

```rust
use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Unique identifier for downloads
pub type DownloadId = String;

/// Download status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Downloading { bytes_downloaded: u64, bytes_total: Option<u64> },
    Paused { bytes_downloaded: u64, bytes_total: Option<u64> },
    Completed { bytes_downloaded: u64, file_path: String },
    Failed { error: String },
    Cancelled,
}

/// Download metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub id: DownloadId,
    pub url: String,
    pub file_name: String,
    pub save_path: String,
    pub mime_type: Option<String>,
    pub status: DownloadStatus,
    pub created_at: i64,  // Unix timestamp
    pub completed_at: Option<i64>,
}

/// Download progress event
#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Started { id: DownloadId },
    Progress { id: DownloadId, bytes_downloaded: u64, bytes_total: Option<u64> },
    Completed { id: DownloadId, file_path: String },
    Failed { id: DownloadId, error: String },
    Paused { id: DownloadId },
    Resumed { id: DownloadId },
    Cancelled { id: DownloadId },
}

/// Download manager interface
pub struct DownloadManager {
    storage: DownloadStorage,
    active_downloads: HashMap<DownloadId, DownloadHandle>,
    event_tx: mpsc::UnboundedSender<DownloadEvent>,
}

impl DownloadManager {
    /// Create new download manager
    pub async fn new(db_path: &str, download_dir: &str) -> Result<Self>;

    /// Start new download
    pub async fn start_download(&mut self, url: String, save_path: Option<String>) -> Result<DownloadId>;

    /// Pause download
    pub async fn pause_download(&mut self, id: &DownloadId) -> Result<()>;

    /// Resume paused download
    pub async fn resume_download(&mut self, id: &DownloadId) -> Result<()>;

    /// Cancel download
    pub async fn cancel_download(&mut self, id: &DownloadId) -> Result<()>;

    /// Get download by ID
    pub async fn get_download(&self, id: &DownloadId) -> Result<Option<Download>>;

    /// List all downloads
    pub async fn list_downloads(&self) -> Result<Vec<Download>>;

    /// List active downloads
    pub async fn list_active_downloads(&self) -> Result<Vec<Download>>;

    /// List completed downloads
    pub async fn list_completed_downloads(&self) -> Result<Vec<Download>>;

    /// Delete download record
    pub async fn delete_download(&mut self, id: &DownloadId) -> Result<()>;

    /// Clear completed downloads
    pub async fn clear_completed(&mut self) -> Result<()>;

    /// Get event receiver for download events
    pub fn event_receiver(&self) -> mpsc::UnboundedReceiver<DownloadEvent>;

    /// Shutdown manager and cancel all active downloads
    pub async fn shutdown(&mut self) -> Result<()>;
}
```

## Database Schema

```sql
CREATE TABLE downloads (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    file_name TEXT NOT NULL,
    save_path TEXT NOT NULL,
    mime_type TEXT,
    status TEXT NOT NULL,  -- JSON serialized DownloadStatus
    created_at INTEGER NOT NULL,
    completed_at INTEGER
);

CREATE INDEX idx_downloads_created_at ON downloads(created_at DESC);
CREATE INDEX idx_downloads_status ON downloads(status);
```

## Implementation Requirements

### 1. DownloadStorage (src/storage.rs)

**Purpose**: SQLite database operations for download records

**Required functionality**:
- Database initialization with schema
- CRUD operations for download records
- Query by status
- List operations with sorting

**Test requirements**:
- Test database creation
- Test all CRUD operations
- Test query by status
- Test sorting by creation date

### 2. DownloadManager (src/lib.rs)

**Purpose**: High-level download management API

**Required functionality**:
- Start/pause/resume/cancel downloads
- Track multiple simultaneous downloads
- Progress reporting via events
- File system operations
- Error handling for network failures, disk full, etc.

**Test requirements**:
- Test download lifecycle (start → complete)
- Test pause/resume functionality
- Test multiple simultaneous downloads
- Test error handling (invalid URL, disk full simulation)
- Test cancellation

### 3. DownloadHandle (src/handle.rs)

**Purpose**: Individual download execution and control

**Required functionality**:
- HTTP download with progress tracking
- Pause/resume support (Range requests)
- File writing
- Cleanup on cancellation

**Test requirements**:
- Test HTTP download (mock HTTP client)
- Test pause/resume
- Test file writing
- Test cleanup on cancel

### 4. Validation (src/validation.rs)

**Purpose**: Input validation and sanitization

**Required functionality**:
- URL validation
- File name sanitization (remove path traversal attempts)
- Save path validation
- Disk space checking

**Test requirements**:
- Test URL validation
- Test file name sanitization (prevent "../" attacks)
- Test disk space checking

## Quality Standards

**Test Coverage**: ≥ 80% (target 90%)

**Test Categories**:
- Unit tests: All functions and methods
- Integration tests: Database operations, file system
- Mock tests: HTTP downloads (use mock HTTP server)

**Code Quality**:
- No `unwrap()` in production code
- All public APIs documented
- Clear error messages
- Safe concurrent access
- Proper cleanup on failure

## File Structure

```
components/downloads/
├── src/
│   ├── lib.rs              # DownloadManager public API
│   ├── storage.rs          # SQLite storage implementation
│   ├── handle.rs           # Individual download execution
│   ├── validation.rs       # Input validation
│   └── types.rs            # Data types
├── tests/
│   ├── unit/
│   │   ├── mod.rs
│   │   ├── manager_tests.rs
│   │   ├── handle_tests.rs
│   │   └── validation_tests.rs
│   └── integration/
│       ├── mod.rs
│       └── storage_tests.rs
├── Cargo.toml
├── CLAUDE.md               # This file
├── README.md
└── component.yaml
```

## Example Usage

```rust
use downloads::{DownloadManager, DownloadEvent};

#[tokio::main]
async fn main() -> Result<()> {
    let mut manager = DownloadManager::new("downloads.db", "./downloads").await?;
    let mut events = manager.event_receiver();

    // Start download
    let id = manager.start_download(
        "https://example.com/file.zip".to_string(),
        None  // Auto-generate save path
    ).await?;

    // Listen for events
    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            match event {
                DownloadEvent::Progress { id, bytes_downloaded, bytes_total } => {
                    println!("Progress: {}/{:?}", bytes_downloaded, bytes_total);
                }
                DownloadEvent::Completed { id, file_path } => {
                    println!("Download complete: {}", file_path);
                }
                DownloadEvent::Failed { id, error } => {
                    println!("Download failed: {}", error);
                }
                _ => {}
            }
        }
    });

    // Pause download
    manager.pause_download(&id).await?;

    // Resume download
    manager.resume_download(&id).await?;

    Ok(())
}
```

## Completion Checklist

Before marking this component complete:

- [ ] All API methods implemented
- [ ] All unit tests passing (≥80% coverage)
- [ ] All integration tests passing
- [ ] SQLite storage working correctly
- [ ] Download start/pause/resume/cancel working
- [ ] Progress tracking working
- [ ] Event system working
- [ ] File system operations working
- [ ] Input validation complete
- [ ] Error handling comprehensive
- [ ] No compiler warnings
- [ ] Documentation complete
- [ ] README.md updated with examples
- [ ] component.yaml complete

## Implementation Notes

**Simplified Download Logic**:
Since this is a browser shell component without a real network stack, the actual HTTP downloading can be mocked or simplified:

```rust
// Simplified download simulation for testing
async fn download_file(url: &str, path: &str) -> Result<()> {
    // In real implementation, this would:
    // 1. Make HTTP request
    // 2. Stream response body
    // 3. Write to file
    // 4. Report progress

    // For now, simulate with delay and mock data
    tokio::time::sleep(Duration::from_millis(100)).await;
    tokio::fs::write(path, b"mock file content").await?;
    Ok(())
}
```

**Real implementation would integrate with network component when available.**

## Questions or Issues

If you encounter specification ambiguities or technical blockers, document them in this section and continue with reasonable assumptions.
