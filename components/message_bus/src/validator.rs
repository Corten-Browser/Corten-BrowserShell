// @implements: REQ-004
//! Message validation

use shared_types::{ComponentMessage, MessagePayload};
use crate::error::{MessageBusError, Result};

/// Message validator with configurable size limits
pub struct MessageValidator {
    max_message_size: usize,
}

impl MessageValidator {
    /// Create a new validator with a maximum message size in bytes
    pub fn new(max_message_size: usize) -> Self {
        Self { max_message_size }
    }

    /// Validate a message
    ///
    /// Checks:
    /// - Message ID is not empty
    /// - Source is not empty
    /// - Message size is within limits
    pub fn validate(&self, msg: &ComponentMessage) -> Result<()> {
        // Check ID
        if msg.id.is_empty() {
            return Err(MessageBusError::ValidationError(
                "Message id cannot be empty".to_string()
            ));
        }

        // Check source
        if msg.source.is_empty() {
            return Err(MessageBusError::ValidationError(
                "Message source cannot be empty".to_string()
            ));
        }

        // Check message size
        let size = self.estimate_message_size(msg);
        if size > self.max_message_size {
            return Err(MessageBusError::ValidationError(
                format!("Message size {} exceeds limit {}", size, self.max_message_size)
            ));
        }

        Ok(())
    }

    /// Estimate message size in bytes
    fn estimate_message_size(&self, msg: &ComponentMessage) -> usize {
        // Basic size estimation
        let base_size = msg.id.len() + msg.source.len() + 100; // overhead for other fields

        let payload_size = match &msg.payload {
            MessagePayload::HealthCheck => 0,
            MessagePayload::ShutdownRequest => 0,
            MessagePayload::ConfigUpdate(map) => {
                map.iter().map(|(k, v)| k.len() + v.len()).sum()
            }
            MessagePayload::Custom(bytes) => bytes.len(),
        };

        base_size + payload_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::{MessageTarget, MessagePriority};

    #[test]
    fn validator_accepts_valid_message() {
        let validator = MessageValidator::new(1024);
        let msg = ComponentMessage {
            id: "test".to_string(),
            source: "source".to_string(),
            target: MessageTarget::Broadcast,
            timestamp: 1000,
            priority: MessagePriority::Normal,
            payload: MessagePayload::HealthCheck,
        };

        assert!(validator.validate(&msg).is_ok());
    }

    #[test]
    fn validator_rejects_empty_id() {
        let validator = MessageValidator::new(1024);
        let msg = ComponentMessage {
            id: "".to_string(),
            source: "source".to_string(),
            target: MessageTarget::Broadcast,
            timestamp: 1000,
            priority: MessagePriority::Normal,
            payload: MessagePayload::HealthCheck,
        };

        assert!(validator.validate(&msg).is_err());
    }
}
