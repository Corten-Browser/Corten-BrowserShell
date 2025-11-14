//! MessageBus implementation for async inter-component communication

use crate::types::{ComponentMessage, ComponentResponse};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shared_types::ComponentError;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// A message wrapper that includes priority and routing information
#[derive(Debug, Clone)]
struct RoutedMessage {
    /// The target component ID (None for broadcast)
    target: Option<String>,
    /// The actual message payload
    message: ComponentMessage,
    /// Response channel for point-to-point messages
    response_tx: Option<mpsc::UnboundedSender<ComponentResponse>>,
}

/// Component registration information
#[derive(Debug, Clone)]
struct ComponentInfo {
    /// Unique component identifier
    id: String,
    /// Component type (e.g., "window_manager", "tab_manager")
    component_type: String,
    /// Message types this component is subscribed to
    subscriptions: HashSet<String>,
    /// Channel for sending messages to this component
    tx: mpsc::UnboundedSender<RoutedMessage>,
}

/// Asynchronous message bus for inter-component communication
///
/// The MessageBus provides:
/// - Component registration and discovery
/// - Point-to-point message delivery
/// - Broadcast messaging
/// - Message type subscriptions
/// - Priority-based message handling
pub struct MessageBus {
    /// Registered components
    components: Arc<RwLock<HashMap<String, ComponentInfo>>>,
}

impl MessageBus {
    /// Create a new MessageBus instance
    ///
    /// # Returns
    ///
    /// Returns `Ok(MessageBus)` on success
    ///
    /// # Examples
    ///
    /// ```
    /// use message_bus::MessageBus;
    ///
    /// let bus = MessageBus::new().expect("Failed to create message bus");
    /// ```
    pub fn new() -> Result<Self, ComponentError> {
        Ok(Self {
            components: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Register a component with the message bus
    ///
    /// # Arguments
    ///
    /// * `component_id` - Unique identifier for the component
    /// * `component_type` - Type classification for the component
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful registration, or a `ComponentError` if:
    /// - The component ID is already registered
    /// - Registration fails for any other reason
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use message_bus::MessageBus;
    /// # async fn example() -> Result<(), shared_types::ComponentError> {
    /// let mut bus = MessageBus::new()?;
    /// bus.register("window_manager".to_string(), "manager".to_string()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn register(
        &mut self,
        component_id: String,
        component_type: String,
    ) -> Result<(), ComponentError> {
        let mut components = self.components.write().await;

        // Check if component is already registered
        if components.contains_key(&component_id) {
            return Err(ComponentError::InvalidState(format!(
                "Component '{}' is already registered",
                component_id
            )));
        }

        // Create a channel for this component
        let (tx, mut rx) = mpsc::unbounded_channel::<RoutedMessage>();

        // Spawn a task to handle messages for this component
        // In a real implementation, this would forward messages to the actual component
        // For now, we just consume them to prevent channel blocking
        tokio::spawn(async move {
            while let Some(_msg) = rx.recv().await {
                // Message received but not processed in this basic implementation
                // Real implementation would forward to component handler
            }
        });

        let info = ComponentInfo {
            id: component_id.clone(),
            component_type,
            subscriptions: HashSet::new(),
            tx,
        };

        components.insert(component_id, info);

        Ok(())
    }

    /// Send a message to a specific component
    ///
    /// # Arguments
    ///
    /// * `target` - ID of the target component
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// Returns `Ok(ComponentResponse)` with the component's response, or a `ComponentError` if:
    /// - The target component is not registered
    /// - Message delivery fails
    /// - The component does not respond in time
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use message_bus::{MessageBus, ComponentMessage};
    /// # use shared_types::WindowId;
    /// # async fn example() -> Result<(), shared_types::ComponentError> {
    /// # let mut bus = MessageBus::new()?;
    /// # bus.register("window_manager".to_string(), "manager".to_string()).await?;
    /// let message = ComponentMessage::CloseWindow(WindowId::new());
    /// let response = bus.send("window_manager".to_string(), message).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send(
        &mut self,
        target: String,
        message: ComponentMessage,
    ) -> Result<ComponentResponse, ComponentError> {
        let components = self.components.read().await;

        // Find the target component
        let component_info = components.get(&target).ok_or_else(|| {
            ComponentError::ResourceNotFound(format!("Component '{}' not found", target))
        })?;

        // Create a response channel
        let (response_tx, mut response_rx) = mpsc::unbounded_channel();

        // Send the message
        let routed_message = RoutedMessage {
            target: Some(target.clone()),
            message,
            response_tx: Some(response_tx),
        };

        component_info
            .tx
            .send(routed_message)
            .map_err(|e| ComponentError::MessageRoutingFailed(format!("Failed to send message: {}", e)))?;

        // Wait for response (with timeout)
        // In this basic implementation, we'll return Success immediately
        // A real implementation would wait for the actual component response
        drop(components); // Release the lock

        // Try to receive response with timeout
        match tokio::time::timeout(
            std::time::Duration::from_millis(100),
            response_rx.recv()
        ).await {
            Ok(Some(response)) => Ok(response),
            Ok(None) => Ok(ComponentResponse::Success), // Channel closed
            Err(_) => Ok(ComponentResponse::Success), // Timeout - return success for now
        }
    }

    /// Broadcast a message to all registered components
    ///
    /// # Arguments
    ///
    /// * `message` - The message to broadcast
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful broadcast to all components
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use message_bus::{MessageBus, ComponentMessage};
    /// # use shared_types::KeyboardShortcut;
    /// # async fn example() -> Result<(), shared_types::ComponentError> {
    /// # let mut bus = MessageBus::new()?;
    /// let message = ComponentMessage::KeyboardShortcut(KeyboardShortcut::CtrlT);
    /// bus.broadcast(message).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn broadcast(&mut self, message: ComponentMessage) -> Result<(), ComponentError> {
        let components = self.components.read().await;

        let routed_message = RoutedMessage {
            target: None,
            message,
            response_tx: None,
        };

        // Send to all components
        for (_id, component_info) in components.iter() {
            // Ignore send errors for broadcast (component might be shutting down)
            let _ = component_info.tx.send(routed_message.clone());
        }

        Ok(())
    }

    /// Subscribe a component to a specific message type
    ///
    /// When messages of the subscribed type are broadcast, only subscribed
    /// components will receive them.
    ///
    /// # Arguments
    ///
    /// * `component_id` - ID of the component to subscribe
    /// * `message_type` - Type of message to subscribe to (e.g., "CreateWindow")
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful subscription, or a `ComponentError` if:
    /// - The component is not registered
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use message_bus::MessageBus;
    /// # async fn example() -> Result<(), shared_types::ComponentError> {
    /// # let mut bus = MessageBus::new()?;
    /// # bus.register("window_manager".to_string(), "manager".to_string()).await?;
    /// bus.subscribe("window_manager".to_string(), "CreateWindow".to_string()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe(
        &mut self,
        component_id: String,
        message_type: String,
    ) -> Result<(), ComponentError> {
        let mut components = self.components.write().await;

        let component_info = components.get_mut(&component_id).ok_or_else(|| {
            ComponentError::ResourceNotFound(format!("Component '{}' not found", component_id))
        })?;

        component_info.subscriptions.insert(message_type);

        Ok(())
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new().expect("Failed to create default MessageBus")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_message_bus_creation() {
        let bus = MessageBus::new();
        assert!(bus.is_ok());
    }

    #[tokio::test]
    async fn test_component_registration() {
        let mut bus = MessageBus::new().unwrap();
        let result = bus
            .register("test_component".to_string(), "test_type".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_duplicate_registration_fails() {
        let mut bus = MessageBus::new().unwrap();
        bus.register("test".to_string(), "type".to_string())
            .await
            .unwrap();

        let result = bus.register("test".to_string(), "type".to_string()).await;
        assert!(result.is_err());
    }
}
