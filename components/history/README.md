# History Component

**Type**: Feature (Level 2)
**Version**: 0.18.0
**Tech Stack**: Rust, SQLite, Tokio

## Overview

Browsing history tracking and management system with search, frecency scoring, and privacy controls.

## Features

- Visit tracking (URL, title, timestamp)
- Search and filtering
- Frecency scoring (frequency + recency)
- Most visited pages
- Privacy controls (clear history)
- SQLite-based persistence

## Usage

```rust
use history::{HistoryManager, HistoryVisit, TransitionType, SearchQuery};

#[tokio::main]
async fn main() -> Result<()> {
    let mut manager = HistoryManager::new("history.db").await?;

    // Record visit
    let visit = HistoryVisit {
        id: "".to_string(),
        url: "https://rust-lang.org".to_string(),
        title: "Rust Programming Language".to_string(),
        visit_time: chrono::Utc::now().timestamp(),
        visit_duration: None,
        from_url: None,
        transition_type: TransitionType::Typed,
    };

    let id = manager.record_visit(visit).await?;

    // Search history
    let results = manager.search(SearchQuery {
        text: Some("rust".to_string()),
        start_time: None,
        end_time: None,
        limit: 10,
    }).await?;

    // Get most visited
    let most_visited = manager.get_most_visited(10).await?;

    Ok(())
}
```

## Testing

```bash
cargo test
```

## Development

See CLAUDE.md for detailed development instructions and TDD requirements.
