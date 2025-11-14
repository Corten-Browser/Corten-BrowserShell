# ui_chrome

**Type**: Feature Component
**Tech Stack**: Rust, egui (UI framework)
**Version**: 0.1.0
**Actual LOC**: ~590 (347 implementation + 244 tests)

## Overview

Browser chrome UI elements including address bar, toolbar, tab bar, and keyboard shortcuts. Provides the visual interface for browser navigation and tab management using the egui immediate-mode GUI framework.

## Features

- **Address Bar**: URL input with text editing and focus management
- **Navigation Toolbar**: Back, forward, and reload buttons
- **Tab Bar**: Multiple tab support with visual indicators
  - Tab creation and deletion
  - Active tab highlighting
  - Loading state indicators
- **Keyboard Shortcuts**:
  - `Ctrl+T`: New tab
  - `Ctrl+W`: Close tab (prevents closing last tab)
  - `Ctrl+L`: Focus address bar
  - `F5` / `Ctrl+R`: Reload page

## Usage

```rust
use ui_chrome::UiChrome;
use shared_types::KeyboardShortcut;

// Create UI chrome instance
let mut chrome = UiChrome::new();

// Handle address bar input
chrome.handle_address_bar_input("https://example.com".to_string())?;

// Create a new tab
let tab_id = chrome.add_tab("New Tab".to_string());

// Update tab title
chrome.update_tab_title(tab_id, "Example - Browser".to_string())?;

// Update loading state
chrome.update_loading_state(tab_id, true)?;

// Handle keyboard shortcuts
chrome.handle_keyboard_shortcut(KeyboardShortcut::CtrlT)?;

// Render UI (in your egui application)
chrome.render(&ctx)?;
```

## API

### `UiChrome`

Main browser chrome component managing UI state and rendering.

**Methods:**

- `new()` - Create new UI chrome with one default tab
- `render(&mut self, ctx: &egui::Context)` - Render UI using egui
- `handle_address_bar_input(&mut self, text: String)` - Update address bar
- `handle_keyboard_shortcut(&mut self, shortcut: KeyboardShortcut)` - Handle shortcuts
- `update_tab_title(&mut self, tab_id: TabId, title: String)` - Update tab title
- `update_loading_state(&mut self, tab_id: TabId, loading: bool)` - Update loading indicator
- `add_tab(&mut self, title: String)` - Add new tab
- `set_active_tab(&mut self, tab_id: TabId)` - Set active tab

### `TabState`

State for individual tabs.

**Fields:**
- `id: TabId` - Unique tab identifier
- `title: String` - Tab display title
- `loading: bool` - Loading indicator state

## Dependencies

- `egui` - Immediate-mode GUI framework for rendering
- `eframe` - Application framework for egui
- `shared_types` - Common types (TabId, KeyboardShortcut, ComponentError)
- `message_bus` - Inter-component messaging (for future integration)

## Testing

Comprehensive test suite with 15 tests covering:
- UI chrome creation and initialization
- Tab management (add, remove, switch)
- Title and loading state updates
- Address bar input handling
- All keyboard shortcuts
- Error cases (invalid tab IDs, closing last tab)

**Run tests:**
```bash
cargo test
```

**Test coverage:** >80% (all core functionality tested)

## Structure

```
ui_chrome/
├── src/
│   └── lib.rs              # Main implementation (347 lines)
├── tests/
│   └── test_ui_chrome.rs   # Integration tests (244 lines)
├── Cargo.toml              # Dependencies and metadata
├── CLAUDE.md               # Development instructions
└── README.md               # This file
```

## Development

Built following Test-Driven Development (TDD):
1. Tests written first (RED phase)
2. Implementation to pass tests (GREEN phase)
3. Code quality and structure (REFACTOR phase)

See `CLAUDE.md` for detailed development instructions, quality standards, and TDD requirements.

## Architecture

The UI chrome follows a clean separation of concerns:
- **State Management**: Tab state, address bar, active tab tracking
- **Input Handling**: Keyboard shortcuts, address bar input
- **Rendering**: egui-based UI with toolbar, tab bar, and content area
- **Integration**: Ready for message bus integration for cross-component communication

## Future Enhancements

Potential improvements (not yet implemented):
- Context menus for tabs
- Tab reordering (drag and drop)
- Bookmark bar
- Browser history integration
- Download manager UI
- Settings/preferences dialog

## License

Part of the CortenBrowser Browser Shell project.
