// @validates: REQ-003
//! Unit tests for priority queue

use message_bus::queue::PriorityQueue;
use shared_types::{ComponentMessage, MessageTarget, MessagePriority, MessagePayload};

#[tokio::test]
async fn test_queue_orders_by_priority() {
    // GIVEN: A priority queue with messages of different priorities
    let mut queue = PriorityQueue::new(100);

    let low_msg = create_message("low", MessagePriority::Low);
    let high_msg = create_message("high", MessagePriority::High);
    let critical_msg = create_message("critical", MessagePriority::Critical);
    let normal_msg = create_message("normal", MessagePriority::Normal);

    // WHEN: We push messages in random order
    queue.push(low_msg).await.unwrap();
    queue.push(high_msg).await.unwrap();
    queue.push(critical_msg).await.unwrap();
    queue.push(normal_msg).await.unwrap();

    // THEN: Messages are retrieved in priority order
    assert_eq!(queue.pop().await.unwrap().id, "critical");
    assert_eq!(queue.pop().await.unwrap().id, "high");
    assert_eq!(queue.pop().await.unwrap().id, "normal");
    assert_eq!(queue.pop().await.unwrap().id, "low");
}

#[tokio::test]
async fn test_queue_respects_capacity() {
    // GIVEN: A small capacity queue
    let mut queue = PriorityQueue::new(2);

    let msg1 = create_message("msg1", MessagePriority::Normal);
    let msg2 = create_message("msg2", MessagePriority::Normal);
    let msg3 = create_message("msg3", MessagePriority::Normal);

    // WHEN: We try to exceed capacity
    assert!(queue.push(msg1).await.is_ok());
    assert!(queue.push(msg2).await.is_ok());

    // THEN: Further pushes fail (or block based on implementation)
    // We'll use try_push for non-blocking version
    assert!(queue.try_push(msg3).is_err());
}

#[tokio::test]
async fn test_queue_returns_none_when_empty() {
    // GIVEN: An empty queue
    let queue = PriorityQueue::new(10);

    // WHEN: We try to pop from empty queue (non-blocking)
    let result = queue.try_pop();

    // THEN: We get None
    assert!(result.is_none());
}

#[tokio::test]
async fn test_queue_fifo_within_same_priority() {
    // GIVEN: Multiple messages with the same priority
    let mut queue = PriorityQueue::new(10);

    let msg1 = create_message_with_timestamp("msg1", MessagePriority::Normal, 1000);
    let msg2 = create_message_with_timestamp("msg2", MessagePriority::Normal, 2000);
    let msg3 = create_message_with_timestamp("msg3", MessagePriority::Normal, 3000);

    // WHEN: We push them
    queue.push(msg1).await.unwrap();
    queue.push(msg2).await.unwrap();
    queue.push(msg3).await.unwrap();

    // THEN: They're retrieved in FIFO order (within same priority)
    assert_eq!(queue.pop().await.unwrap().id, "msg1");
    assert_eq!(queue.pop().await.unwrap().id, "msg2");
    assert_eq!(queue.pop().await.unwrap().id, "msg3");
}

fn create_message(id: &str, priority: MessagePriority) -> ComponentMessage {
    ComponentMessage {
        id: id.to_string(),
        source: "test".to_string(),
        target: MessageTarget::Component("test".to_string()),
        timestamp: 1000,
        priority,
        payload: MessagePayload::HealthCheck,
    }
}

fn create_message_with_timestamp(id: &str, priority: MessagePriority, timestamp: u64) -> ComponentMessage {
    ComponentMessage {
        id: id.to_string(),
        source: "test".to_string(),
        target: MessageTarget::Component("test".to_string()),
        timestamp,
        priority,
        payload: MessagePayload::HealthCheck,
    }
}
