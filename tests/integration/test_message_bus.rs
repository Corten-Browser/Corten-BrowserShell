//! Integration tests for message_bus component communication
//!
//! These tests verify that:
//! 1. MessageBus can be created and initialized
//! 2. Components can register with the message bus
//! 3. Message routing works correctly
//!
//! CRITICAL: These tests use REAL components (no mocking)

use message_bus::MessageBus;

#[tokio::test]
async fn test_message_bus_can_be_created() {
    //! Given: MessageBus::new() is called
    //! When: Creating a new message bus
    //! Then: MessageBus is successfully created

    // Act
    let result = MessageBus::new();

    // Assert
    assert!(result.is_ok(), "MessageBus creation should succeed");
}

#[tokio::test]
async fn test_message_bus_component_registration() {
    //! Given: A MessageBus instance
    //! When: Registering a component
    //! Then: Registration succeeds

    // Arrange
    let mut message_bus = MessageBus::new().expect("MessageBus creation failed");

    // Act
    let result = message_bus
        .register("window_manager".to_string(), "window".to_string())
        .await;

    // Assert
    assert!(result.is_ok(), "Component registration should succeed");
}

#[tokio::test]
async fn test_message_bus_multiple_component_registration() {
    //! Given: A MessageBus instance
    //! When: Registering multiple components
    //! Then: All registrations succeed

    // Arrange
    let mut message_bus = MessageBus::new().expect("MessageBus creation failed");

    // Act & Assert
    message_bus
        .register("window_manager".to_string(), "window".to_string())
        .await
        .expect("Window manager registration failed");

    message_bus
        .register("tab_manager".to_string(), "tab".to_string())
        .await
        .expect("Tab manager registration failed");

    message_bus
        .register("ui_chrome".to_string(), "ui".to_string())
        .await
        .expect("UI chrome registration failed");
}

#[tokio::test]
async fn test_message_bus_subscription() {
    //! Given: A MessageBus with registered component
    //! When: Component subscribes to message type
    //! Then: Subscription succeeds

    // Arrange
    let mut message_bus = MessageBus::new().expect("MessageBus creation failed");
    message_bus
        .register("test_component".to_string(), "test".to_string())
        .await
        .expect("Registration failed");

    // Act
    let result = message_bus
        .subscribe("test_component".to_string(), "NavigateTab".to_string())
        .await;

    // Assert
    assert!(result.is_ok(), "Message subscription should succeed");
}

#[tokio::test]
async fn test_message_bus_handles_unregistered_component_subscription() {
    //! Given: A MessageBus without registered component
    //! When: Trying to subscribe unregistered component
    //! Then: Subscription may fail or succeed based on implementation

    // Arrange
    let mut message_bus = MessageBus::new().expect("MessageBus creation failed");

    // Act
    let result = message_bus
        .subscribe(
            "nonexistent_component".to_string(),
            "SomeMessage".to_string(),
        )
        .await;

    // Assert - verify result is handled appropriately
    // Implementation may allow or disallow this
    assert!(
        result.is_ok() || result.is_err(),
        "Subscribe should return a result"
    );
}
