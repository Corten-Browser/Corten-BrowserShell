# Browser Shell API Contracts

**Version**: 0.17.0
**Last Updated**: 2025-11-13

## Overview

This directory contains the API contracts (interface definitions) for all browser shell components. These contracts define the communication protocols and interfaces that components must implement.

## Contract Files

### browser_component.rs
Base interface that all browser components must implement:
- `BrowserComponent` trait - lifecycle, message handling, health checks
- `ComponentMessage` - generic message envelope
- `ComponentResponse` - generic response types
- `ComponentHealth` - health status reporting
- `ComponentMetrics` - performance metrics

### window_manager.rs
Window lifecycle management interface:
- `WindowManager` trait - window creation, updates, events
- `WindowId` - unique window identifier
- `WindowConfig` - window configuration
- `WindowUpdate` - window update operations
- `PlatformEvent` - platform-specific window events

### tab_manager.rs
Tab lifecycle and navigation interface:
- `TabManager` trait - tab creation, navigation, history
- `TabId` - unique tab identifier
- `Tab` - tab representation
- `Url` - URL type
- Navigation operations (back, forward, reload)

### message_protocol.rs
Complete browser shell message protocol:
- `ShellMessage` - all shell-specific messages
- `ShellResponse` - shell response types
- `MenuAction` - menu action types
- `KeyboardShortcut` - keyboard shortcut definitions
- `DownloadInfo` - download state

## Usage

Components import these contracts to implement required interfaces:

```rust
// In shared_types component
pub mod browser_component;
pub mod window_manager;
pub mod tab_manager;
pub mod message_protocol;

// In other components
use shared_types::browser_component::BrowserComponent;
use shared_types::window_manager::WindowManager;
```

## Contract Compliance

All components MUST:
1. Implement the `BrowserComponent` trait
2. Follow the message protocol defined in `message_protocol.rs`
3. Use the type definitions from contracts (WindowId, TabId, etc.)
4. Return errors as defined in contract error types

## Validation

Contract compliance is verified in Phase 4.5 (Contract Validation) using:
```bash
python orchestration/contract_enforcer.py check <component-name>
```

## Versioning

All contracts are versioned together with the project. Breaking changes to contracts require:
1. Version increment in all contract files
2. Update to all implementing components
3. Integration test updates

---

**Note**: These are Rust-like contract definitions. In actual implementation, these would be in the `shared_types` component as Rust traits and types.
