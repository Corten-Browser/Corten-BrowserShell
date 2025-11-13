// @implements: REQ-005
//! Browser component base interface
//!
//! This module defines the base interface that all browser components must implement
//! for integration with the Browser Shell orchestration system.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use crate::error::ComponentError;

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentHealth {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

/// Component metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetrics {
    pub cpu_usage: f64,
    pub memory_bytes: u64,
    pub message_queue_length: usize,
    pub uptime: Duration,
    pub custom_metrics: HashMap<String, f64>,
}

/// Component configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    pub component_id: String,
    pub config_values: HashMap<String, String>,
    pub debug_mode: bool,
    pub log_level: LogLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Base trait that all browser components must implement
#[async_trait]
pub trait BrowserComponent: Send + Sync {
    /// Initialize component with configuration
    ///
    /// This is called once during component startup. Components should:
    /// - Validate configuration
    /// - Initialize internal state
    /// - Establish connections to dependencies
    /// - Register message handlers
    async fn initialize(&mut self, config: ComponentConfig) -> Result<(), ComponentError>;

    /// Shutdown component gracefully
    ///
    /// Components should:
    /// - Save any pending state
    /// - Close connections
    /// - Stop background tasks
    /// - Release resources
    async fn shutdown(&mut self) -> Result<(), ComponentError>;

    /// Handle inter-component messages
    ///
    /// Process messages from other components and return a response.
    /// This is the primary communication mechanism between components.
    async fn handle_message(
        &mut self,
        msg: ComponentMessage,
    ) -> Result<ComponentResponse, ComponentError>;

    /// Get component health status
    ///
    /// Return current health status. Called periodically by the orchestrator
    /// for monitoring and health checks.
    fn health_check(&self) -> ComponentHealth;

    /// Get component metrics
    ///
    /// Return current performance and operational metrics.
    fn get_metrics(&self) -> ComponentMetrics;
}

/// Generic component message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMessage {
    pub id: String,
    pub source: String,
    pub target: MessageTarget,
    pub timestamp: u64,
    pub priority: MessagePriority,
    pub payload: MessagePayload,
}

/// Message targeting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageTarget {
    Component(String),
    Broadcast,
    Group(String),
}

/// Message priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

/// Message payload types (extensible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    HealthCheck,
    ShutdownRequest,
    ConfigUpdate(HashMap<String, String>),
    Custom(Vec<u8>),
}

/// Generic component response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentResponse {
    Ok,
    Error(String),
    HealthStatus(ComponentHealth),
    Custom(Vec<u8>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_health_can_be_created() {
        let health = ComponentHealth::Healthy;
        assert_eq!(health, ComponentHealth::Healthy);
    }

    #[test]
    fn component_health_degraded_has_reason() {
        let health = ComponentHealth::Degraded {
            reason: "high load".to_string()
        };
        match health {
            ComponentHealth::Degraded { reason } => {
                assert_eq!(reason, "high load");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn component_metrics_can_be_created() {
        let metrics = ComponentMetrics {
            cpu_usage: 45.5,
            memory_bytes: 1024 * 1024,
            message_queue_length: 10,
            uptime: Duration::from_secs(3600),
            custom_metrics: HashMap::new(),
        };
        assert_eq!(metrics.cpu_usage, 45.5);
        assert_eq!(metrics.memory_bytes, 1024 * 1024);
    }

    #[test]
    fn message_priority_ordering() {
        assert!(MessagePriority::Critical < MessagePriority::High);
        assert!(MessagePriority::High < MessagePriority::Normal);
        assert!(MessagePriority::Normal < MessagePriority::Low);
    }

    #[test]
    fn component_message_can_be_created() {
        let msg = ComponentMessage {
            id: "msg-123".to_string(),
            source: "component-a".to_string(),
            target: MessageTarget::Component("component-b".to_string()),
            timestamp: 1234567890,
            priority: MessagePriority::Normal,
            payload: MessagePayload::HealthCheck,
        };
        assert_eq!(msg.id, "msg-123");
        assert_eq!(msg.priority, MessagePriority::Normal);
    }

    #[test]
    fn component_response_variants() {
        let resp_ok = ComponentResponse::Ok;
        let resp_err = ComponentResponse::Error("error".to_string());
        let resp_health = ComponentResponse::HealthStatus(ComponentHealth::Healthy);

        match resp_ok {
            ComponentResponse::Ok => {},
            _ => panic!("Wrong variant"),
        }

        match resp_err {
            ComponentResponse::Error(msg) => assert_eq!(msg, "error"),
            _ => panic!("Wrong variant"),
        }

        match resp_health {
            ComponentResponse::HealthStatus(health) => {
                assert_eq!(health, ComponentHealth::Healthy);
            },
            _ => panic!("Wrong variant"),
        }
    }
}
