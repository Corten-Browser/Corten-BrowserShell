# Extensions Component

Browser extension system for the CortenBrowser Browser Shell.

## Overview

This component provides a Chrome extension-compatible API for browser extensions, including:

- Extension registration and lifecycle management
- Browser action API (toolbar buttons with popups)
- Context menu contribution points
- Extension messaging (content scripts <-> background)
- Permission system for extension capabilities
- Manifest parsing (Chrome extension manifest v2/v3 compatible)

## Usage

```rust
use extensions::{ExtensionManager, ExtensionHost, Extension};

// Create the extension manager
let mut manager = ExtensionManager::new();

// Load an extension from manifest
let manifest_json = r#"{
    "name": "My Extension",
    "version": "1.0.0",
    "manifest_version": 3,
    "permissions": ["storage", "tabs"],
    "action": {
        "default_title": "Click me",
        "default_popup": "popup.html"
    }
}"#;

let extension_id = manager.load_from_manifest(manifest_json).await?;

// Enable the extension
manager.enable(extension_id).await?;

// Access extension APIs
let browser_action_api = manager.browser_action_api();
let context_menu_api = manager.context_menu_api();
let messaging_api = manager.messaging_api();
```

## API

### ExtensionManager

Central manager for all browser extensions.

- `new()` - Create a new manager
- `load_from_manifest(json)` - Load extension from manifest JSON
- `register(extension)` - Register an extension
- `unregister(id)` - Unregister an extension
- `enable(id)` - Enable an extension
- `disable(id)` - Disable an extension
- `get_extension(id)` - Get extension by ID
- `list_extensions()` - List all extension IDs

### BrowserActionApi

Manages toolbar buttons for extensions.

- `register(ext_id, action)` - Register browser action
- `unregister(ext_id)` - Remove browser action
- `set_badge_text(ext_id, text)` - Set badge text
- `set_badge_background_color(ext_id, color)` - Set badge color
- `visible_actions()` - Get visible toolbar actions

### ContextMenuApi

Manages context menu contributions.

- `add_item(ext_id, item)` - Add menu item
- `remove_item(ext_id, item_id)` - Remove menu item
- `get_items_for_context(context, url)` - Get items for context

### MessagingApi

Handles extension message passing.

- `register_channel(ext_id, sender)` - Register message channel
- `send(from, message)` - Send a message
- `send_and_wait(from, message, timeout)` - Send and wait for response

## Permissions

Supports Chrome extension permission model:

- `activeTab`, `storage`, `tabs`, `bookmarks`, `history`, etc.
- Host permissions (`https://example.com/*`)
- Optional permissions

## Architecture

```
extensions/
├── src/
│   ├── lib.rs           # Main module, ExtensionManager
│   ├── types.rs         # Core types (Extension, ExtensionId, etc.)
│   ├── permissions.rs   # Permission system
│   ├── browser_action.rs # Browser action API
│   ├── context_menu.rs  # Context menu API
│   ├── messaging.rs     # Extension messaging
│   └── manifest.rs      # Manifest parsing
└── tests/
```

## Development

```bash
# Run tests
cargo test -p extensions

# Check formatting
cargo fmt -p extensions -- --check

# Run clippy
cargo clippy -p extensions
```
