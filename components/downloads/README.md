# Downloads Component

**Type**: Feature (Level 2)
**Version**: 0.18.0
**Tech Stack**: Rust, SQLite, Tokio

## Overview

Download management system for tracking and controlling file downloads with pause/resume functionality and progress tracking.

## Features

- Download lifecycle management (start, pause, resume, cancel)
- Progress tracking with events
- Multiple simultaneous downloads
- Download history
- SQLite-based persistence
- File system operations

## Usage

```rust
use downloads::{DownloadManager, DownloadEvent};

#[tokio::main]
async fn main() -> Result<()> {
    let mut manager = DownloadManager::new("downloads.db", "./downloads").await?;
    let mut events = manager.event_receiver();

    // Start download
    let id = manager.start_download(
        "https://example.com/file.zip".to_string(),
        None
    ).await?;

    // Listen for events
    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            match event {
                DownloadEvent::Progress { id, bytes_downloaded, bytes_total } => {
                    println!("Progress: {}/{:?}", bytes_downloaded, bytes_total);
                }
                DownloadEvent::Completed { id, file_path } => {
                    println!("Complete: {}", file_path);
                }
                _ => {}
            }
        }
    });

    Ok(())
}
```

## Testing

```bash
cargo test
```

## Development

See CLAUDE.md for detailed development instructions and TDD requirements.
