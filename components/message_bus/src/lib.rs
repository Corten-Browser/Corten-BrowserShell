//! Message Bus Component
//!
//! Asynchronous inter-component message routing and delivery system for the CortenBrowser Browser Shell.
//!
//! This component provides:
//! - Component registration with the message bus
//! - Point-to-point message sending
//! - Broadcast messaging to all components
//! - Message type subscription system
//! - Priority-based message handling
//! - Multi-threaded architecture with dedicated thread pools

mod bus;
pub mod threading;
mod types;

// Re-export public types
pub use bus::MessageBus;
pub use threading::{ThreadPool, ThreadPoolConfig, ThreadType, ThreadHandle, TaskResult};
pub use types::{ComponentMessage, ComponentResponse, MessagePriority};
