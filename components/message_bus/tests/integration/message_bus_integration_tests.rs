// @validates: REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006
//! Integration tests for the complete message bus system

use message_bus::MessageBus;
use shared_types::{ComponentMessage, MessageTarget, MessagePriority, MessagePayload};
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_end_to_end_message_routing() {
    // GIVEN: A message bus with two registered components
    let bus = MessageBus::new(1000, 1024 * 1024);
    let (tx1, mut rx1) = tokio::sync::mpsc::channel(10);
    let (tx2, mut rx2) = tokio::sync::mpsc::channel(10);

    bus.register_component("component-a".to_string(), tx1).await.unwrap();
    bus.register_component("component-b".to_string(), tx2).await.unwrap();

    // WHEN: We route messages to each component
    let msg1 = ComponentMessage {
        id: "msg-1".to_string(),
        source: "sender".to_string(),
        target: MessageTarget::Component("component-a".to_string()),
        timestamp: 1000,
        priority: MessagePriority::Normal,
        payload: MessagePayload::HealthCheck,
    };

    let msg2 = ComponentMessage {
        id: "msg-2".to_string(),
        source: "sender".to_string(),
        target: MessageTarget::Component("component-b".to_string()),
        timestamp: 2000,
        priority: MessagePriority::Normal,
        payload: MessagePayload::HealthCheck,
    };

    bus.route_message(msg1.clone()).await.unwrap();
    bus.route_message(msg2.clone()).await.unwrap();

    // THEN: Each component receives its targeted message
    let received1 = timeout(Duration::from_secs(1), rx1.recv())
        .await
        .expect("Timeout")
        .expect("Message");
    let received2 = timeout(Duration::from_secs(1), rx2.recv())
        .await
        .expect("Timeout")
        .expect("Message");

    assert_eq!(received1.id, "msg-1");
    assert_eq!(received2.id, "msg-2");
}

#[tokio::test]
async fn test_broadcast_reaches_all_components() {
    // GIVEN: A message bus with three components
    let bus = MessageBus::new(1000, 1024 * 1024);
    let (tx1, mut rx1) = tokio::sync::mpsc::channel(10);
    let (tx2, mut rx2) = tokio::sync::mpsc::channel(10);
    let (tx3, mut rx3) = tokio::sync::mpsc::channel(10);

    bus.register_component("comp-1".to_string(), tx1).await.unwrap();
    bus.register_component("comp-2".to_string(), tx2).await.unwrap();
    bus.register_component("comp-3".to_string(), tx3).await.unwrap();

    // WHEN: We broadcast a message
    let broadcast_msg = ComponentMessage {
        id: "broadcast-1".to_string(),
        source: "broadcaster".to_string(),
        target: MessageTarget::Broadcast,
        timestamp: 1000,
        priority: MessagePriority::Critical,
        payload: MessagePayload::ShutdownRequest,
    };

    bus.route_message(broadcast_msg.clone()).await.unwrap();

    // THEN: All components receive the broadcast
    let r1 = timeout(Duration::from_secs(1), rx1.recv()).await.unwrap().unwrap();
    let r2 = timeout(Duration::from_secs(1), rx2.recv()).await.unwrap().unwrap();
    let r3 = timeout(Duration::from_secs(1), rx3.recv()).await.unwrap().unwrap();

    assert_eq!(r1.id, "broadcast-1");
    assert_eq!(r2.id, "broadcast-1");
    assert_eq!(r3.id, "broadcast-1");
}

#[tokio::test]
async fn test_priority_queue_ordering() {
    // GIVEN: A message bus
    let bus = MessageBus::new(1000, 1024 * 1024);
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    bus.register_component("target".to_string(), tx).await.unwrap();

    // WHEN: We enqueue messages with different priorities
    let low_msg = create_message("low", MessagePriority::Low);
    let high_msg = create_message("high", MessagePriority::High);
    let critical_msg = create_message("critical", MessagePriority::Critical);
    let normal_msg = create_message("normal", MessagePriority::Normal);

    bus.enqueue_message(low_msg).await.unwrap();
    bus.enqueue_message(high_msg).await.unwrap();
    bus.enqueue_message(critical_msg).await.unwrap();
    bus.enqueue_message(normal_msg).await.unwrap();

    // AND: We process them in order
    bus.process_next().await.unwrap();
    let r1 = timeout(Duration::from_millis(100), rx.recv()).await.unwrap().unwrap();

    bus.process_next().await.unwrap();
    let r2 = timeout(Duration::from_millis(100), rx.recv()).await.unwrap().unwrap();

    bus.process_next().await.unwrap();
    let r3 = timeout(Duration::from_millis(100), rx.recv()).await.unwrap().unwrap();

    bus.process_next().await.unwrap();
    let r4 = timeout(Duration::from_millis(100), rx.recv()).await.unwrap().unwrap();

    // THEN: Messages are processed in priority order
    assert_eq!(r1.id, "critical");
    assert_eq!(r2.id, "high");
    assert_eq!(r3.id, "normal");
    assert_eq!(r4.id, "low");
}

#[tokio::test]
async fn test_validation_prevents_invalid_messages() {
    // GIVEN: A message bus
    let bus = MessageBus::new(1000, 1024 * 1024);

    // WHEN: We try to route an invalid message (empty ID)
    let invalid_msg = ComponentMessage {
        id: "".to_string(),
        source: "sender".to_string(),
        target: MessageTarget::Broadcast,
        timestamp: 1000,
        priority: MessagePriority::Normal,
        payload: MessagePayload::HealthCheck,
    };

    let result = bus.route_message(invalid_msg).await;

    // THEN: The message is rejected
    assert!(result.is_err());
}

#[tokio::test]
async fn test_component_registration_and_metrics() {
    // GIVEN: A message bus
    let bus = MessageBus::new(1000, 1024 * 1024);

    // WHEN: We register components
    let (tx1, _rx1) = tokio::sync::mpsc::channel(10);
    let (tx2, _rx2) = tokio::sync::mpsc::channel(10);

    bus.register_component("comp-1".to_string(), tx1).await.unwrap();
    bus.register_component("comp-2".to_string(), tx2).await.unwrap();

    // AND: Enqueue some messages
    let msg = create_message("test", MessagePriority::Normal);
    bus.enqueue_message(msg.clone()).await.unwrap();
    bus.enqueue_message(msg.clone()).await.unwrap();
    bus.enqueue_message(msg).await.unwrap();

    // THEN: Metrics reflect the state
    let metrics = bus.get_metrics().await;
    assert_eq!(metrics.registered_components, 2);
    assert_eq!(metrics.queued_messages, 3);

    // WHEN: We check registration status
    assert!(bus.is_component_registered("comp-1").await);
    assert!(bus.is_component_registered("comp-2").await);
    assert!(!bus.is_component_registered("unknown").await);
}

#[tokio::test]
async fn test_deregistration_prevents_routing() {
    // GIVEN: A registered component
    let bus = MessageBus::new(1000, 1024 * 1024);
    let (tx, _rx) = tokio::sync::mpsc::channel(10);
    bus.register_component("temp-component".to_string(), tx).await.unwrap();

    // WHEN: We deregister it
    bus.deregister_component("temp-component").await.unwrap();

    // AND: Try to route a message to it
    let msg = ComponentMessage {
        id: "msg-1".to_string(),
        source: "sender".to_string(),
        target: MessageTarget::Component("temp-component".to_string()),
        timestamp: 1000,
        priority: MessagePriority::Normal,
        payload: MessagePayload::HealthCheck,
    };

    let result = bus.route_message(msg).await;

    // THEN: Routing fails
    assert!(result.is_err());
}

// Helper function
fn create_message(id: &str, priority: MessagePriority) -> ComponentMessage {
    ComponentMessage {
        id: id.to_string(),
        source: "test".to_string(),
        target: MessageTarget::Component("target".to_string()),
        timestamp: 1000,
        priority,
        payload: MessagePayload::HealthCheck,
    }
}
