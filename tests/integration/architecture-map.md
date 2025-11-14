# Component Architecture - CortenBrowser Browser Shell

## Overview

CortenBrowser Browser Shell is a modular browser implementation written in Rust. Components communicate through a message bus architecture with clear separation of concerns.

## Components

### Base Layer (Level 0)
**No dependencies on other components**

1. **shared_types** (Base)
   - Location: `components/shared_types/`
   - Purpose: Common data structures and types
   - Key Types: WindowId, TabId, WindowConfig, ComponentError, TabError
   - Dependencies: None

### Core Layer (Level 1)
**Depends only on Base components**

2. **message_bus** (Core)
   - Location: `components/message_bus/`
   - Purpose: Async message routing between browser components
   - Key Types: ComponentMessage, ComponentResponse, MessagePriority
   - Dependencies: shared_types
   - API:
     - `register(component_id, component_type)` - Register component
     - `send(target, message)` - Send to specific component
     - `broadcast(message)` - Broadcast to all
     - `subscribe(component_id, message_type)` - Subscribe to messages

3. **platform_abstraction** (Core)
   - Location: `components/platform_abstraction/`
   - Purpose: Platform-specific functionality abstraction
   - Dependencies: shared_types
   - Platforms: Linux, Windows, macOS

### Feature Layer (Level 2)
**Depends on Base and Core components**

4. **window_manager** (Feature)
   - Location: `components/window_manager/`
   - Purpose: Browser window lifecycle and management
   - Dependencies: shared_types, message_bus
   - API:
     - `create_window(config)` → WindowId
     - `close_window(id)`
     - `get_windows()` → Vec<WindowId>
     - `resize_window(id, width, height)`
     - `focus_window(id)`

5. **tab_manager** (Feature)
   - Location: `components/tab_manager/`
   - Purpose: Tab lifecycle and navigation management
   - Dependencies: shared_types, message_bus
   - API:
     - `create_tab(window_id, url)` → TabId
     - `close_tab(tab_id)`
     - `navigate(tab_id, url)`
     - `reload(tab_id, ignore_cache)`
     - `go_back(tab_id)`, `go_forward(tab_id)`
     - `get_tab_info(tab_id)` → TabInfo

6. **ui_chrome** (Feature)
   - Location: `components/ui_chrome/`
   - Purpose: Browser UI components (address bar, toolbar, etc.)
   - Dependencies: shared_types, message_bus

7. **settings_manager** (Feature)
   - Location: `components/settings_manager/`
   - Purpose: User settings and preferences management
   - Dependencies: shared_types
   - API:
     - `get_setting(key)` → SettingValue
     - `set_setting(key, value)`
     - `get_all_settings()` → HashMap
     - `reset_to_defaults()`
     - `save()`, `load()` - Persistence

8. **downloads_manager** (Feature)
   - Location: `components/downloads_manager/`
   - Purpose: Download management
   - Dependencies: shared_types

9. **bookmarks_manager** (Feature)
   - Location: `components/bookmarks_manager/`
   - Purpose: Bookmark storage and management
   - Dependencies: shared_types
   - API:
     - `add_bookmark(bookmark)` → BookmarkId
     - `remove_bookmark(id)`
     - `update_bookmark(id, bookmark)`
     - `get_bookmark(id)` → Option<Bookmark>
     - `search_bookmarks(query)` → Vec<Bookmark>

### Integration Layer (Level 3)
**Orchestrates multiple feature components**

10. **browser_shell** (Integration)
    - Location: `components/browser_shell/`
    - Purpose: Main orchestrator coordinating all browser components
    - Dependencies: ALL feature components + message_bus
    - API:
      - `initialize(config)` - Initialize all components
      - `shutdown()` - Graceful shutdown
      - `run()` - Start event loop
      - `new_window(config)` → WindowId
      - `new_tab(url)` → TabId
      - `navigate(url)`

### Application Layer (Level 4)
**Minimal entry point**

11. **shell_app** (Application)
    - Location: `components/shell_app/`
    - Purpose: CLI entry point and application bootstrap
    - Dependencies: browser_shell

## Component Dependencies Graph

```
shell_app (Level 4)
    └── browser_shell (Level 3)
        ├── window_manager (Level 2)
        │   ├── message_bus (Level 1)
        │   │   └── shared_types (Level 0)
        │   └── shared_types (Level 0)
        ├── tab_manager (Level 2)
        │   ├── message_bus (Level 1)
        │   └── shared_types (Level 0)
        ├── ui_chrome (Level 2)
        │   ├── message_bus (Level 1)
        │   └── shared_types (Level 0)
        ├── settings_manager (Level 2)
        │   └── shared_types (Level 0)
        ├── downloads_manager (Level 2)
        │   └── shared_types (Level 0)
        ├── bookmarks_manager (Level 2)
        │   └── shared_types (Level 0)
        └── message_bus (Level 1)

platform_abstraction (Level 1)
    └── shared_types (Level 0)
```

## Data Flows

### 1. Browser Startup Flow
```
shell_app → browser_shell.initialize()
    → window_manager.create_window()
        → platform_abstraction (create OS window)
        → message_bus.broadcast(WindowCreated)
    → tab_manager.create_tab()
        → message_bus.broadcast(TabCreated)
    → settings_manager.load()
    → bookmarks_manager initialization
```

### 2. New Window Creation Flow
```
User Input → browser_shell.new_window()
    → window_manager.create_window(config)
        → platform_abstraction (OS-specific window creation)
        → message_bus.send(target: "ui_chrome", WindowCreated)
    → ui_chrome updates UI
    → Result: WindowId returned
```

### 3. Tab Navigation Flow
```
User Input → browser_shell.navigate(url)
    → tab_manager.get_active_tab()
    → tab_manager.navigate(tab_id, url)
        → message_bus.broadcast(NavigateTab(tab_id, url))
    → ui_chrome.update_address_bar(url)
    → tab renders content
```

### 4. Settings Persistence Flow
```
User Changes Settings → browser_shell
    → settings_manager.set_setting(key, value)
    → settings_manager.save()
        → Writes to disk (user_data_dir/settings.json)
```

### 5. Bookmark Management Flow
```
User Adds Bookmark → browser_shell
    → bookmarks_manager.add_bookmark(bookmark)
        → Generates BookmarkId
        → message_bus.broadcast(BookmarkAdded)
    → ui_chrome updates bookmark UI
```

### 6. Message Bus Communication Pattern
```
Component A wants to notify Component B:
    1. Component A: message_bus.send(target: "component_b", message)
    2. Message Bus: Routes message to Component B
    3. Component B: Receives ComponentMessage
    4. Component B: Processes and responds
    5. Component A: Receives ComponentResponse
```

## Integration Points to Test

### Critical Component Pairs

1. **browser_shell → window_manager**
   - Window creation delegation
   - Window lifecycle management
   - Window configuration passing

2. **browser_shell → tab_manager**
   - Tab creation delegation
   - Navigation coordination
   - Tab state synchronization

3. **browser_shell → settings_manager**
   - Settings retrieval and updates
   - Settings persistence
   - Default settings initialization

4. **browser_shell → bookmarks_manager**
   - Bookmark CRUD operations
   - Bookmark search
   - Bookmark persistence

5. **window_manager → platform_abstraction**
   - OS-specific window operations
   - Platform-specific rendering
   - Native window events

6. **tab_manager → message_bus**
   - Tab navigation events
   - Tab state broadcasting
   - Component coordination

7. **message_bus → all components**
   - Message routing correctness
   - Broadcast functionality
   - Subscribe/notify pattern

## Key Contracts

### WindowConfig (shared_types)
```rust
struct WindowConfig {
    title: String,
    width: u32,
    height: u32,
    x: Option<i32>,
    y: Option<i32>,
    fullscreen: bool,
    resizable: bool,
    decorations: bool,
    always_on_top: bool,
    skip_taskbar: bool,
}
```

### TabInfo (tab_manager)
```rust
struct TabInfo {
    id: TabId,
    window_id: WindowId,
    title: String,
    url: Option<String>,
    loading: bool,
    can_go_back: bool,
    can_go_forward: bool,
}
```

### ComponentMessage (message_bus)
```rust
enum ComponentMessage {
    CreateWindow(WindowConfig),
    CloseWindow(WindowId),
    CreateTab(WindowId, Option<String>),
    CloseTab(TabId),
    NavigateTab(TabId, String),
    UpdateAddressBar(TabId, String),
    UpdateTitle(TabId, String),
    KeyboardShortcut(KeyboardShortcut),
}
```

### Bookmark (bookmarks_manager)
```rust
struct Bookmark {
    id: Option<BookmarkId>,
    url: String,
    title: String,
    folder: Option<String>,
    tags: Vec<String>,
    created_at: u64,
}
```

## Testing Strategy

### Unit Tests (Component Level)
Each component has its own unit tests in `components/*/tests/`

### Integration Tests (This Directory)
Test cross-component communication:
- Real component instantiation (no mocking)
- Actual message bus routing
- Real data flows between components
- Contract compliance verification

### E2E Tests
Complete user workflows:
- Browser startup to first page load
- Window creation → Tab creation → Navigation
- Settings persistence across restarts
- Bookmark add → save → load workflow

## Version Information

- Project Version: 0.1.0
- Language: Rust (async/await with tokio)
- Architecture: Message Bus with Component Isolation
- All components use semantic versioning
