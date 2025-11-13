// @implements: REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006
//! Main MessageBus implementation

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use shared_types::ComponentMessage;
use crate::{
    router::MessageRouter,
    queue::PriorityQueue,
    validator::MessageValidator,
    error::Result,
};

/// Main message bus that coordinates routing, queuing, and validation
pub struct MessageBus {
    router: Arc<RwLock<MessageRouter>>,
    queue: Arc<RwLock<PriorityQueue>>,
    validator: MessageValidator,
}

impl MessageBus {
    /// Create a new message bus
    ///
    /// # Arguments
    /// * `queue_capacity` - Maximum number of messages in the priority queue
    /// * `max_message_size` - Maximum message size in bytes
    pub fn new(queue_capacity: usize, max_message_size: usize) -> Self {
        Self {
            router: Arc::new(RwLock::new(MessageRouter::new())),
            queue: Arc::new(RwLock::new(PriorityQueue::new(queue_capacity))),
            validator: MessageValidator::new(max_message_size),
        }
    }

    /// Register a component
    pub async fn register_component(
        &self,
        component_id: String,
        sender: mpsc::Sender<ComponentMessage>,
    ) -> Result<()> {
        let mut router = self.router.write().await;
        router.register_component(component_id, sender).await
    }

    /// Deregister a component
    pub async fn deregister_component(&self, component_id: &str) -> Result<()> {
        let mut router = self.router.write().await;
        router.deregister_component(component_id).await
    }

    /// Route a message immediately (without queuing)
    pub async fn route_message(&self, message: ComponentMessage) -> Result<()> {
        // Validate message
        self.validator.validate(&message)?;

        // Route directly
        let router = self.router.read().await;
        router.route(message).await
    }

    /// Enqueue a message for later processing
    pub async fn enqueue_message(&self, message: ComponentMessage) -> Result<()> {
        // Validate message
        self.validator.validate(&message)?;

        // Add to queue
        let mut queue = self.queue.write().await;
        queue.push(message).await
    }

    /// Process the next message from the queue
    pub async fn process_next(&self) -> Result<()> {
        // Get highest priority message
        let mut queue = self.queue.write().await;
        let message = queue.pop().await?;
        drop(queue);

        // Route it
        let router = self.router.read().await;
        router.route(message).await
    }

    /// Get the current queue length
    pub async fn queue_length(&self) -> usize {
        let queue = self.queue.read().await;
        queue.len().await
    }

    /// Check if a component is registered
    pub async fn is_component_registered(&self, component_id: &str) -> bool {
        let router = self.router.read().await;
        router.is_registered(component_id).await
    }

    /// Get health metrics
    pub async fn get_metrics(&self) -> BusMetrics {
        let router = self.router.read().await;
        let queue = self.queue.read().await;

        BusMetrics {
            registered_components: router.component_count().await,
            queued_messages: queue.len().await,
        }
    }
}

/// Message bus metrics
#[derive(Debug, Clone)]
pub struct BusMetrics {
    pub registered_components: usize,
    pub queued_messages: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::{MessageTarget, MessagePriority, MessagePayload};

    #[tokio::test]
    async fn bus_routes_message() {
        let bus = MessageBus::new(100, 1024 * 1024);
        let (tx, mut rx) = mpsc::channel(10);

        bus.register_component("comp1".to_string(), tx).await.unwrap();

        let msg = ComponentMessage {
            id: "msg1".to_string(),
            source: "source".to_string(),
            target: MessageTarget::Component("comp1".to_string()),
            timestamp: 1000,
            priority: MessagePriority::Normal,
            payload: MessagePayload::HealthCheck,
        };

        bus.route_message(msg.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, msg.id);
    }

    #[tokio::test]
    async fn bus_validates_messages() {
        let bus = MessageBus::new(100, 1024 * 1024);

        let invalid_msg = ComponentMessage {
            id: "".to_string(), // Empty ID
            source: "source".to_string(),
            target: MessageTarget::Broadcast,
            timestamp: 1000,
            priority: MessagePriority::Normal,
            payload: MessagePayload::HealthCheck,
        };

        let result = bus.route_message(invalid_msg).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn bus_enqueues_and_processes() {
        let bus = MessageBus::new(100, 1024 * 1024);
        let (tx, mut rx) = mpsc::channel(10);

        bus.register_component("comp1".to_string(), tx).await.unwrap();

        let msg = ComponentMessage {
            id: "msg1".to_string(),
            source: "source".to_string(),
            target: MessageTarget::Component("comp1".to_string()),
            timestamp: 1000,
            priority: MessagePriority::Normal,
            payload: MessagePayload::HealthCheck,
        };

        // Enqueue
        bus.enqueue_message(msg.clone()).await.unwrap();
        assert_eq!(bus.queue_length().await, 1);

        // Process
        bus.process_next().await.unwrap();
        assert_eq!(bus.queue_length().await, 0);

        // Verify received
        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, msg.id);
    }
}
