# Bookmarks Component

**Type**: Feature (Level 2)
**Version**: 0.18.0
**Tech Stack**: Rust, SQLite, Tokio

## Overview

Bookmark management system for saving and organizing user bookmarks with folders, tags, and import/export functionality.

## Features

- Bookmark CRUD operations
- Folder hierarchy management
- Tag support for categorization
- Import/export (HTML bookmarks format)
- Search and filtering
- SQLite-based persistence

## Usage

```rust
use bookmarks::{BookmarkManager, Bookmark};

#[tokio::main]
async fn main() -> Result<()> {
    let mut manager = BookmarkManager::new("bookmarks.db").await?;

    // Add bookmark
    let bookmark = Bookmark {
        id: "".to_string(),
        url: "https://rust-lang.org".to_string(),
        title: "Rust Programming Language".to_string(),
        folder: Some("Programming/Rust".to_string()),
        tags: vec!["programming".to_string()],
        favicon: None,
        created_at: 0,
        updated_at: 0,
    };

    let id = manager.add_bookmark(bookmark).await?;

    // Search by tag
    let results = manager.find_by_tag("programming").await?;

    Ok(())
}
```

## Testing

```bash
cargo test
```

## Development

See CLAUDE.md for detailed development instructions and TDD requirements.
