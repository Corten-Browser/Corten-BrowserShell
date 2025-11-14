//! Unit tests for MessageBus functionality

use message_bus::{ComponentMessage, ComponentResponse, MessageBus};
use shared_types::{ComponentError, KeyboardShortcut, WindowId};

#[tokio::test]
async fn test_message_bus_creation() {
    let bus = MessageBus::new();
    assert!(bus.is_ok());
}

#[tokio::test]
async fn test_register_component() {
    let mut bus = MessageBus::new().expect("Failed to create message bus");
    let result = bus.register("window_manager".to_string(), "manager".to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_duplicate_component() {
    let mut bus = MessageBus::new().expect("Failed to create message bus");

    // Register once - should succeed
    let result1 = bus.register("window_manager".to_string(), "manager".to_string()).await;
    assert!(result1.is_ok());

    // Register same component again - should fail
    let result2 = bus.register("window_manager".to_string(), "manager".to_string()).await;
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_send_message_to_unregistered_component() {
    let mut bus = MessageBus::new().expect("Failed to create message bus");
    let message = ComponentMessage::CloseWindow(WindowId::new());

    let result = bus.send("nonexistent".to_string(), message).await;
    assert!(result.is_err());

    if let Err(ComponentError::ResourceNotFound(msg)) = result {
        assert!(msg.contains("nonexistent") || msg.contains("not found"));
    } else {
        panic!("Expected ResourceNotFound error");
    }
}

#[tokio::test]
async fn test_send_message_to_registered_component() {
    let mut bus = MessageBus::new().expect("Failed to create message bus");

    // Register a component
    bus.register("test_component".to_string(), "test".to_string())
        .await
        .expect("Failed to register component");

    let message = ComponentMessage::CloseWindow(WindowId::new());

    // Should not error even though no handler is set up yet
    let _result = bus.send("test_component".to_string(), message).await;
    // The result depends on implementation - either succeeds or times out
    // For now, we just test that it doesn't panic
}

#[tokio::test]
async fn test_broadcast_message() {
    let mut bus = MessageBus::new().expect("Failed to create message bus");

    // Register multiple components
    bus.register("component1".to_string(), "type1".to_string())
        .await
        .expect("Failed to register component1");
    bus.register("component2".to_string(), "type2".to_string())
        .await
        .expect("Failed to register component2");

    let message = ComponentMessage::KeyboardShortcut(KeyboardShortcut::CtrlT);

    // Broadcast should succeed
    let result = bus.broadcast(message).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_subscribe_to_message_type() {
    let mut bus = MessageBus::new().expect("Failed to create message bus");

    // Register a component first
    bus.register("subscriber".to_string(), "type1".to_string())
        .await
        .expect("Failed to register component");

    // Subscribe to a message type
    let result = bus
        .subscribe("subscriber".to_string(), "CreateWindow".to_string())
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_subscribe_unregistered_component() {
    let mut bus = MessageBus::new().expect("Failed to create message bus");

    // Try to subscribe without registering first
    let result = bus
        .subscribe("nonexistent".to_string(), "CreateWindow".to_string())
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_multiple_subscriptions() {
    let mut bus = MessageBus::new().expect("Failed to create message bus");

    bus.register("subscriber".to_string(), "type1".to_string())
        .await
        .expect("Failed to register component");

    // Subscribe to multiple message types
    let result1 = bus
        .subscribe("subscriber".to_string(), "CreateWindow".to_string())
        .await;
    assert!(result1.is_ok());

    let result2 = bus
        .subscribe("subscriber".to_string(), "CreateTab".to_string())
        .await;
    assert!(result2.is_ok());
}
