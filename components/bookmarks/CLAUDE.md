# bookmarks Feature Component

## ⚠️ VERSION CONTROL RESTRICTIONS
**FORBIDDEN ACTIONS:**
- ❌ NEVER change project version to 1.0.0
- ❌ NEVER declare system "production ready"
- ❌ NEVER change lifecycle_state

**ALLOWED:**
- ✅ Report test coverage and quality metrics
- ✅ Complete your component work
- ✅ Suggest improvements

You are a specialized agent building ONLY the bookmarks feature component.

## Component Overview

**Purpose**: Bookmark management system for saving and organizing user bookmarks

**Responsibilities**:
- Bookmark CRUD operations (create, read, update, delete)
- Folder/hierarchy management
- Tags and metadata
- Import/export functionality (HTML bookmarks format)
- Search and filtering
- SQLite-based persistence
- Async operations using Tokio

**Technology Stack**:
- Rust 2021 edition
- Tokio for async runtime
- rusqlite for SQLite database
- serde for serialization
- anyhow for error handling
- parking_lot for synchronization

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
git commit -m "[bookmarks] test: Add failing test for bookmark creation"
git commit -m "[bookmarks] feat: Implement bookmark creation to pass test"
git commit -m "[bookmarks] refactor: Extract bookmark validation logic"
```

## API Specification

### BookmarkManager

```rust
use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Unique identifier for bookmarks
pub type BookmarkId = String;

/// Bookmark structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bookmark {
    pub id: BookmarkId,
    pub url: String,
    pub title: String,
    pub folder: Option<String>,  // Path like "Programming/Rust"
    pub tags: Vec<String>,
    pub favicon: Option<Vec<u8>>,
    pub created_at: i64,  // Unix timestamp
    pub updated_at: i64,
}

/// Folder structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkFolder {
    pub path: String,  // "Programming/Rust"
    pub parent: Option<String>,  // "Programming"
    pub children: Vec<String>,  // Subfolders
    pub bookmark_count: usize,
}

/// Bookmark manager interface
pub struct BookmarkManager {
    storage: BookmarkStorage,
}

impl BookmarkManager {
    /// Create new bookmark manager with database path
    pub async fn new(db_path: &str) -> Result<Self>;

    /// Add new bookmark
    pub async fn add_bookmark(&mut self, bookmark: Bookmark) -> Result<BookmarkId>;

    /// Get bookmark by ID
    pub async fn get_bookmark(&self, id: &BookmarkId) -> Result<Option<Bookmark>>;

    /// Update existing bookmark
    pub async fn update_bookmark(&mut self, bookmark: Bookmark) -> Result<()>;

    /// Delete bookmark
    pub async fn delete_bookmark(&mut self, id: &BookmarkId) -> Result<()>;

    /// List all bookmarks
    pub async fn list_bookmarks(&self) -> Result<Vec<Bookmark>>;

    /// List bookmarks in folder
    pub async fn list_bookmarks_in_folder(&self, folder: &str) -> Result<Vec<Bookmark>>;

    /// Search bookmarks by title or URL
    pub async fn search_bookmarks(&self, query: &str) -> Result<Vec<Bookmark>>;

    /// Search bookmarks by tag
    pub async fn find_by_tag(&self, tag: &str) -> Result<Vec<Bookmark>>;

    /// Create folder
    pub async fn create_folder(&mut self, path: &str) -> Result<()>;

    /// Delete folder (and optionally move bookmarks)
    pub async fn delete_folder(&mut self, path: &str, move_to: Option<String>) -> Result<()>;

    /// List all folders
    pub async fn list_folders(&self) -> Result<Vec<BookmarkFolder>>;

    /// Move bookmark to different folder
    pub async fn move_bookmark(&mut self, id: &BookmarkId, folder: Option<String>) -> Result<()>;

    /// Import bookmarks from HTML file
    pub async fn import_html(&mut self, html_content: &str) -> Result<usize>;

    /// Export bookmarks to HTML format
    pub async fn export_html(&self) -> Result<String>;

    /// Get bookmark count
    pub async fn count(&self) -> Result<usize>;

    /// Clear all bookmarks
    pub async fn clear(&mut self) -> Result<()>;
}
```

## Database Schema

```sql
CREATE TABLE bookmarks (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    folder TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    favicon BLOB
);

CREATE TABLE bookmark_tags (
    bookmark_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY (bookmark_id, tag),
    FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE
);

CREATE TABLE bookmark_folders (
    path TEXT PRIMARY KEY,
    parent TEXT,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_bookmarks_folder ON bookmarks(folder);
CREATE INDEX idx_bookmarks_url ON bookmarks(url);
CREATE INDEX idx_bookmarks_title ON bookmarks(title);
CREATE INDEX idx_bookmark_tags_tag ON bookmark_tags(tag);
```

## Implementation Requirements

### 1. BookmarkStorage (src/storage.rs)

**Purpose**: SQLite database operations

**Required functionality**:
- Database initialization with schema
- CRUD operations
- Transaction support
- Concurrent access safety (RwLock)

**Test requirements**:
- Test database creation
- Test all CRUD operations
- Test folder operations
- Test tag operations
- Test search functionality

### 2. BookmarkManager (src/lib.rs)

**Purpose**: High-level bookmark management API

**Required functionality**:
- All API methods listed above
- Input validation (URL format, folder paths)
- Error handling for duplicate URLs (optional warning, not error)
- Bookmark ID generation (UUID v4)

**Test requirements**:
- Test all public API methods
- Test error cases (invalid URLs, non-existent IDs)
- Test folder hierarchy
- Test tag operations
- Test search and filtering

### 3. HTML Import/Export (src/import_export.rs)

**Purpose**: Netscape bookmark HTML format support

**Required functionality**:
- Parse HTML bookmarks file
- Generate HTML bookmarks file
- Handle folder hierarchy in HTML
- Preserve creation dates

**Test requirements**:
- Test parsing valid HTML
- Test generating HTML
- Test round-trip (import → export → import)
- Test malformed HTML handling

### 4. Validation (src/validation.rs)

**Purpose**: Input validation and sanitization

**Required functionality**:
- URL validation
- Folder path validation (no "../", absolute paths only)
- Title sanitization
- Tag validation

**Test requirements**:
- Test URL validation (valid/invalid formats)
- Test folder path validation
- Test malicious input rejection

## Quality Standards

**Test Coverage**: ≥ 80% (target 90%)

**Test Categories**:
- Unit tests: All functions and methods
- Integration tests: Database operations
- Property tests: URL validation, folder paths

**Code Quality**:
- No `unwrap()` in production code (use `?` or proper error handling)
- All public APIs documented with doc comments
- Clear error messages
- Safe concurrent access (use RwLock/Mutex appropriately)

## File Structure

```
components/bookmarks/
├── src/
│   ├── lib.rs              # BookmarkManager public API
│   ├── storage.rs          # SQLite storage implementation
│   ├── import_export.rs    # HTML import/export
│   ├── validation.rs       # Input validation
│   └── types.rs            # Data types
├── tests/
│   ├── unit/
│   │   ├── mod.rs
│   │   ├── bookmark_tests.rs
│   │   ├── folder_tests.rs
│   │   └── import_export_tests.rs
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
use bookmarks::BookmarkManager;

#[tokio::main]
async fn main() -> Result<()> {
    let mut manager = BookmarkManager::new("bookmarks.db").await?;

    // Add bookmark
    let bookmark = Bookmark {
        id: "".to_string(),  // Will be generated
        url: "https://rust-lang.org".to_string(),
        title: "Rust Programming Language".to_string(),
        folder: Some("Programming/Rust".to_string()),
        tags: vec!["programming".to_string(), "rust".to_string()],
        favicon: None,
        created_at: 0,  // Will be set
        updated_at: 0,
    };

    let id = manager.add_bookmark(bookmark).await?;

    // Search by tag
    let rust_bookmarks = manager.find_by_tag("rust").await?;

    // Export to HTML
    let html = manager.export_html().await?;

    Ok(())
}
```

## Completion Checklist

Before marking this component complete:

- [ ] All API methods implemented
- [ ] All unit tests passing (≥80% coverage)
- [ ] All integration tests passing
- [ ] SQLite storage working correctly
- [ ] HTML import/export functional
- [ ] Folder hierarchy working
- [ ] Tag operations working
- [ ] Search functionality working
- [ ] Input validation complete
- [ ] No compiler warnings
- [ ] Documentation complete
- [ ] README.md updated with examples
- [ ] component.yaml complete

## Notes

- This component is independent and doesn't require other browser components to function
- Can be tested in isolation with in-memory SQLite (":memory:")
- Thread-safe for concurrent access from multiple browser windows
- Should handle large bookmark collections (10,000+) efficiently

## Questions or Issues

If you encounter specification ambiguities or technical blockers, document them in this section and continue with reasonable assumptions.
