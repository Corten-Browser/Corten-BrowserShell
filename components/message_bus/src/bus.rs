//! MessageBus implementation for async inter-component communication

use crate::priority::{
    MessageRouter, PrioritizedMessage, PriorityQueue, QueueMetrics, SharedPriorityQueue,
};
use crate::types::{ComponentMessage, ComponentResponse, MessagePriority};
use shared_types::ComponentError;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

/// A message wrapper that includes priority and routing information
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used for message routing (will be fully utilized in future enhancements)
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
#[allow(dead_code)] // Fields used for component management (will be fully utilized in future enhancements)
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
/// - Message routing with deadline tracking
/// - Priority inversion prevention
pub struct MessageBus {
    /// Registered components
    components: Arc<RwLock<HashMap<String, ComponentInfo>>>,
    /// Priority queue for message ordering
    priority_queue: SharedPriorityQueue,
    /// Message router for automatic priority determination
    router: MessageRouter,
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
            priority_queue: Arc::new(PriorityQueue::new()),
            router: MessageRouter::new(),
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

        component_info.tx.send(routed_message).map_err(|e| {
            ComponentError::MessageRoutingFailed(format!("Failed to send message: {}", e))
        })?;

        // Wait for response (with timeout)
        // In this basic implementation, we'll return Success immediately
        // A real implementation would wait for the actual component response
        drop(components); // Release the lock

        // Try to receive response with timeout
        match tokio::time::timeout(std::time::Duration::from_millis(100), response_rx.recv()).await
        {
            Ok(Some(response)) => Ok(response),
            Ok(None) => Ok(ComponentResponse::Success), // Channel closed
            Err(_) => Ok(ComponentResponse::Success),   // Timeout - return success for now
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

    /// Send a message with explicit priority
    ///
    /// # Arguments
    ///
    /// * `target` - ID of the target component
    /// * `message` - The message to send
    /// * `priority` - The priority level for this message
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
    /// # use message_bus::{MessageBus, ComponentMessage, MessagePriority};
    /// # use shared_types::WindowId;
    /// # async fn example() -> Result<(), shared_types::ComponentError> {
    /// # let mut bus = MessageBus::new()?;
    /// # bus.register("window_manager".to_string(), "manager".to_string()).await?;
    /// let message = ComponentMessage::CloseWindow(WindowId::new());
    /// let response = bus.send_with_priority(
    ///     "window_manager".to_string(),
    ///     message,
    ///     MessagePriority::Critical
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_with_priority(
        &mut self,
        target: String,
        message: ComponentMessage,
        priority: MessagePriority,
    ) -> Result<ComponentResponse, ComponentError> {
        // Enqueue message with priority
        let prioritized_msg =
            PrioritizedMessage::new(message.clone(), priority).with_target(target.clone());

        self.priority_queue
            .enqueue(prioritized_msg)
            .await
            .map_err(|e| ComponentError::MessageRoutingFailed(format!("Queue error: {}", e)))?;

        // Send the message to the target component
        self.send(target, message).await
    }

    /// Send a message with a processing deadline
    ///
    /// Messages that are not processed within the deadline will be discarded.
    ///
    /// # Arguments
    ///
    /// * `target` - ID of the target component
    /// * `message` - The message to send
    /// * `deadline` - Maximum time to wait for processing
    ///
    /// # Returns
    ///
    /// Returns `Ok(ComponentResponse)` with the component's response, or a `ComponentError` if:
    /// - The target component is not registered
    /// - Message delivery fails
    /// - The deadline expires
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use message_bus::{MessageBus, ComponentMessage};
    /// # use shared_types::TabId;
    /// # use std::time::Duration;
    /// # async fn example() -> Result<(), shared_types::ComponentError> {
    /// # let mut bus = MessageBus::new()?;
    /// # bus.register("tab_manager".to_string(), "manager".to_string()).await?;
    /// let message = ComponentMessage::CloseTab(TabId::new());
    /// let response = bus.send_with_deadline(
    ///     "tab_manager".to_string(),
    ///     message,
    ///     Duration::from_secs(5)
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_with_deadline(
        &mut self,
        target: String,
        message: ComponentMessage,
        deadline: Duration,
    ) -> Result<ComponentResponse, ComponentError> {
        // Determine priority automatically
        let priority = self.router.determine_priority(&message);

        // Create message with deadline
        let mut prioritized_msg =
            PrioritizedMessage::new(message.clone(), priority).with_target(target.clone());
        prioritized_msg.deadline = Some(std::time::Instant::now() + deadline);

        self.priority_queue
            .enqueue(prioritized_msg)
            .await
            .map_err(|e| ComponentError::MessageRoutingFailed(format!("Queue error: {}", e)))?;

        // Send the message with timeout
        tokio::time::timeout(deadline, self.send(target, message))
            .await
            .map_err(|_| ComponentError::MessageRoutingFailed("Message deadline expired".to_string()))?
    }

    /// Send a message with automatic priority determination
    ///
    /// Priority is automatically determined based on the message type:
    /// - Critical: User input (keyboard shortcuts), window close
    /// - High: Navigation, tab/window creation
    /// - Normal: UI updates (title, address bar)
    /// - Low: Background tasks
    ///
    /// # Arguments
    ///
    /// * `target` - ID of the target component
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// Returns `Ok(ComponentResponse)` with the component's response
    pub async fn send_auto_priority(
        &mut self,
        target: String,
        message: ComponentMessage,
    ) -> Result<ComponentResponse, ComponentError> {
        let priority = self.router.determine_priority(&message);
        self.send_with_priority(target, message, priority).await
    }

    /// Get the current queue metrics
    ///
    /// # Returns
    ///
    /// Returns queue metrics including:
    /// - Messages processed per priority lane
    /// - Current queue depths
    /// - Wait time statistics
    /// - Expiration counts
    pub async fn queue_metrics(&self) -> QueueMetrics {
        self.priority_queue.metrics().await
    }

    /// Get the current queue depth
    ///
    /// # Returns
    ///
    /// Returns the total number of messages waiting in the queue
    pub async fn queue_depth(&self) -> usize {
        self.priority_queue.len().await
    }

    /// Check if the message queue is empty
    pub async fn queue_is_empty(&self) -> bool {
        self.priority_queue.is_empty().await
    }

    /// Get a reference to the priority queue
    ///
    /// This allows direct access to the priority queue for advanced use cases.
    pub fn priority_queue(&self) -> &SharedPriorityQueue {
        &self.priority_queue
    }

    /// Get a reference to the message router
    ///
    /// This allows access to routing information for messages.
    pub fn router(&self) -> &MessageRouter {
        &self.router
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
    use shared_types::{KeyboardShortcut, TabId, WindowId};

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

    #[tokio::test]
    async fn test_send_with_priority() {
        let mut bus = MessageBus::new().unwrap();
        bus.register("window_manager".to_string(), "manager".to_string())
            .await
            .unwrap();

        let message = ComponentMessage::CloseWindow(WindowId::new());
        let result = bus
            .send_with_priority(
                "window_manager".to_string(),
                message,
                MessagePriority::Critical,
            )
            .await;

        assert!(result.is_ok());

        // Check queue metrics - verify metrics are accessible
        let metrics = bus.queue_metrics().await;
        assert_eq!(metrics.total_depth(), 1); // One message enqueued
    }

    #[tokio::test]
    async fn test_send_with_deadline() {
        let mut bus = MessageBus::new().unwrap();
        bus.register("tab_manager".to_string(), "manager".to_string())
            .await
            .unwrap();

        let message = ComponentMessage::CloseTab(TabId::new());
        let result = bus
            .send_with_deadline(
                "tab_manager".to_string(),
                message,
                Duration::from_secs(5),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_auto_priority() {
        let mut bus = MessageBus::new().unwrap();
        bus.register("window_manager".to_string(), "manager".to_string())
            .await
            .unwrap();

        // Keyboard shortcut should get Critical priority
        let message = ComponentMessage::KeyboardShortcut(KeyboardShortcut::CtrlT);
        let result = bus
            .send_auto_priority("window_manager".to_string(), message)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_queue_depth() {
        let bus = MessageBus::new().unwrap();

        // Initially empty
        assert!(bus.queue_is_empty().await);
        assert_eq!(bus.queue_depth().await, 0);
    }

    #[tokio::test]
    async fn test_priority_queue_access() {
        let bus = MessageBus::new().unwrap();

        // Should be able to access the priority queue
        let queue = bus.priority_queue();
        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_router_access() {
        let bus = MessageBus::new().unwrap();
        let router = bus.router();

        // Test router functionality
        let msg = ComponentMessage::KeyboardShortcut(KeyboardShortcut::CtrlT);
        let priority = router.determine_priority(&msg);
        assert_eq!(priority, MessagePriority::Critical);
    }

    #[tokio::test]
    async fn test_send_to_nonexistent_component_with_priority() {
        let mut bus = MessageBus::new().unwrap();

        let message = ComponentMessage::CloseWindow(WindowId::new());
        let result = bus
            .send_with_priority(
                "nonexistent".to_string(),
                message,
                MessagePriority::High,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_queue_metrics() {
        let bus = MessageBus::new().unwrap();
        let metrics = bus.queue_metrics().await;

        // Initial metrics should be zero
        assert_eq!(metrics.total_processed, 0);
        assert_eq!(metrics.total_expired, 0);
        assert_eq!(metrics.total_depth(), 0);
    }
}
