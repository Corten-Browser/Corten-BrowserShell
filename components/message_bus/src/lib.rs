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
//! - Message routing with deadline tracking
//! - Priority inversion prevention

mod bus;
pub mod priority;
pub mod threading;
mod types;

// Re-export public types
pub use bus::MessageBus;
pub use priority::{
    LaneStats, MessageRouter, PrioritizedMessage, PriorityQueue, PriorityQueueConfig,
    QueueError, QueueMetrics, RoutingTarget, SharedPriorityQueue,
};
pub use threading::{TaskResult, ThreadHandle, ThreadPool, ThreadPoolConfig, ThreadType};
pub use types::{ComponentMessage, ComponentResponse, MessagePriority};
