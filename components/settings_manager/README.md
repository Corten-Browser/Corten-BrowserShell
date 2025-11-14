# Settings Manager Component

## Overview

The **settings_manager** component manages user settings and preferences for the CortenBrowser Browser Shell. It provides persistent storage, synchronization, and default settings management using YAML format.

## Features

- **Type-Safe Settings**: Support for String, Integer, Float, and Boolean values
- **Persistent Storage**: Automatic save/load to YAML configuration files
- **Default Settings**: Comprehensive defaults based on browser shell specification
- **Async API**: All operations are asynchronous for non-blocking I/O
- **Thread-Safe**: Concurrent access supported through internal locking
- **Graceful Fallbacks**: Handles missing or corrupted files gracefully

## Setup

Add to your `Cargo.toml`:

```toml
[dependencies]
settings_manager = { path = "../settings_manager" }
```

## Usage

### Basic Usage

```rust
use settings_manager::{SettingsManager, SettingValue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new settings manager with defaults
    let manager = SettingsManager::new();

    // Get a setting
    let width = manager.get_setting("window.default_width").await?;
    match width {
        SettingValue::Integer(w) => println!("Window width: {}", w),
        _ => println!("Unexpected type"),
    }

    // Set a setting
    manager.set_setting(
        "ui.theme".to_string(),
        SettingValue::String("dark".to_string())
    ).await?;

    // Save to disk
    manager.save().await?;

    Ok(())
}
```

### Get All Settings

```rust
let all_settings = manager.get_all_settings().await?;

for (key, value) in all_settings {
    println!("{}: {:?}", key, value);
}
```

### Reset to Defaults

```rust
// Reset all settings to their default values
manager.reset_to_defaults().await?;
```

### Custom Configuration Directory

```rust
use std::path::PathBuf;

let config_dir = PathBuf::from("/custom/config/path");
let manager = SettingsManager::with_config_dir(config_dir);
```

## API

See full API documentation in the source code documentation.

## Default Settings

The component provides comprehensive defaults based on the browser shell specification (Appendix B), including window, tab, UI, performance, security, network, privacy, and developer settings.

## Development

### Running Tests

```bash
cargo test
```

### Code Quality

```bash
cargo fmt
cargo clippy
```

## Testing

- **44 tests total**
- **100% pass rate**
- Comprehensive coverage including unit, integration, and contract tests

## License

Part of the CortenBrowser Browser Shell project.
