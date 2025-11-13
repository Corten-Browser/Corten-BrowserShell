// Browser Component Base Interface Contract
// Version: 0.17.0
//
// This contract defines the base interface that all browser components must implement
// for integration with the Browser Shell orchestration system.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

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

/// Component error types
#[derive(Debug, thiserror::Error)]
pub enum ComponentError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Component not initialized")]
    NotInitialized,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Message handling error: {0}")]
    MessageError(String),

    #[error("Component shutdown error: {0}")]
    ShutdownError(String),

    #[error("Internal error: {0}")]
    Internal(String),
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
