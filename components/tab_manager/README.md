# tab_manager

**Type**: feature
**Tech Stack**: Rust, tokio, url
**Actual LOC**: ~370 (src) + ~760 (tests)

## Responsibility

Tab lifecycle management, navigation, process isolation, and history tracking for the CortenBrowser Browser Shell.

## Features

- **Tab Lifecycle**: Create and close tabs with proper state management
- **Navigation**: Navigate to URLs with full history tracking
- **History Management**: Back/forward navigation with proper history stack
- **Reload**: Reload pages with optional cache bypass
- **Stop Loading**: Cancel in-progress page loads
- **Tab Information**: Query tab state and metadata

## Dependencies

- `shared_types` - Common types (TabId, WindowId, TabError, etc.)
- `tokio` - Async runtime
- `url` - URL parsing and validation
- `serde` - Serialization support
- `thiserror` - Error handling

## API

### TabManager

Main component providing tab management functionality:

```rust
pub struct TabManager {
    // Internal state
}

impl TabManager {
    pub fn new() -> Self;

    // Tab lifecycle
    pub async fn create_tab(&mut self, window_id: WindowId, url: Option<String>) -> Result<TabId, TabError>;
    pub async fn close_tab(&mut self, tab_id: TabId) -> Result<(), TabError>;

    // Navigation
    pub async fn navigate(&mut self, tab_id: TabId, url: String) -> Result<(), TabError>;
    pub async fn reload(&mut self, tab_id: TabId, ignore_cache: bool) -> Result<(), TabError>;
    pub async fn stop(&mut self, tab_id: TabId) -> Result<(), TabError>;

    // History
    pub async fn go_back(&mut self, tab_id: TabId) -> Result<(), TabError>;
    pub async fn go_forward(&mut self, tab_id: TabId) -> Result<(), TabError>;

    // Query
    pub fn get_tab_info(&self, tab_id: TabId) -> Option<TabInfo>;
}
```

### TabInfo

Public representation of tab state:

```rust
pub struct TabInfo {
    pub id: TabId,
    pub window_id: WindowId,
    pub title: String,
    pub url: Option<Url>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}
```

## Usage Example

```rust
use tab_manager::TabManager;
use shared_types::WindowId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    // Create a tab
    let tab_id = manager.create_tab(window_id, Some("https://example.com".to_string())).await?;

    // Navigate
    manager.navigate(tab_id, "https://example.org".to_string()).await?;

    // Go back
    manager.go_back(tab_id).await?;

    // Get tab info
    if let Some(info) = manager.get_tab_info(tab_id) {
        println!("Current URL: {:?}", info.url);
        println!("Can go back: {}", info.can_go_back);
        println!("Can go forward: {}", info.can_go_forward);
    }

    // Close tab
    manager.close_tab(tab_id).await?;

    Ok(())
}
```

## Structure

```
├── src/
│   └── lib.rs          # Main implementation
├── tests/
│   ├── unit/           # Unit tests
│   │   ├── test_tab.rs
│   │   └── test_tab_manager.rs
│   ├── contracts/      # Contract compliance tests
│   │   └── test_contract_compliance.rs
│   └── test_lib.rs     # Test entry point
├── Cargo.toml          # Dependencies
├── CLAUDE.md           # Component-specific instructions
└── README.md           # This file
```

## Testing

All tests follow TDD principles with comprehensive coverage:

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test test_lib

# Run with output
cargo test -- --nocapture

# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Test Coverage

- **Unit Tests**: 35 tests covering all functionality
- **Contract Tests**: Verify exact API compliance with contract specification
- **Test Coverage**: Estimated >85% based on comprehensive test suite
- **Test Types**:
  - Tab creation and lifecycle
  - Navigation and URL validation
  - History management (back/forward)
  - Reload and stop operations
  - Error handling for all edge cases
  - Multiple tabs per window

## Development

See CLAUDE.md for detailed development instructions, quality standards, and TDD requirements.

## Architecture

The component uses a simple but effective architecture:

- **TabManager**: Main facade managing all tabs
- **TabState**: Internal state combining Tab and NavigationHistory
- **NavigationHistory**: Stack-based history with current position tracking
- **TabInfo**: Public read-only view of tab state

History management uses a standard browser history model:
- New navigations truncate forward history
- Back/forward navigation updates current position
- History tracks URLs and titles

This component is part of the CortenBrowser Browser Shell project.
