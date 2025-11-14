# Message Bus Component

## Overview

The message bus component provides asynchronous inter-component communication for the CortenBrowser Browser Shell. It enables components to communicate through message passing without direct coupling, supporting both point-to-point messaging and broadcast patterns.

## Features

- **Component Registration**: Register components with the message bus for communication
- **Point-to-Point Messaging**: Send messages directly to specific components
- **Broadcast Messaging**: Send messages to all registered components
- **Message Subscriptions**: Components can subscribe to specific message types
- **Priority Support**: Messages can be prioritized (Critical, High, Normal, Low)
- **Async/Await**: Built on Tokio for efficient asynchronous operation
- **Type Safety**: Strong typing prevents message routing errors

## Setup

Add this component as a dependency in your `Cargo.toml`:

```toml
[dependencies]
message_bus = { path = "../message_bus" }
shared_types = { path = "../shared_types" }
tokio = { version = "1.35", features = ["full"] }
```

## Usage

### Basic Example

```rust
use message_bus::{MessageBus, ComponentMessage, ComponentResponse};
use shared_types::WindowConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a message bus
    let mut bus = MessageBus::new()?;

    // Register components
    bus.register("window_manager".to_string(), "manager".to_string()).await?;
    bus.register("tab_manager".to_string(), "manager".to_string()).await?;

    // Send a message to a specific component
    let message = ComponentMessage::CreateWindow(WindowConfig::default());
    let response = bus.send("window_manager".to_string(), message).await?;

    // Broadcast a message to all components
    let broadcast_msg = ComponentMessage::KeyboardShortcut(
        shared_types::KeyboardShortcut::CtrlT
    );
    bus.broadcast(broadcast_msg).await?;

    Ok(())
}
```

### Message Subscription

```rust
use message_bus::{MessageBus, ComponentMessage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::new()?;

    // Register a component
    bus.register("ui_chrome".to_string(), "ui".to_string()).await?;

    // Subscribe to specific message types
    bus.subscribe("ui_chrome".to_string(), "CreateWindow".to_string()).await?;
    bus.subscribe("ui_chrome".to_string(), "UpdateTitle".to_string()).await?;

    // Now ui_chrome will only receive CreateWindow and UpdateTitle messages
    // when using filtered broadcast (future enhancement)

    Ok(())
}
```

## API

### MessageBus

The main message bus interface.

#### Methods

- `new() -> Result<MessageBus, ComponentError>` - Create a new message bus instance
- `register(component_id: String, component_type: String) -> Result<(), ComponentError>` - Register a component
- `send(target: String, message: ComponentMessage) -> Result<ComponentResponse, ComponentError>` - Send to specific component
- `broadcast(message: ComponentMessage) -> Result<(), ComponentError>` - Broadcast to all components
- `subscribe(component_id: String, message_type: String) -> Result<(), ComponentError>` - Subscribe to message types

### Message Types

#### ComponentMessage

Messages that can be sent between components:

- `CreateWindow(WindowConfig)` - Create a new browser window
- `CloseWindow(WindowId)` - Close a window
- `CreateTab(WindowId, Option<String>)` - Create a new tab with optional URL
- `CloseTab(TabId)` - Close a tab
- `NavigateTab(TabId, String)` - Navigate tab to URL
- `UpdateAddressBar(TabId, String)` - Update address bar text
- `UpdateTitle(TabId, String)` - Update tab title
- `KeyboardShortcut(KeyboardShortcut)` - Handle keyboard shortcut

#### ComponentResponse

Responses from message handlers:

- `WindowCreated(WindowId)` - Window was created successfully
- `TabCreated(TabId)` - Tab was created successfully
- `NavigationStarted(TabId)` - Navigation started
- `Success` - Operation succeeded
- `Error(String)` - Operation failed with error message

#### MessagePriority

Message processing priority levels:

- `Critical` - Highest priority (system-critical operations)
- `High` - High priority (user-initiated actions)
- `Normal` - Normal priority (default)
- `Low` - Lowest priority (background tasks)

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_message_bus_creation

# Run with coverage
cargo tarpaulin --out Html
```

### Code Quality

This component follows strict quality standards:

- **Test Coverage**: Target 80%+ (currently meeting target)
- **TDD**: All features developed test-first
- **Documentation**: All public APIs fully documented
- **Async Safety**: Proper use of Tokio async primitives
- **Error Handling**: Comprehensive error types

### Building

```bash
# Build the component
cargo build

# Build with optimizations
cargo build --release

# Check without building
cargo check
```

## Architecture

### Design Principles

1. **Loose Coupling**: Components don't need to know about each other
2. **Async-First**: Built on Tokio for efficient async operations
3. **Type Safety**: Strong typing prevents routing errors
4. **Testability**: Easy to test components in isolation
5. **Scalability**: Supports many components with low overhead

### Internal Structure

```
message_bus/
├── src/
│   ├── lib.rs          # Public API exports
│   ├── bus.rs          # MessageBus implementation
│   └── types.rs        # Message and response types
├── tests/
│   ├── unit/           # Unit tests
│   │   ├── test_message_types.rs
│   │   └── test_message_bus.rs
│   └── integration/    # Integration tests
│       └── test_message_flow.rs
├── Cargo.toml          # Dependencies and metadata
└── README.md           # This file
```

### Message Flow

```
Component A                MessageBus              Component B
    |                          |                        |
    |-- register() ----------->|                        |
    |                          |<----- register() ------|
    |                          |                        |
    |-- send(B, msg) --------->|                        |
    |                          |-- forward(msg) ------->|
    |                          |<----- response --------|
    |<----- response ----------|                        |
    |                          |                        |
    |-- broadcast(msg) ------->|                        |
    |                          |-- forward(msg) ------->|
    |                          |-- forward(msg) ------->| (all components)
```

## Error Handling

All operations return `Result` types with `ComponentError`:

```rust
use message_bus::MessageBus;
use shared_types::ComponentError;

async fn example() -> Result<(), ComponentError> {
    let mut bus = MessageBus::new()?;

    // Handle registration errors
    if let Err(e) = bus.register("component".to_string(), "type".to_string()).await {
        match e {
            ComponentError::InvalidState(msg) => {
                eprintln!("Component already registered: {}", msg);
            }
            _ => {
                eprintln!("Registration failed: {}", e);
            }
        }
    }

    Ok(())
}
```

## Performance Considerations

- **Lock Contention**: Uses `RwLock` for read-heavy workloads
- **Channel Buffering**: Unbounded channels prevent blocking (monitor memory)
- **Async Overhead**: Minimal async overhead using Tokio
- **Message Cloning**: Messages are cloned for broadcast (consider Arc for large payloads)

## Future Enhancements

- Message priority queue implementation
- Filtered broadcast based on subscriptions
- Message persistence and replay
- Dead letter queue for failed messages
- Metrics and monitoring
- Message batching for efficiency

## Testing Strategy

Tests are organized by type:

- **Unit Tests**: Test individual functions and types in isolation
- **Integration Tests**: Test complete message flows
- **Contract Tests**: Verify API contract compliance

See `tests/` directory for comprehensive test examples.

## License

MIT OR Apache-2.0

## Contributing

This component is part of the CortenBrowser Browser Shell project. Follow the project's contribution guidelines and coding standards.
