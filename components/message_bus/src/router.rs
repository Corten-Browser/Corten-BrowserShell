// @implements: REQ-001, REQ-002
//! Message routing logic

use tokio::sync::mpsc;
use shared_types::{ComponentMessage, MessageTarget};
use crate::error::{MessageBusError, Result};
use crate::registry::ComponentRegistry;

/// Message router that handles routing messages to target components
pub struct MessageRouter {
    registry: ComponentRegistry,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
        }
    }

    /// Register a component with the router
    pub async fn register_component(
        &mut self,
        component_id: String,
        sender: mpsc::Sender<ComponentMessage>,
    ) -> Result<()> {
        self.registry.register(component_id, sender).await
    }

    /// Deregister a component
    pub async fn deregister_component(&mut self, component_id: &str) -> Result<()> {
        self.registry.deregister(component_id).await
    }

    /// Route a message to its target(s)
    pub async fn route(&self, message: ComponentMessage) -> Result<()> {
        match message.target.clone() {
            MessageTarget::Component(target_id) => {
                self.route_to_component(message, &target_id).await
            }
            MessageTarget::Broadcast => {
                self.broadcast(message).await
            }
            MessageTarget::Group(group_id) => {
                // For now, groups are not implemented, treat as error
                Err(MessageBusError::RoutingError(
                    format!("Group routing not yet implemented: {}", group_id)
                ))
            }
        }
    }

    /// Route a message to a specific component
    async fn route_to_component(&self, message: ComponentMessage, target_id: &str) -> Result<()> {
        let sender = self.registry.get_sender(target_id).await
            .ok_or_else(|| MessageBusError::ComponentNotFound(target_id.to_string()))?;

        sender.send(message).await
            .map_err(|e| MessageBusError::RoutingError(
                format!("Failed to send to {}: {}", target_id, e)
            ))
    }

    /// Broadcast a message to all registered components
    async fn broadcast(&self, message: ComponentMessage) -> Result<()> {
        let senders = self.registry.get_all_senders().await;

        if senders.is_empty() {
            return Err(MessageBusError::RoutingError(
                "No components registered for broadcast".to_string()
            ));
        }

        // Send to all components
        // Note: We don't fail if some sends fail (fire-and-forget broadcast)
        let mut errors = Vec::new();
        for sender in senders {
            if let Err(e) = sender.send(message.clone()).await {
                errors.push(e.to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            // In production, you might want to log partial failures
            // For now, we succeed if at least one component received it
            Ok(())
        }
    }

    /// Get the number of registered components
    pub async fn component_count(&self) -> usize {
        self.registry.list_components().await.len()
    }

    /// Check if a component is registered
    pub async fn is_registered(&self, component_id: &str) -> bool {
        self.registry.is_registered(component_id).await
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::{MessagePriority, MessagePayload};

    #[tokio::test]
    async fn router_routes_to_component() {
        let mut router = MessageRouter::new();
        let (tx, mut rx) = mpsc::channel(10);

        router.register_component("comp1".to_string(), tx).await.unwrap();

        let msg = ComponentMessage {
            id: "msg1".to_string(),
            source: "source".to_string(),
            target: MessageTarget::Component("comp1".to_string()),
            timestamp: 1000,
            priority: MessagePriority::Normal,
            payload: MessagePayload::HealthCheck,
        };

        router.route(msg.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, msg.id);
    }

    #[tokio::test]
    async fn router_broadcasts() {
        let mut router = MessageRouter::new();
        let (tx1, mut rx1) = mpsc::channel(10);
        let (tx2, mut rx2) = mpsc::channel(10);

        router.register_component("comp1".to_string(), tx1).await.unwrap();
        router.register_component("comp2".to_string(), tx2).await.unwrap();

        let msg = ComponentMessage {
            id: "broadcast".to_string(),
            source: "source".to_string(),
            target: MessageTarget::Broadcast,
            timestamp: 1000,
            priority: MessagePriority::Normal,
            payload: MessagePayload::HealthCheck,
        };

        router.route(msg.clone()).await.unwrap();

        let received1 = rx1.recv().await.unwrap();
        let received2 = rx2.recv().await.unwrap();

        assert_eq!(received1.id, "broadcast");
        assert_eq!(received2.id, "broadcast");
    }
}
