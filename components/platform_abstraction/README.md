# platform_abstraction

Platform-specific window management implementations for the CortenBrowser Browser Shell.

**Type**: core
**Tech Stack**: Rust, x11rb/wayland (Linux), windows crate (Windows), cocoa (macOS)
**Phase**: 1 - Stub Implementation
**LOC**: ~500 (stub), ~12,000 (full implementation)

## Overview

This component provides a unified interface for creating and managing windows across Linux, Windows, and macOS platforms. It defines a common `PlatformWindow` trait and platform-specific implementations that abstract away OS-specific windowing details.

**Current Status**: Phase 1 - Stub Implementation

This is a stub implementation that compiles on all platforms and provides mock functionality for testing. Full native window integration (X11/Wayland, Win32, Cocoa) will be implemented in later phases.

## Features

- ✅ Cross-platform window abstraction via `PlatformWindow` trait
- ✅ Platform-specific handle types (`LinuxHandle`, `WindowsHandle`, `MacOSHandle`)
- ✅ Stub implementations that compile on all platforms
- ✅ Comprehensive test coverage (14 tests: 7 unit, 7 integration)
- ⏳ Full native windowing integration (planned for Phase 3)

## Dependencies

- `shared_types` - Common type definitions (WindowConfig, WindowError)
- `serde` - Serialization support for handle types
- `log` - Logging framework for operation tracing

## Structure

```
platform_abstraction/
├── src/
│   ├── lib.rs              # Main module exports
│   ├── handles.rs          # Platform handle type definitions
│   ├── traits.rs           # PlatformWindow trait definition
│   └── platform/
│       ├── mod.rs          # Platform module exports
│       ├── linux.rs        # Linux (X11/Wayland) stub
│       ├── windows.rs      # Windows (Win32) stub
│       └── macos.rs        # macOS (Cocoa/AppKit) stub
└── tests/
    └── integration_test.rs # Cross-platform integration tests
```

## Usage

### Basic Window Creation

```rust
use platform_abstraction::PlatformWindow;
use shared_types::WindowConfig;

// Platform-specific import based on target OS
#[cfg(target_os = "linux")]
use platform_abstraction::LinuxWindow as NativeWindow;

#[cfg(target_os = "windows")]
use platform_abstraction::WindowsWindow as NativeWindow;

#[cfg(target_os = "macos")]
use platform_abstraction::MacWindow as NativeWindow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a window with default configuration
    let config = WindowConfig::default();
    let mut window = NativeWindow::create(&config)?;

    // Show and manipulate the window
    window.show()?;
    window.resize(1920, 1080)?;
    window.move_to(100, 100)?;
    window.focus()?;

    // Clean up
    window.destroy()?;
    Ok(())
}
```

### Custom Window Configuration

```rust
let config = WindowConfig {
    title: "My Custom Window".to_string(),
    width: 1280,
    height: 720,
    x: Some(200),
    y: Some(150),
    fullscreen: false,
    resizable: true,
    decorations: true,
    always_on_top: false,
    skip_taskbar: false,
};

let window = NativeWindow::create(&config)?;
```

### Getting Platform Handle

```rust
match window.get_handle() {
    PlatformHandle::Linux(handle) => {
        println!("X11 Window ID: {}", handle.window);
    }
    PlatformHandle::Windows(handle) => {
        println!("HWND: 0x{:X}", handle.hwnd);
    }
    PlatformHandle::MacOS(handle) => {
        println!("NSWindow: 0x{:X}", handle.ns_window);
    }
}
```

## API

### PlatformWindow Trait

All platform implementations must implement this trait:

```rust
pub trait PlatformWindow {
    fn create(config: &WindowConfig) -> Result<Self, WindowError>;
    fn destroy(&mut self) -> Result<(), WindowError>;
    fn show(&mut self) -> Result<(), WindowError>;
    fn hide(&mut self) -> Result<(), WindowError>;
    fn resize(&mut self, width: u32, height: u32) -> Result<(), WindowError>;
    fn move_to(&mut self, x: i32, y: i32) -> Result<(), WindowError>;
    fn focus(&mut self) -> Result<(), WindowError>;
    fn get_handle(&self) -> PlatformHandle;
}
```

### Platform Handle Types

```rust
pub enum PlatformHandle {
    Linux(LinuxHandle),
    Windows(WindowsHandle),
    MacOS(MacOSHandle),
}

pub struct LinuxHandle {
    pub window: u32,  // X11/Wayland window ID
}

pub struct WindowsHandle {
    pub hwnd: usize,  // HWND as raw pointer
}

pub struct MacOSHandle {
    pub ns_window: usize,  // NSWindow pointer
}
```

## Development

### Running Tests

```bash
# Run all tests (unit + integration)
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run only integration tests
cargo test --test integration_test
```

**Test Results:**
- Unit tests: 7 passed
- Integration tests: 7 passed
- Total: 14 tests, 100% passing

### Linting and Formatting

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Check compilation
cargo check
```

## Contract Compliance

This component implements the contract defined in `contracts/platform_abstraction.yaml`:

- ✅ All `PlatformWindow` trait methods match contract signatures exactly
- ✅ All handle types match contract specifications
- ✅ Return types (Result<T, WindowError>) match contract
- ✅ Platform-specific implementations for Linux, Windows, macOS

## Phase Roadmap

### Phase 1 (✅ Current) - Stub Implementation
- ✅ Define `PlatformWindow` trait
- ✅ Create platform handle types
- ✅ Implement stub versions for all platforms
- ✅ Comprehensive test coverage (14 tests)
- ✅ Contract compliance verification

### Phase 2 - Basic Native Integration
- ⏳ Implement basic X11 window creation on Linux
- ⏳ Implement basic Win32 window creation on Windows
- ⏳ Implement basic Cocoa window creation on macOS

### Phase 3 - Full Feature Implementation
- ⏳ Complete window manipulation (resize, move, focus)
- ⏳ Window decorations and styles
- ⏳ Fullscreen and multi-monitor support
- ⏳ Native event handling

## Known Limitations (Stub Phase)

- Window operations are logged but don't create actual windows
- Platform handles are mock values (incrementing counters)
- No actual X11/Win32/Cocoa interaction
- Event handling not implemented

These are expected for Phase 1 and will be addressed in Phase 2+.

## Quality Metrics

- Test coverage: 100% of public API
- Tests passing: 14/14 (100%)
- Linting: cargo clippy passing
- Formatting: cargo fmt compliant
- Contract compliance: 100%

---

See CLAUDE.md for detailed development instructions, quality standards, and TDD requirements.

This component is part of the CortenBrowser Browser Shell project.
