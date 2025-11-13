// @implements: REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006
//! Message Bus Library
//!
//! Inter-component message routing, dispatch, and async processing with priority queues.
//!
//! # Modules
//!
//! - `router`: Message routing logic
//! - `queue`: Priority-based message queue
//! - `registry`: Component registration and management
//! - `validator`: Message validation
//! - `bus`: Main MessageBus struct that ties everything together
//!
//! # Example
//!
//! ```rust
//! use message_bus::MessageBus;
//! use shared_types::{ComponentMessage, MessageTarget, MessagePriority, MessagePayload};
//!
//! #[tokio::main]
//! async fn main() {
//!     let bus = MessageBus::new(1000, 1024 * 1024);
//!
//!     // Register a component
//!     let (tx, mut rx) = tokio::sync::mpsc::channel(10);
//!     bus.register_component("my-component".to_string(), tx).await.unwrap();
//!
//!     // Route a message
//!     let msg = ComponentMessage {
//!         id: "msg-1".to_string(),
//!         source: "sender".to_string(),
//!         target: MessageTarget::Component("my-component".to_string()),
//!         timestamp: 1000,
//!         priority: MessagePriority::Normal,
//!         payload: MessagePayload::HealthCheck,
//!     };
//!
//!     bus.route_message(msg).await.unwrap();
//! }
//! ```

pub mod router;
pub mod queue;
pub mod registry;
pub mod validator;
pub mod bus;
pub mod error;

// Re-export main types
pub use bus::MessageBus;
pub use error::MessageBusError;
