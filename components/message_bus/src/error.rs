// @implements: REQ-006
//! Error types for message bus

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MessageBusError {
    #[error("Component '{0}' not found")]
    ComponentNotFound(String),

    #[error("Component '{0}' already registered")]
    ComponentAlreadyRegistered(String),

    #[error("Message validation failed: {0}")]
    ValidationError(String),

    #[error("Queue is full")]
    QueueFull,

    #[error("Queue is empty")]
    QueueEmpty,

    #[error("Routing failed: {0}")]
    RoutingError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, MessageBusError>;
