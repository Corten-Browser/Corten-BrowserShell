// @validates: REQ-005
//! Unit tests for component registry

use message_bus::registry::ComponentRegistry;

#[tokio::test]
async fn test_registry_registers_component() {
    // GIVEN: An empty registry
    let mut registry = ComponentRegistry::new();
    let (tx, _rx) = tokio::sync::mpsc::channel(10);

    // WHEN: We register a component
    let result = registry.register("comp-1".to_string(), tx).await;

    // THEN: Registration succeeds
    assert!(result.is_ok());

    // AND: Component is marked as registered
    assert!(registry.is_registered("comp-1").await);
}

#[tokio::test]
async fn test_registry_rejects_duplicate_registration() {
    // GIVEN: A registry with an already-registered component
    let mut registry = ComponentRegistry::new();
    let (tx1, _rx1) = tokio::sync::mpsc::channel(10);
    let (tx2, _rx2) = tokio::sync::mpsc::channel(10);

    registry.register("comp-1".to_string(), tx1).await.unwrap();

    // WHEN: We try to register the same component again
    let result = registry.register("comp-1".to_string(), tx2).await;

    // THEN: Registration fails
    assert!(result.is_err());
}

#[tokio::test]
async fn test_registry_deregisters_component() {
    // GIVEN: A registry with a registered component
    let mut registry = ComponentRegistry::new();
    let (tx, _rx) = tokio::sync::mpsc::channel(10);
    registry.register("comp-1".to_string(), tx).await.unwrap();

    // WHEN: We deregister the component
    let result = registry.deregister("comp-1").await;

    // THEN: Deregistration succeeds
    assert!(result.is_ok());

    // AND: Component is no longer registered
    assert!(!registry.is_registered("comp-1").await);
}

#[tokio::test]
async fn test_registry_lists_all_components() {
    // GIVEN: A registry with multiple components
    let mut registry = ComponentRegistry::new();
    let (tx1, _rx1) = tokio::sync::mpsc::channel(10);
    let (tx2, _rx2) = tokio::sync::mpsc::channel(10);
    let (tx3, _rx3) = tokio::sync::mpsc::channel(10);

    registry.register("comp-1".to_string(), tx1).await.unwrap();
    registry.register("comp-2".to_string(), tx2).await.unwrap();
    registry.register("comp-3".to_string(), tx3).await.unwrap();

    // WHEN: We list all components
    let components = registry.list_components().await;

    // THEN: All components are listed
    assert_eq!(components.len(), 3);
    assert!(components.contains(&"comp-1".to_string()));
    assert!(components.contains(&"comp-2".to_string()));
    assert!(components.contains(&"comp-3".to_string()));
}

#[tokio::test]
async fn test_registry_get_sender_returns_channel() {
    // GIVEN: A registry with a registered component
    let mut registry = ComponentRegistry::new();
    let (tx, _rx) = tokio::sync::mpsc::channel(10);
    registry.register("comp-1".to_string(), tx).await.unwrap();

    // WHEN: We get the sender for that component
    let sender = registry.get_sender("comp-1").await;

    // THEN: We receive the sender
    assert!(sender.is_some());
}

#[tokio::test]
async fn test_registry_get_sender_returns_none_for_unregistered() {
    // GIVEN: An empty registry
    let registry = ComponentRegistry::new();

    // WHEN: We try to get a sender for an unregistered component
    let sender = registry.get_sender("unknown").await;

    // THEN: We get None
    assert!(sender.is_none());
}
