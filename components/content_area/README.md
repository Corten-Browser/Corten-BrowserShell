# Content Area Component

**Version**: 0.4.0 (Part of Phase 5.1)
**Status**: Navigation API Complete, WebView Integration Pending
**Tests**: 14 tests, 100% passing

## Overview

The content_area component manages web content rendering for the browser shell. It provides navigation control, history management, and state tracking for each browser tab.

## Current Implementation (v0.4.0)

### ✅ Completed Features

- **Navigation API**: Navigate, go_back, go_forward, reload, stop
- **History Management**: Full forward/back history with truncation
- **State Tracking**: URL, title, loading status, navigation capabilities
- **Auto-URL Scheme**: Automatically adds https:// or http:// (localhost)
- **egui Rendering**: Placeholder UI showing navigation status
- **Comprehensive Tests**: 14 unit tests covering all functionality

### API Example

```rust
use content_area::ContentArea;

let mut content = ContentArea::new();

// Navigate to URL
content.navigate("example.com".to_string()).await?;

// Auto-adds https://
assert_eq!(content.current_url(), Some("https://example.com"));

// Navigation
content.navigate("rust-lang.org".to_string()).await?;
content.go_back().await?;
assert_eq!(content.current_url(), Some("https://example.com"));

// State queries
assert!(content.can_go_forward());
assert!(!content.is_loading());
println!("Title: {}", content.title());
```

## Pending Integration (Phase 5.1 continuation)

### WebView Integration

**Approach**: Use `wry` library for WebView rendering

**Requirements**:
- System dependencies: GTK3, WebKit2GTK (Linux), or platform equivalents
- Window coordination between wry (via tao) and eframe/egui
- Bi-directional communication (chrome ↔ content)

**Current Status**:
- `wry` dependency commented out (requires system libraries)
- Integration architecture designed but not yet implemented
- Placeholder rendering in egui shows navigation status

### Integration with Tab Manager

Each tab should have its own `ContentArea` instance. This requires:

1. **Modify `tab_manager::TabState`**:
   ```rust
   struct TabState {
       tab: Tab,
       history: NavigationHistory,
       content: Arc<RwLock<ContentArea>>,  // Add this
   }
   ```

2. **Wire navigation from `ui_chrome`**:
   - Address bar → tab_manager → active tab's content_area
   - Back/forward buttons → tab_manager → active tab's content_area

3. **Render content in UI**:
   - CentralPanel in shell_app → active tab's content_area.render()

### Dependencies for Full WebView

**Linux**:
```bash
sudo apt-get install libgtk-3-dev libwebkit2gtk-4.0-dev
```

**macOS**:
- Uses system WebKit (no additional dependencies)

**Windows**:
- Uses system WebView2 (no additional dependencies)

## Architecture

### Content Per Tab

```
BrowserShell
  ├── WindowManager
  └── TabManager
      ├── Tab 1 → ContentArea 1
      ├── Tab 2 → ContentArea 2
      └── Tab 3 → ContentArea 3
```

Each tab has its own:
- URL and navigation history
- Loading state
- WebView instance (when integrated)

### Message Flow

```
User types URL in address bar
  ↓
UiChrome emits Navigate message
  ↓
TabManager receives message
  ↓
ActiveTab's ContentArea.navigate()
  ↓
WebView navigates (when integrated)
  ↓
ContentArea updates state
  ↓
UiChrome displays updated state
```

## Testing

Run all tests:
```bash
cargo test -p content_area
```

Test coverage:
- Navigation (basic, with auto-scheme, localhost)
- History management (forward/back, truncation)
- State queries (URL, title, loading, can_go_back/forward)
- Error cases (empty URL, invalid navigation)
- Edge cases (reload without page, history boundaries)

## Next Steps

1. **Install system dependencies** (for WebView)
2. **Add wry dependency** (uncomment in Cargo.toml)
3. **Implement WebView window coordination**
4. **Wire into tab_manager** (one ContentArea per tab)
5. **Integrate with ui_chrome** (address bar navigation)
6. **Add content rendering** (replace placeholder)

## Files

- `src/lib.rs` - Main implementation (440 lines)
- `tests/` - Unit tests (14 tests)
- `Cargo.toml` - Dependencies (wry commented out)
- `README.md` - This file

## Known Limitations

- WebView not yet integrated (requires system dependencies)
- Placeholder rendering only (no actual web content)
- No JavaScript execution
- No DOM access
- No network request handling

These limitations will be addressed when WebView integration is completed in a development environment with proper system dependencies.

## License

MIT OR Apache-2.0 (matches workspace)
