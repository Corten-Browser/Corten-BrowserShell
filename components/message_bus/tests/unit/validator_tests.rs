// @validates: REQ-004
//! Unit tests for message validation

use message_bus::validator::MessageValidator;
use shared_types::{ComponentMessage, MessageTarget, MessagePriority, MessagePayload};

#[test]
fn test_validator_accepts_valid_message() {
    // GIVEN: A valid message
    let msg = ComponentMessage {
        id: "msg-1".to_string(),
        source: "comp-a".to_string(),
        target: MessageTarget::Component("comp-b".to_string()),
        timestamp: 1000,
        priority: MessagePriority::Normal,
        payload: MessagePayload::HealthCheck,
    };

    let validator = MessageValidator::new(1024 * 1024); // 1MB limit

    // WHEN: We validate the message
    let result = validator.validate(&msg);

    // THEN: Validation succeeds
    assert!(result.is_ok());
}

#[test]
fn test_validator_rejects_empty_id() {
    // GIVEN: A message with empty ID
    let msg = ComponentMessage {
        id: "".to_string(),
        source: "comp-a".to_string(),
        target: MessageTarget::Component("comp-b".to_string()),
        timestamp: 1000,
        priority: MessagePriority::Normal,
        payload: MessagePayload::HealthCheck,
    };

    let validator = MessageValidator::new(1024 * 1024);

    // WHEN: We validate the message
    let result = validator.validate(&msg);

    // THEN: Validation fails
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("id"));
}

#[test]
fn test_validator_rejects_empty_source() {
    // GIVEN: A message with empty source
    let msg = ComponentMessage {
        id: "msg-1".to_string(),
        source: "".to_string(),
        target: MessageTarget::Component("comp-b".to_string()),
        timestamp: 1000,
        priority: MessagePriority::Normal,
        payload: MessagePayload::HealthCheck,
    };

    let validator = MessageValidator::new(1024 * 1024);

    // WHEN: We validate the message
    let result = validator.validate(&msg);

    // THEN: Validation fails
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("source"));
}

#[test]
fn test_validator_rejects_oversized_payload() {
    // GIVEN: A message with a large payload
    let large_payload = vec![0u8; 2 * 1024 * 1024]; // 2MB
    let msg = ComponentMessage {
        id: "msg-1".to_string(),
        source: "comp-a".to_string(),
        target: MessageTarget::Component("comp-b".to_string()),
        timestamp: 1000,
        priority: MessagePriority::Normal,
        payload: MessagePayload::Custom(large_payload),
    };

    let validator = MessageValidator::new(1024 * 1024); // 1MB limit

    // WHEN: We validate the message
    let result = validator.validate(&msg);

    // THEN: Validation fails due to size
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("size") || err_msg.contains("too large"));
}

#[test]
fn test_validator_accepts_message_below_size_limit() {
    // GIVEN: A message well below the size limit
    let payload = vec![0u8; 512 * 1024]; // 512KB payload
    let msg = ComponentMessage {
        id: "msg-1".to_string(),
        source: "comp-a".to_string(),
        target: MessageTarget::Component("comp-b".to_string()),
        timestamp: 1000,
        priority: MessagePriority::Normal,
        payload: MessagePayload::Custom(payload),
    };

    let validator = MessageValidator::new(1024 * 1024); // 1MB limit

    // WHEN: We validate the message
    let result = validator.validate(&msg);

    // THEN: Validation succeeds (payload + overhead < limit)
    assert!(result.is_ok());
}
