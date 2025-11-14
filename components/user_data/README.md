# user_data Component

**Type**: Feature (Level 2)
**Tech Stack**: Rust, SQLite (rusqlite)
**Version**: 0.17.0

## Purpose

User data management for the Corten browser, providing:
- **Settings persistence** - Key-value storage with SQLite backend ✅
- **Bookmarks management** - Hierarchical bookmarks with folders (stub)
- **Downloads tracking** - Download state tracking with resume support (stub)
- **History storage** - Browsing history with search capability (stub)

## Implementation Status

### ✅ Complete: Settings Manager

Fully implemented with comprehensive test coverage:
- Key-value storage using SQLite
- CRUD operations (set, get, delete, list_all, clear_all)
- In-memory database support for testing
- Defensive input validation
- Timestamp tracking for updates
- Persistence across instances

**Tests**: 17 passing (11 integration + 6 unit)
**Coverage**: 100% of Settings module

### 🚧 Future: Bookmarks, Downloads, History

These modules have stub implementations and will be completed in future iterations.

## Structure

```
components/user_data/
├── src/
│   ├── lib.rs              # Main library interface
│   ├── settings/
│   │   ├── mod.rs          # Settings Manager API
│   │   └── storage.rs      # SQLite storage backend
│   ├── bookmarks/mod.rs    # Stub
│   ├── downloads/mod.rs    # Stub
│   └── history/mod.rs      # Stub
├── tests/
│   ├── settings_tests.rs           # Settings integration tests
│   ├── complete_integration.rs     # Full workflow tests
│   └── unit/test_settings.rs       # Additional unit tests
├── Cargo.toml
└── README.md
```

## Usage

### Settings Manager

```rust
use user_data::SettingsManager;
use rusqlite::Connection;

// Create with persistent database
let conn = Connection::open("user_data.db")?;
let mut settings = SettingsManager::new(conn)?;

// Set settings
settings.set("theme", "dark")?;
settings.set("language", "en")?;

// Get settings
let theme = settings.get("theme")?; // Some("dark")

// List all settings
let all = settings.list_all()?;

// Delete setting
settings.delete("theme")?;

// Clear all
settings.clear_all()?;
```

### In-Memory Database (Testing)

```rust
let conn = Connection::open_in_memory()?;
let mut settings = SettingsManager::new(conn)?;
// Settings are stored only in memory
```

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_settings_manager_creation

# Check coverage
cargo tarpaulin
```

## Test Coverage

**Current**: >80% (Settings module fully covered)

All Settings Manager functionality is tested:
- ✅ Creation and initialization
- ✅ CRUD operations
- ✅ Edge cases (empty keys, nonexistent keys)
- ✅ Persistence across instances
- ✅ Concurrent access safety
- ✅ Error handling

## Development

This component follows TDD principles:
1. **RED**: Write failing tests first
2. **GREEN**: Implement minimum code to pass
3. **REFACTOR**: Clean up and optimize

See CLAUDE.md for detailed development instructions and quality standards.

## Dependencies

- `rusqlite` (0.30) - SQLite database
- `serde` (1.0) - Serialization
- `thiserror` (1.0) - Error handling
- `shared_types` (0.17.0) - Common types
- `tempfile` (3.8) - Test utilities (dev only)

## API Documentation

See `component.yaml` for public API declaration and module documentation in source files.

---
**Last Updated**: 2025-11-13
**Project**: Corten-BrowserShell
