# Browser Shell Architecture

**Project**: Corten-BrowserShell
**Version**: 0.17.0
**Date**: 2025-11-13
**Status**: Pre-release

## Overview

The Browser Shell is the primary user interface and orchestration layer of CortenBrowser. It coordinates window management, tab management, browser chrome rendering, and all component communication through a central message bus.

**Target Implementation**: 50,000-75,000 lines of Rust code
**Component Count**: 8 components
**Timeline**: 6-8 weeks

## Technology Stack

- **Language**: Rust (edition 2021)
- **UI Framework**: egui or iced (pure Rust)
- **Current Implementation**: Tauri with WRY (WebView Rendering)
- **Migration Strategy**: Gradual replacement maintaining API compatibility
- **Platform Support**: Linux, Windows, macOS, Web (WASM)

## Component Architecture

The system is decomposed into 8 components organized in 4 layers:

### Layer 0: Base Components (No Dependencies)

#### 1. shared_types (~3,000-5,000 LOC)
**Token Budget**: 30,000-50,000 tokens
**Responsibility**: Common types and interfaces

**Contents**:
- Type definitions: `WindowId`, `TabId`, `Url` wrappers
- Component interface trait: `BrowserComponent`
- Message protocol definitions
- Error type hierarchy
- Configuration structures
- Serialization support (serde)

**Key Types**:
```rust
pub trait BrowserComponent: Send + Sync;
pub enum ComponentMessage;
pub enum ComponentResponse;
pub struct WindowConfig;
pub struct TabConfig;
pub struct ComponentConfig;
```

**Dependencies**: None
**Provides Public API**: Yes

---

### Layer 1: Core Components (Depend on Base)

#### 2. message_bus (~6,000-8,000 LOC)
**Token Budget**: 60,000-80,000 tokens
**Responsibility**: Inter-component communication

**Contents**:
- Message routing and dispatch
- Async message processing with tokio
- Component registry
- Message validation and security
- Priority-based message queue
- Broadcast and targeted messaging

**Key Features**:
- Thread-safe message dispatch
- Priority queuing (Critical, High, Normal, Low)
- Message size validation (security)
- Component health monitoring
- Metrics collection

**Dependencies**: shared_types
**Provides Public API**: Yes

#### 3. platform_abstraction (~8,000-10,000 LOC)
**Token Budget**: 80,000-100,000 tokens
**Responsibility**: Platform-specific functionality

**Contents**:
- Platform-specific window implementations
  - Linux (X11rb, Wayland)
  - Windows (Win32 API)
  - macOS (Cocoa, AppKit)
- Clipboard integration
- System notifications
- File system integration
- Platform event translation

**Key Modules**:
- `platform/linux.rs` (X11rb)
- `platform/windows.rs` (Win32)
- `platform/macos.rs` (Cocoa)
- `platform/notifications.rs`
- `platform/clipboard.rs`

**Dependencies**: shared_types
**Provides Public API**: Yes (cross-platform abstraction)

---

### Layer 2: Feature Components (Depend on Base + Core)

#### 4. window_manager (~8,000-10,000 LOC)
**Token Budget**: 80,000-100,000 tokens
**Responsibility**: Window lifecycle management

**Contents**:
- Window creation and destruction
- Multi-window support
- Window state management (size, position, fullscreen)
- Window events handling
- Platform window integration

**Key Features**:
- Create/close windows
- Resize/move/focus windows
- Fullscreen mode
- Window state persistence
- Multi-monitor support

**Dependencies**: shared_types, message_bus, platform_abstraction
**Provides Public API**: Yes (WindowManager trait)

#### 5. tab_manager (~8,000-10,000 LOC)
**Token Budget**: 80,000-100,000 tokens
**Responsibility**: Tab lifecycle and isolation

**Contents**:
- Tab creation and destruction
- Process isolation per tab
- Navigation history management
- Tab switching and state
- Tab restoration after crashes

**Key Features**:
- Per-tab process isolation
- Navigation history (back/forward)
- Tab lazy loading
- Crash recovery
- Session restore

**Dependencies**: shared_types, message_bus
**Provides Public API**: Yes (TabManager trait)

#### 6. ui_chrome (~10,000-12,000 LOC)
**Token Budget**: 100,000-120,000 tokens
**Responsibility**: Browser UI rendering

**Contents**:
- Address bar widget
- Tab bar with drag-and-drop
- Navigation toolbar (back, forward, reload, stop)
- Menu system
- Theme management (light/dark/auto)
- Keyboard shortcuts
- egui or iced framework integration

**Key Widgets**:
- `widgets/address_bar.rs` (URL input, suggestions)
- `widgets/tab_bar.rs` (tab strip with close buttons)
- `widgets/toolbar.rs` (navigation buttons)
- `widgets/menu.rs` (application menu)
- `theme.rs` (theming system)

**Dependencies**: shared_types, message_bus
**Provides Public API**: Yes (UI widgets API)

#### 7. user_data (~8,000-10,000 LOC)
**Token Budget**: 80,000-100,000 tokens
**Responsibility**: User data management

**Contents**:
- Settings management and persistence (SQLite)
- Bookmarks storage and UI
- Downloads manager and tracking
- History tracking and search
- Data synchronization support

**Key Modules**:
- `settings/` (preferences, storage, sync)
- `bookmarks/` (storage, UI, folders)
- `downloads/` (tracker, UI, resume)
- `history/` (storage, search, cleanup)

**Dependencies**: shared_types, message_bus
**Provides Public API**: Yes (Settings, Bookmarks, Downloads APIs)

---

### Layer 3: Integration Components (Orchestrate Everything)

#### 8. browser_shell (~6,000-8,000 LOC)
**Token Budget**: 60,000-80,000 tokens
**Responsibility**: Main orchestration and coordination

**Contents**:
- Component lifecycle management
- Public Browser Shell API
- State management (Arc<RwLock<...>>)
- Extension integration points
- Developer tools hosting
- Application entry point

**Key Responsibilities**:
- Initialize all components
- Coordinate component communication
- Expose unified API to extensions
- Handle application lifecycle (startup, shutdown)
- Manage global state

**Dependencies**: ALL components (this is the integration layer)
**Provides Public API**: Yes (main BrowserShellAPI)

---

## Dependency Graph

```
┌─────────────────┐
│  shared_types   │ (Level 0)
└────────┬────────┘
         │
    ┌────┴─────────────────────┐
    │                          │
┌───▼───────┐       ┌──────────▼──────────┐
│message_bus│       │platform_abstraction │ (Level 1)
└─────┬─────┘       └──────────┬──────────┘
      │                        │
   ┌──┴──────────────┬─────────┴────┬────────────┐
   │                 │              │            │
┌──▼──────────┐  ┌──▼────────┐  ┌──▼──────┐  ┌─▼────────┐
│window_manager│  │tab_manager│  │ui_chrome│  │user_data │ (Level 2)
└──────┬───────┘  └─────┬─────┘  └────┬────┘  └────┬─────┘
       │                │              │            │
       └────────────────┴──────────────┴────────────┘
                         │
                 ┌───────▼────────┐
                 │ browser_shell  │ (Level 3)
                 └────────────────┘
```

**Build Order**:
1. shared_types
2. message_bus, platform_abstraction (parallel)
3. window_manager, tab_manager, ui_chrome, user_data (parallel)
4. browser_shell

**No circular dependencies** - clean dependency hierarchy.

---

## Threading Model

```
┌─────────────────────────────────────────────────────────┐
│                    Browser Shell                         │
├─────────────────────────────────────────────────────────┤
│ UI Thread          │ Main rendering and user input       │
│ Message Bus Thread │ Routes component messages           │
│ IO Thread Pool     │ File I/O, settings, bookmarks       │
│ Network Thread     │ Network coordination                │
│ Per-Tab Threads    │ One render thread per tab (isolation)│
└─────────────────────────────────────────────────────────┘
```

**Thread Safety**:
- All shared state wrapped in `Arc<RwLock<T>>`
- Message passing via async channels
- Lock-free message queue where possible

---

## Component Size Constraints

| Component | Estimated LOC | Token Budget | Status |
|-----------|---------------|--------------|--------|
| shared_types | 3,000-5,000 | 30,000-50,000 | ✅ Within limits |
| message_bus | 6,000-8,000 | 60,000-80,000 | ✅ Within limits |
| platform_abstraction | 8,000-10,000 | 80,000-100,000 | ✅ Within limits |
| window_manager | 8,000-10,000 | 80,000-100,000 | ✅ Within limits |
| tab_manager | 8,000-10,000 | 80,000-100,000 | ✅ Within limits |
| ui_chrome | 10,000-12,000 | 100,000-120,000 | ⚠️ Near limit |
| user_data | 8,000-10,000 | 80,000-100,000 | ✅ Within limits |
| browser_shell | 6,000-8,000 | 60,000-80,000 | ✅ Within limits |
| **TOTAL** | **57,000-73,000** | **570,000-730,000** | ✅ On target |

**Note**: ui_chrome is the largest component at 100-120k tokens (near the 120k hard limit). Will monitor during development and split if needed.

**Limits**:
- Optimal: 80,000 tokens (~8,000 LOC)
- Warning: 100,000 tokens (~10,000 LOC)
- Hard limit: 120,000 tokens (~12,000 LOC)

---

## API Contracts

All components expose contracts via Rust traits:

```rust
// shared_types provides base traits
pub trait BrowserComponent: Send + Sync {
    async fn initialize(&mut self, config: ComponentConfig) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    async fn handle_message(&mut self, msg: ComponentMessage) -> Result<ComponentResponse>;
    fn health_check(&self) -> ComponentHealth;
}

// Each component implements specific traits
pub trait WindowManager: Send + Sync { ... }
pub trait TabManager: Send + Sync { ... }
pub trait MessageBus: Send + Sync { ... }
pub trait SettingsManager: Send + Sync { ... }
```

Contracts will be generated in Phase 3 after component structure is finalized.

---

## Security Model

1. **Process Isolation**: Each tab runs in a separate process
2. **Message Validation**: All IPC messages validated for size and content
3. **Input Sanitization**: All user input sanitized (URLs, text)
4. **Sandboxing**: Tab processes run in restricted sandbox
5. **Permission System**: Extensions require explicit permissions

---

## Performance Requirements

| Metric | Target | Maximum |
|--------|--------|---------|
| Window creation | < 100ms | 200ms |
| Tab creation | < 50ms | 100ms |
| Tab switching | < 10ms | 20ms |
| Message routing | < 1ms | 5ms |
| UI render frame | < 16ms | 33ms (60 FPS) |

---

## Testing Strategy

| Test Type | Coverage Target | Priority |
|-----------|----------------|----------|
| Unit Tests | 85% | High |
| Integration Tests | 70% | High |
| Contract Tests | 100% | Critical |
| UI Tests | 60% | Medium |

**Quality Gates** (all must pass):
- ✅ 100% test pass rate (unit, integration, contract, E2E)
- ✅ Test coverage ≥ 80%
- ✅ TDD compliance (git history shows Red-Green-Refactor)
- ✅ Zero linting errors
- ✅ Complexity ≤ 10 per function
- ✅ Contract compliance
- ✅ Security audit passed

---

## Development Phases

### Phase 1: Tauri-based Shell (Week 1)
- Basic window and tab management
- Address bar navigation
- Message bus implementation
- Integration with network stack

### Phase 2: egui Migration (Weeks 2-3)
- Replace Tauri UI with egui
- Browser chrome implementation
- Settings and downloads UI

### Phase 3: Pure Rust Shell (Weeks 4-6)
- Complete Rust implementation
- Direct render engine integration
- Extension system integration
- Developer tools hosting

### Phase 4: Advanced Features (Weeks 7-8)
- Multi-window drag-and-drop
- Picture-in-picture
- PWA support
- Sync system

---

## Migration Strategy

Current implementation uses Tauri/WRY (system WebView). Migration to pure Rust:

1. ✅ **Phase 1**: Keep Tauri, implement message bus and component structure
2. 🔄 **Phase 2**: Replace Tauri UI with egui, keep WebView for content
3. 🔄 **Phase 3**: Replace WebView with custom render engine
4. 🔄 **Phase 4**: Production-ready with all features

**API Compatibility**: All public APIs maintained throughout migration.

---

## Related Documentation

- [Specification](../browser-shell-specification.md) - Full technical specification
- [API Reference](./API.md) - Generated after Phase 2
- [Testing Guide](./TESTING.md) - Generated after Phase 3
- [Contributing Guide](./CONTRIBUTING.md) - Generated after Phase 4

---

**Last Updated**: 2025-11-13
**Next Review**: After Phase 2 completion
