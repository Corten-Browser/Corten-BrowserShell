# shared_types

**Type**: base
**Tech Stack**: Rust
**Actual LOC**: ~1,155 (425 src + 730 tests)
**Test Coverage**: 68 tests passing (100% pass rate)

## Overview

Common data structures, type definitions, and error types used across all browser shell components. This is a base-level component with no dependencies on other browser components.

## Responsibility

Provides strongly-typed wrappers and shared types for:
- Unique identifiers (WindowId, TabId, ProcessId, etc.)
- Configuration structures (WindowConfig)
- Keyboard shortcuts enumeration
- Error types with std::error::Error implementation

## Exported Types

### ID Types
- `WindowId` - Unique identifier for browser windows (UUID-based)
- `TabId` - Unique identifier for tabs (UUID-based)
- `ProcessId` - Operating system process identifier (u32-based)
- `RenderSurfaceId` - Identifier for render surfaces (UUID-based)
- `DownloadId` - Identifier for downloads (UUID-based)
- `BookmarkId` - Identifier for bookmarks (UUID-based)

### Configuration Types
- `WindowConfig` - Configuration for creating/updating browser windows
  - title, width, height, position (x, y)
  - fullscreen, resizable, decorations
  - always_on_top, skip_taskbar

### Keyboard Shortcuts
- `KeyboardShortcut` - Enumeration of supported keyboard shortcuts
  - CtrlT (New tab), CtrlW (Close tab), CtrlN (New window)
  - CtrlShiftT (Reopen closed tab), CtrlL (Focus address bar)
  - F5/CtrlR (Reload), CtrlShiftR (Hard reload)

### Error Types
All error types implement `std::error::Error`:
- `ComponentError` - Component initialization and operation errors
- `WindowError` - Window management errors
- `TabError` - Tab management errors

## Features

- All types are serializable/deserializable (via serde)
- All types implement Debug, Clone where appropriate
- All ID types use newtype pattern for type safety
- Comprehensive test coverage (unit tests + contract tests)

## Dependencies

**Runtime:**
- `serde` - Serialization/deserialization
- `uuid` - UUID generation for ID types
- `thiserror` - Error type derivation

**Development:**
- `serde_json` - JSON serialization for tests

## Usage Example

```rust
use shared_types::*;

// Create IDs
let window_id = WindowId::new();
let tab_id = TabId::new();

// Configure a window
let config = WindowConfig {
    title: "My Browser".to_string(),
    width: 1920,
    height: 1080,
    fullscreen: false,
    resizable: true,
    ..Default::default()
};

// Handle errors
match some_operation() {
    Ok(result) => { /* success */ },
    Err(WindowError::NotFound(id)) => {
        println!("Window {} not found", id.as_uuid());
    },
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

## Testing

Run tests:
```bash
cargo test -p shared_types
```

Build:
```bash
cargo build -p shared_types
cargo build -p shared_types --release
```

## Structure

```
├── src/
│   ├── lib.rs                 # Public API exports
│   ├── ids.rs                 # ID type definitions
│   ├── window_config.rs       # Window configuration
│   ├── keyboard_shortcut.rs   # Keyboard shortcut enum
│   └── errors.rs              # Error types
├── tests/
│   ├── unit/                  # Unit tests
│   │   ├── test_ids.rs
│   │   ├── test_window_config.rs
│   │   ├── test_keyboard_shortcut.rs
│   │   └── test_errors.rs
│   ├── contracts/             # Contract compliance tests
│   │   └── test_contract_compliance.rs
│   └── test_lib.rs           # Test runner
├── Cargo.toml
├── CLAUDE.md                  # Development instructions
└── README.md                  # This file
```

## Contract Compliance

This component implements all types specified in `contracts/shared_types.yaml`:
- ✅ All ID types (6 types)
- ✅ WindowConfig struct with all fields
- ✅ KeyboardShortcut enum with all variants
- ✅ All error types with std::error::Error implementation
- ✅ All types are serializable
- ✅ All types have required derives (Debug, Clone, etc.)

## Development

This component follows Test-Driven Development (TDD):
- Tests written before implementation
- 100% test pass rate (68/68 tests passing)
- Contract tests verify specification compliance
- All public API documented

See CLAUDE.md for detailed development instructions, quality standards, and TDD requirements.

## Version

**Current Version**: 0.1.0 (pre-release)

This component is part of the CortenBrowser Browser Shell project.
