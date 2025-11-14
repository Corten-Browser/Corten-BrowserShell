//! Integration tests for message flow through the bus

use message_bus::{ComponentMessage, ComponentResponse, MessageBus};
use shared_types::{KeyboardShortcut, TabId, WindowConfig, WindowId};

#[tokio::test]
async fn test_complete_message_flow() {
    let mut bus = MessageBus::new().expect("Failed to create message bus");

    // Register components
    bus.register("window_manager".to_string(), "manager".to_string())
        .await
        .expect("Failed to register window_manager");

    bus.register("tab_manager".to_string(), "manager".to_string())
        .await
        .expect("Failed to register tab_manager");

    bus.register("ui_chrome".to_string(), "ui".to_string())
        .await
        .expect("Failed to register ui_chrome");

    // Send messages between components
    let window_msg = ComponentMessage::CreateWindow(WindowConfig::default());
    let result = bus.send("window_manager".to_string(), window_msg).await;
    assert!(result.is_ok());

    let tab_msg =
        ComponentMessage::CreateTab(WindowId::new(), Some("https://example.com".to_string()));
    let result = bus.send("tab_manager".to_string(), tab_msg).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_broadcast_to_multiple_components() {
    let mut bus = MessageBus::new().expect("Failed to create message bus");

    // Register multiple components
    for i in 0..5 {
        let component_id = format!("component_{}", i);
        bus.register(component_id, format!("type_{}", i))
            .await
            .expect(&format!("Failed to register component_{}", i));
    }

    // Broadcast a keyboard shortcut
    let message = ComponentMessage::KeyboardShortcut(KeyboardShortcut::CtrlT);
    let result = bus.broadcast(message).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_subscription_filtering() {
    let mut bus = MessageBus::new().expect("Failed to create message bus");

    // Register components
    bus.register("component_a".to_string(), "type_a".to_string())
        .await
        .expect("Failed to register component_a");

    bus.register("component_b".to_string(), "type_b".to_string())
        .await
        .expect("Failed to register component_b");

    // Component A subscribes to CreateWindow
    bus.subscribe("component_a".to_string(), "CreateWindow".to_string())
        .await
        .expect("Failed to subscribe component_a");

    // Component B subscribes to CreateTab
    bus.subscribe("component_b".to_string(), "CreateTab".to_string())
        .await
        .expect("Failed to subscribe component_b");

    // Both subscriptions should succeed
}

#[tokio::test]
async fn test_message_types_serialization() {
    // Test that messages can be serialized and deserialized
    let window_id = WindowId::new();
    let msg = ComponentMessage::CloseWindow(window_id);

    let serialized = serde_json::to_string(&msg).expect("Failed to serialize message");
    assert!(!serialized.is_empty());

    let deserialized: ComponentMessage =
        serde_json::from_str(&serialized).expect("Failed to deserialize message");

    match deserialized {
        ComponentMessage::CloseWindow(id) => assert_eq!(id, window_id),
        _ => panic!("Wrong message type after deserialization"),
    }
}

#[tokio::test]
async fn test_response_types_serialization() {
    let tab_id = TabId::new();
    let response = ComponentResponse::TabCreated(tab_id);

    let serialized = serde_json::to_string(&response).expect("Failed to serialize response");
    assert!(!serialized.is_empty());

    let deserialized: ComponentResponse =
        serde_json::from_str(&serialized).expect("Failed to deserialize response");

    match deserialized {
        ComponentResponse::TabCreated(id) => assert_eq!(id, tab_id),
        _ => panic!("Wrong response type after deserialization"),
    }
}

#[tokio::test]
async fn test_concurrent_registrations() {
    let mut bus = MessageBus::new().expect("Failed to create message bus");

    // Register multiple components sequentially
    // (Concurrent registration would require Arc<Mutex<MessageBus>>)
    for i in 0..10 {
        let component_id = format!("concurrent_{}", i);
        let component_type = format!("type_{}", i);
        let result = bus.register(component_id, component_type).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_error_handling_on_invalid_operations() {
    let mut bus = MessageBus::new().expect("Failed to create message bus");

    // Try to send to non-existent component
    let msg = ComponentMessage::CloseWindow(WindowId::new());
    let result = bus.send("nonexistent".to_string(), msg).await;
    assert!(result.is_err());

    // Try to subscribe non-existent component
    let result = bus
        .subscribe("nonexistent".to_string(), "CreateWindow".to_string())
        .await;
    assert!(result.is_err());

    // Try to register duplicate
    bus.register("duplicate".to_string(), "type".to_string())
        .await
        .expect("First registration should succeed");

    let result = bus
        .register("duplicate".to_string(), "type".to_string())
        .await;
    assert!(result.is_err());
}
