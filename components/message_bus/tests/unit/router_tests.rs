// @validates: REQ-001, REQ-002
//! Unit tests for message router

use message_bus::router::MessageRouter;
use shared_types::{ComponentMessage, MessageTarget, MessagePriority, MessagePayload};

#[tokio::test]
async fn test_router_routes_targeted_message() {
    // GIVEN: A message router with a registered component
    let mut router = MessageRouter::new();
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    router.register_component("component-a".to_string(), tx).await.unwrap();

    // WHEN: We route a targeted message
    let msg = ComponentMessage {
        id: "msg-1".to_string(),
        source: "source".to_string(),
        target: MessageTarget::Component("component-a".to_string()),
        timestamp: 1000,
        priority: MessagePriority::Normal,
        payload: MessagePayload::HealthCheck,
    };

    let result = router.route(msg.clone()).await;

    // THEN: The message is successfully routed
    assert!(result.is_ok());

    // AND: The target component receives the message
    let received = rx.recv().await.unwrap();
    assert_eq!(received.id, msg.id);
    assert_eq!(received.source, msg.source);
}

#[tokio::test]
async fn test_router_fails_for_unregistered_component() {
    // GIVEN: A message router with no registered components
    let router = MessageRouter::new();

    // WHEN: We try to route a message to an unregistered component
    let msg = ComponentMessage {
        id: "msg-1".to_string(),
        source: "source".to_string(),
        target: MessageTarget::Component("unknown".to_string()),
        timestamp: 1000,
        priority: MessagePriority::Normal,
        payload: MessagePayload::HealthCheck,
    };

    let result = router.route(msg).await;

    // THEN: The routing fails
    assert!(result.is_err());
}

#[tokio::test]
async fn test_router_broadcasts_to_all_components() {
    // GIVEN: A router with multiple registered components
    let mut router = MessageRouter::new();
    let (tx1, mut rx1) = tokio::sync::mpsc::channel(10);
    let (tx2, mut rx2) = tokio::sync::mpsc::channel(10);

    router.register_component("comp-1".to_string(), tx1).await.unwrap();
    router.register_component("comp-2".to_string(), tx2).await.unwrap();

    // WHEN: We broadcast a message
    let msg = ComponentMessage {
        id: "msg-broadcast".to_string(),
        source: "source".to_string(),
        target: MessageTarget::Broadcast,
        timestamp: 1000,
        priority: MessagePriority::Normal,
        payload: MessagePayload::HealthCheck,
    };

    let result = router.route(msg.clone()).await;

    // THEN: Routing succeeds
    assert!(result.is_ok());

    // AND: All components receive the message
    let received1 = rx1.recv().await.unwrap();
    let received2 = rx2.recv().await.unwrap();

    assert_eq!(received1.id, msg.id);
    assert_eq!(received2.id, msg.id);
}

#[tokio::test]
async fn test_router_deregisters_component() {
    // GIVEN: A router with a registered component
    let mut router = MessageRouter::new();
    let (tx, _rx) = tokio::sync::mpsc::channel(10);
    router.register_component("comp-1".to_string(), tx).await.unwrap();

    // WHEN: We deregister the component
    let result = router.deregister_component("comp-1").await;

    // THEN: Deregistration succeeds
    assert!(result.is_ok());

    // AND: Messages to that component fail
    let msg = ComponentMessage {
        id: "msg-1".to_string(),
        source: "source".to_string(),
        target: MessageTarget::Component("comp-1".to_string()),
        timestamp: 1000,
        priority: MessagePriority::Normal,
        payload: MessagePayload::HealthCheck,
    };

    let route_result = router.route(msg).await;
    assert!(route_result.is_err());
}
