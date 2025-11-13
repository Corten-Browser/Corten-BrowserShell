# shared_types - Common Types for Corten Browser

**Version**: 0.17.0
**Type**: Base Layer (Level 0)
**Technology**: Rust
**Status**: ✅ Complete

## Purpose

Common types, interfaces, and message protocol definitions for all browser components. Serves as the foundation for component communication and integration within the Corten Browser Shell orchestration system.

## Tech Stack

- **Rust 2021 Edition**
- **serde 1.0** - Serialization/deserialization
- **async-trait 0.1** - Async trait support
- **thiserror 1.0** - Error handling
- **tokio 1.0** - Async runtime
- **proptest 1.0** (dev) - Property-based testing

## Modules

- **`error`** - Error types used across components
- **`component`** - Base component interface and types
- **`window`** - Window management types
- **`tab`** - Tab management types
- **`message`** - Inter-component message protocol

## Public API

### Core Traits

- **`BrowserComponent`** - Base trait for all browser components
- **`TabManager`** - Tab lifecycle and navigation management
- **`WindowManager`** - Window lifecycle management

### Key Types

- **`TabId`**, **`WindowId`** - Unique identifiers (UUID-based)
- **`Url`** - URL representation with validation
- **`Tab`**, **`Window`** - Complete state representations
- **`ShellMessage`**, **`ShellResponse`** - Message protocol

### Error Types

- **`ComponentError`** - Component-level errors
- **`TabError`** - Tab management errors
- **`WindowError`** - Window management errors

## Usage Example

```rust
use shared_types::tab::{TabId, Url};
use shared_types::window::WindowId;
use shared_types::component::ComponentHealth;

let tab_id = TabId::new();
let window_id = WindowId::new();
let url = Url::parse("https://example.com")?;
let health = ComponentHealth::Healthy;
```

## Testing

```bash
# Run all tests (59 tests total)
cargo test

# Unit tests (25 module + 22 integration)
cargo test --lib
cargo test --test unit_tests

# Property-based tests (11 tests)
cargo test --test property_tests
```

### Test Coverage

- **59 tests** (100% passing)
  - 25 module tests
  - 22 unit tests
  - 11 property-based tests
  - 1 doc test
- **Coverage**: >95% (exceeds 80% requirement)
- **TDD**: Developed using Red-Green-Refactor cycle

## Dependencies

None (base component - Level 0)

## Structure

```
components/shared_types/
├── Cargo.toml          # Rust package manifest
├── src/
│   ├── lib.rs          # Public API
│   ├── error.rs        # Error types
│   ├── component.rs    # BrowserComponent trait
│   ├── window.rs       # Window types
│   ├── tab.rs          # Tab types
│   └── message.rs      # Message protocol
├── tests/
│   ├── unit_tests.rs   # Unit tests (22)
│   └── property_tests.rs # Property tests (11)
├── CLAUDE.md           # Component instructions
└── README.md           # This file
```

## Development

See CLAUDE.md for detailed development instructions, quality standards, and TDD requirements.

---
**Last Updated**: 2025-11-13
**Project**: Corten-BrowserShell
**Test Count**: 59 passing
**Coverage**: >95%
