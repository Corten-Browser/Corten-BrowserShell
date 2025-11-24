//! Core BrowserComponent trait and lifecycle management types.
//!
//! This module provides the foundational trait that all browser components must implement,
//! along with types for health reporting, metrics collection, and lifecycle state management.
//!
//! # Overview
//!
//! The [`BrowserComponent`] trait defines a standard interface for all browser shell components,
//! enabling consistent initialization, shutdown, message handling, and health monitoring.
//!
//! # Example
//!
//! ```rust,ignore
//! use shared_types::{BrowserComponent, ComponentState, ComponentHealth, ComponentMetrics, ComponentError};
//! use async_trait::async_trait;
//!
//! struct MyComponent {
//!     state: ComponentState,
//! }
//!
//! #[async_trait]
//! impl BrowserComponent for MyComponent {
//!     fn name(&self) -> &str {
//!         "my_component"
//!     }
//!
//!     fn state(&self) -> ComponentState {
//!         self.state.clone()
//!     }
//!
//!     async fn initialize(&mut self) -> Result<(), ComponentError> {
//!         self.state = ComponentState::Running;
//!         Ok(())
//!     }
//!
//!     async fn shutdown(&mut self) -> Result<(), ComponentError> {
//!         self.state = ComponentState::Stopped;
//!         Ok(())
//!     }
//!
//!     async fn handle_message(&mut self, message: &[u8]) -> Result<Vec<u8>, ComponentError> {
//!         Ok(vec![])
//!     }
//!
//!     async fn health_check(&self) -> ComponentHealth {
//!         ComponentHealth::Healthy
//!     }
//!
//!     fn get_metrics(&self) -> ComponentMetrics {
//!         ComponentMetrics::new("my_component")
//!     }
//! }
//! ```

use crate::ComponentError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the lifecycle state of a browser component.
///
/// Components transition through these states during their lifetime:
/// `Created` -> `Initializing` -> `Running` -> `Stopping` -> `Stopped`
///
/// # State Transitions
///
/// - `Created`: Initial state after construction, before any initialization
/// - `Initializing`: Component is performing async initialization
/// - `Running`: Component is fully operational and processing messages
/// - `Stopping`: Component is performing graceful shutdown
/// - `Stopped`: Component has completed shutdown and released resources
///
/// # Example
///
/// ```rust
/// use shared_types::ComponentState;
///
/// let state = ComponentState::Created;
/// assert!(state.is_created());
/// assert!(!state.is_running());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ComponentState {
    /// Component has been created but not yet initialized
    #[default]
    Created,
    /// Component is currently initializing
    Initializing,
    /// Component is running and operational
    Running,
    /// Component is in the process of stopping
    Stopping,
    /// Component has stopped and released resources
    Stopped,
}

impl ComponentState {
    /// Returns `true` if the component is in the `Created` state.
    #[inline]
    pub fn is_created(&self) -> bool {
        matches!(self, ComponentState::Created)
    }

    /// Returns `true` if the component is in the `Initializing` state.
    #[inline]
    pub fn is_initializing(&self) -> bool {
        matches!(self, ComponentState::Initializing)
    }

    /// Returns `true` if the component is in the `Running` state.
    #[inline]
    pub fn is_running(&self) -> bool {
        matches!(self, ComponentState::Running)
    }

    /// Returns `true` if the component is in the `Stopping` state.
    #[inline]
    pub fn is_stopping(&self) -> bool {
        matches!(self, ComponentState::Stopping)
    }

    /// Returns `true` if the component is in the `Stopped` state.
    #[inline]
    pub fn is_stopped(&self) -> bool {
        matches!(self, ComponentState::Stopped)
    }

    /// Returns `true` if the component can accept and process messages.
    ///
    /// Only components in the `Running` state can process messages.
    #[inline]
    pub fn can_process_messages(&self) -> bool {
        matches!(self, ComponentState::Running)
    }

    /// Returns `true` if the component can be initialized.
    ///
    /// Only components in the `Created` state can be initialized.
    #[inline]
    pub fn can_initialize(&self) -> bool {
        matches!(self, ComponentState::Created)
    }

    /// Returns `true` if the component can be shut down.
    ///
    /// Components in `Running` or `Initializing` states can be shut down.
    #[inline]
    pub fn can_shutdown(&self) -> bool {
        matches!(self, ComponentState::Running | ComponentState::Initializing)
    }
}

impl std::fmt::Display for ComponentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentState::Created => write!(f, "Created"),
            ComponentState::Initializing => write!(f, "Initializing"),
            ComponentState::Running => write!(f, "Running"),
            ComponentState::Stopping => write!(f, "Stopping"),
            ComponentState::Stopped => write!(f, "Stopped"),
        }
    }
}

/// Represents the health status of a component.
///
/// Health status is used by monitoring systems to determine if a component
/// is functioning correctly, experiencing issues, or completely unavailable.
///
/// # Example
///
/// ```rust
/// use shared_types::ComponentHealth;
///
/// let health = ComponentHealth::Degraded {
///     reason: "High memory usage".to_string(),
/// };
///
/// assert!(health.is_degraded());
/// assert!(!health.is_healthy());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ComponentHealth {
    /// Component is fully operational
    #[default]
    Healthy,
    /// Component is operational but experiencing issues
    Degraded {
        /// Description of why the component is degraded
        reason: String,
    },
    /// Component is not operational
    Unhealthy {
        /// Description of why the component is unhealthy
        reason: String,
    },
}

impl ComponentHealth {
    /// Creates a new `Healthy` health status.
    #[inline]
    pub fn healthy() -> Self {
        ComponentHealth::Healthy
    }

    /// Creates a new `Degraded` health status with the given reason.
    ///
    /// # Arguments
    ///
    /// * `reason` - A description of why the component is degraded
    #[inline]
    pub fn degraded(reason: impl Into<String>) -> Self {
        ComponentHealth::Degraded {
            reason: reason.into(),
        }
    }

    /// Creates a new `Unhealthy` health status with the given reason.
    ///
    /// # Arguments
    ///
    /// * `reason` - A description of why the component is unhealthy
    #[inline]
    pub fn unhealthy(reason: impl Into<String>) -> Self {
        ComponentHealth::Unhealthy {
            reason: reason.into(),
        }
    }

    /// Returns `true` if the component is healthy.
    #[inline]
    pub fn is_healthy(&self) -> bool {
        matches!(self, ComponentHealth::Healthy)
    }

    /// Returns `true` if the component is degraded.
    #[inline]
    pub fn is_degraded(&self) -> bool {
        matches!(self, ComponentHealth::Degraded { .. })
    }

    /// Returns `true` if the component is unhealthy.
    #[inline]
    pub fn is_unhealthy(&self) -> bool {
        matches!(self, ComponentHealth::Unhealthy { .. })
    }

    /// Returns the reason string if the component is degraded or unhealthy.
    ///
    /// Returns `None` if the component is healthy.
    pub fn reason(&self) -> Option<&str> {
        match self {
            ComponentHealth::Healthy => None,
            ComponentHealth::Degraded { reason } | ComponentHealth::Unhealthy { reason } => {
                Some(reason)
            }
        }
    }
}

impl std::fmt::Display for ComponentHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentHealth::Healthy => write!(f, "Healthy"),
            ComponentHealth::Degraded { reason } => write!(f, "Degraded: {}", reason),
            ComponentHealth::Unhealthy { reason } => write!(f, "Unhealthy: {}", reason),
        }
    }
}

/// Metrics collected from a browser component.
///
/// This struct provides a standardized way to report metrics from any component,
/// including message counts, error rates, and custom metrics.
///
/// # Example
///
/// ```rust
/// use shared_types::ComponentMetrics;
///
/// let mut metrics = ComponentMetrics::new("window_manager");
/// metrics.increment_messages_received();
/// metrics.increment_messages_sent();
/// metrics.set_custom_metric("active_windows", 5.0);
///
/// assert_eq!(metrics.messages_received(), 1);
/// assert_eq!(metrics.messages_sent(), 1);
/// assert_eq!(metrics.custom_metric("active_windows"), Some(&5.0));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentMetrics {
    /// Name of the component these metrics belong to
    component_name: String,
    /// Timestamp when metrics collection started
    started_at: DateTime<Utc>,
    /// Timestamp of last metric update
    last_updated: DateTime<Utc>,
    /// Total number of messages received
    messages_received: u64,
    /// Total number of messages sent
    messages_sent: u64,
    /// Total number of errors encountered
    errors_count: u64,
    /// Average message processing time in microseconds
    avg_processing_time_us: u64,
    /// Component-specific custom metrics
    custom_metrics: HashMap<String, f64>,
}

impl ComponentMetrics {
    /// Creates a new `ComponentMetrics` instance for the given component name.
    ///
    /// # Arguments
    ///
    /// * `component_name` - The name of the component
    ///
    /// # Example
    ///
    /// ```rust
    /// use shared_types::ComponentMetrics;
    ///
    /// let metrics = ComponentMetrics::new("tab_manager");
    /// assert_eq!(metrics.component_name(), "tab_manager");
    /// ```
    pub fn new(component_name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            component_name: component_name.into(),
            started_at: now,
            last_updated: now,
            messages_received: 0,
            messages_sent: 0,
            errors_count: 0,
            avg_processing_time_us: 0,
            custom_metrics: HashMap::new(),
        }
    }

    /// Returns the component name.
    #[inline]
    pub fn component_name(&self) -> &str {
        &self.component_name
    }

    /// Returns when metrics collection started.
    #[inline]
    pub fn started_at(&self) -> DateTime<Utc> {
        self.started_at
    }

    /// Returns when metrics were last updated.
    #[inline]
    pub fn last_updated(&self) -> DateTime<Utc> {
        self.last_updated
    }

    /// Returns the total number of messages received.
    #[inline]
    pub fn messages_received(&self) -> u64 {
        self.messages_received
    }

    /// Returns the total number of messages sent.
    #[inline]
    pub fn messages_sent(&self) -> u64 {
        self.messages_sent
    }

    /// Returns the total number of errors encountered.
    #[inline]
    pub fn errors_count(&self) -> u64 {
        self.errors_count
    }

    /// Returns the average message processing time in microseconds.
    #[inline]
    pub fn avg_processing_time_us(&self) -> u64 {
        self.avg_processing_time_us
    }

    /// Increments the messages received counter.
    pub fn increment_messages_received(&mut self) {
        self.messages_received += 1;
        self.last_updated = Utc::now();
    }

    /// Increments the messages sent counter.
    pub fn increment_messages_sent(&mut self) {
        self.messages_sent += 1;
        self.last_updated = Utc::now();
    }

    /// Increments the error counter.
    pub fn increment_errors(&mut self) {
        self.errors_count += 1;
        self.last_updated = Utc::now();
    }

    /// Updates the average processing time.
    ///
    /// # Arguments
    ///
    /// * `time_us` - The processing time in microseconds
    ///
    /// Note: This performs a simple moving average calculation.
    pub fn update_processing_time(&mut self, time_us: u64) {
        if self.messages_received == 0 {
            self.avg_processing_time_us = time_us;
        } else {
            // Simple moving average
            self.avg_processing_time_us =
                (self.avg_processing_time_us + time_us) / 2;
        }
        self.last_updated = Utc::now();
    }

    /// Sets a custom metric value.
    ///
    /// # Arguments
    ///
    /// * `name` - The metric name
    /// * `value` - The metric value
    pub fn set_custom_metric(&mut self, name: impl Into<String>, value: f64) {
        self.custom_metrics.insert(name.into(), value);
        self.last_updated = Utc::now();
    }

    /// Gets a custom metric value by name.
    ///
    /// Returns `None` if the metric doesn't exist.
    pub fn custom_metric(&self, name: &str) -> Option<&f64> {
        self.custom_metrics.get(name)
    }

    /// Returns all custom metrics.
    pub fn custom_metrics(&self) -> &HashMap<String, f64> {
        &self.custom_metrics
    }

    /// Calculates the uptime since metrics collection started.
    pub fn uptime(&self) -> chrono::Duration {
        Utc::now() - self.started_at
    }

    /// Resets all counters to zero.
    ///
    /// This resets messages_received, messages_sent, errors_count, and
    /// avg_processing_time_us while preserving custom metrics.
    pub fn reset_counters(&mut self) {
        self.messages_received = 0;
        self.messages_sent = 0;
        self.errors_count = 0;
        self.avg_processing_time_us = 0;
        self.last_updated = Utc::now();
    }
}

impl Default for ComponentMetrics {
    fn default() -> Self {
        Self::new("unknown")
    }
}

/// Core trait that all browser shell components must implement.
///
/// This trait provides a standardized interface for component lifecycle management,
/// message handling, health monitoring, and metrics collection.
///
/// # Thread Safety
///
/// The trait is designed to be used in async contexts and includes `Send + Sync`
/// bounds to ensure thread safety when components are shared across tasks.
///
/// # Lifecycle
///
/// Components should follow this lifecycle:
/// 1. Create the component (state: `Created`)
/// 2. Call `initialize()` (state transitions: `Created` -> `Initializing` -> `Running`)
/// 3. Process messages with `handle_message()` (state: `Running`)
/// 4. Call `shutdown()` (state transitions: `Running` -> `Stopping` -> `Stopped`)
///
/// # Example Implementation
///
/// See the module documentation for a complete example implementation.
#[async_trait]
pub trait BrowserComponent: Send + Sync {
    /// Returns the unique name of this component.
    ///
    /// This name should be consistent and unique within the browser shell,
    /// typically matching the component's module name (e.g., "window_manager").
    fn name(&self) -> &str;

    /// Returns the current lifecycle state of the component.
    fn state(&self) -> ComponentState;

    /// Initializes the component.
    ///
    /// This method performs any necessary async initialization, such as:
    /// - Connecting to external resources
    /// - Loading configuration
    /// - Setting up internal state
    ///
    /// # State Transition
    ///
    /// The component state should transition:
    /// `Created` -> `Initializing` -> `Running` (on success)
    /// `Created` -> `Initializing` -> `Created` or `Stopped` (on failure)
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::InitializationFailed` if initialization fails.
    /// Returns `ComponentError::InvalidState` if called when not in `Created` state.
    async fn initialize(&mut self) -> Result<(), ComponentError>;

    /// Shuts down the component gracefully.
    ///
    /// This method should:
    /// - Stop accepting new messages
    /// - Complete any in-flight operations
    /// - Release resources
    /// - Disconnect from external systems
    ///
    /// # State Transition
    ///
    /// The component state should transition:
    /// `Running` -> `Stopping` -> `Stopped`
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::InvalidState` if called when component cannot be shut down.
    async fn shutdown(&mut self) -> Result<(), ComponentError>;

    /// Handles an incoming message.
    ///
    /// # Arguments
    ///
    /// * `message` - The raw message bytes to process
    ///
    /// # Returns
    ///
    /// Returns the response bytes on success, or a `ComponentError` on failure.
    ///
    /// # Errors
    ///
    /// - `ComponentError::InvalidState` if component is not in `Running` state
    /// - `ComponentError::MessageRoutingFailed` if message processing fails
    async fn handle_message(&mut self, message: &[u8]) -> Result<Vec<u8>, ComponentError>;

    /// Performs a health check on the component.
    ///
    /// This method should verify that the component is functioning correctly
    /// by checking internal state, resource availability, and dependencies.
    ///
    /// Health checks should be lightweight and fast (< 100ms).
    async fn health_check(&self) -> ComponentHealth;

    /// Returns current metrics for this component.
    ///
    /// Metrics should include standard counters (messages received/sent, errors)
    /// as well as any component-specific metrics.
    fn get_metrics(&self) -> ComponentMetrics;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test ComponentState

    #[test]
    fn test_component_state_default() {
        let state = ComponentState::default();
        assert_eq!(state, ComponentState::Created);
    }

    #[test]
    fn test_component_state_is_methods() {
        assert!(ComponentState::Created.is_created());
        assert!(ComponentState::Initializing.is_initializing());
        assert!(ComponentState::Running.is_running());
        assert!(ComponentState::Stopping.is_stopping());
        assert!(ComponentState::Stopped.is_stopped());
    }

    #[test]
    fn test_component_state_can_process_messages() {
        assert!(!ComponentState::Created.can_process_messages());
        assert!(!ComponentState::Initializing.can_process_messages());
        assert!(ComponentState::Running.can_process_messages());
        assert!(!ComponentState::Stopping.can_process_messages());
        assert!(!ComponentState::Stopped.can_process_messages());
    }

    #[test]
    fn test_component_state_can_initialize() {
        assert!(ComponentState::Created.can_initialize());
        assert!(!ComponentState::Initializing.can_initialize());
        assert!(!ComponentState::Running.can_initialize());
        assert!(!ComponentState::Stopping.can_initialize());
        assert!(!ComponentState::Stopped.can_initialize());
    }

    #[test]
    fn test_component_state_can_shutdown() {
        assert!(!ComponentState::Created.can_shutdown());
        assert!(ComponentState::Initializing.can_shutdown());
        assert!(ComponentState::Running.can_shutdown());
        assert!(!ComponentState::Stopping.can_shutdown());
        assert!(!ComponentState::Stopped.can_shutdown());
    }

    #[test]
    fn test_component_state_display() {
        assert_eq!(ComponentState::Created.to_string(), "Created");
        assert_eq!(ComponentState::Initializing.to_string(), "Initializing");
        assert_eq!(ComponentState::Running.to_string(), "Running");
        assert_eq!(ComponentState::Stopping.to_string(), "Stopping");
        assert_eq!(ComponentState::Stopped.to_string(), "Stopped");
    }

    #[test]
    fn test_component_state_serialization() {
        let state = ComponentState::Running;
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ComponentState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }

    // Test ComponentHealth

    #[test]
    fn test_component_health_constructors() {
        let healthy = ComponentHealth::healthy();
        assert!(healthy.is_healthy());

        let degraded = ComponentHealth::degraded("high memory");
        assert!(degraded.is_degraded());

        let unhealthy = ComponentHealth::unhealthy("crashed");
        assert!(unhealthy.is_unhealthy());
    }

    #[test]
    fn test_component_health_reason() {
        assert_eq!(ComponentHealth::Healthy.reason(), None);

        let degraded = ComponentHealth::degraded("memory pressure");
        assert_eq!(degraded.reason(), Some("memory pressure"));

        let unhealthy = ComponentHealth::unhealthy("connection lost");
        assert_eq!(unhealthy.reason(), Some("connection lost"));
    }

    #[test]
    fn test_component_health_default() {
        let health = ComponentHealth::default();
        assert!(health.is_healthy());
    }

    #[test]
    fn test_component_health_display() {
        assert_eq!(ComponentHealth::Healthy.to_string(), "Healthy");
        assert_eq!(
            ComponentHealth::degraded("slow").to_string(),
            "Degraded: slow"
        );
        assert_eq!(
            ComponentHealth::unhealthy("error").to_string(),
            "Unhealthy: error"
        );
    }

    #[test]
    fn test_component_health_serialization() {
        let health = ComponentHealth::degraded("test reason");
        let json = serde_json::to_string(&health).unwrap();
        let deserialized: ComponentHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(health, deserialized);
    }

    // Test ComponentMetrics

    #[test]
    fn test_component_metrics_new() {
        let metrics = ComponentMetrics::new("test_component");
        assert_eq!(metrics.component_name(), "test_component");
        assert_eq!(metrics.messages_received(), 0);
        assert_eq!(metrics.messages_sent(), 0);
        assert_eq!(metrics.errors_count(), 0);
    }

    #[test]
    fn test_component_metrics_increment_counters() {
        let mut metrics = ComponentMetrics::new("test");

        metrics.increment_messages_received();
        metrics.increment_messages_received();
        assert_eq!(metrics.messages_received(), 2);

        metrics.increment_messages_sent();
        assert_eq!(metrics.messages_sent(), 1);

        metrics.increment_errors();
        metrics.increment_errors();
        metrics.increment_errors();
        assert_eq!(metrics.errors_count(), 3);
    }

    #[test]
    fn test_component_metrics_processing_time() {
        let mut metrics = ComponentMetrics::new("test");

        metrics.update_processing_time(100);
        assert_eq!(metrics.avg_processing_time_us(), 100);

        // Second update should average
        metrics.increment_messages_received();
        metrics.update_processing_time(200);
        // (100 + 200) / 2 = 150
        assert_eq!(metrics.avg_processing_time_us(), 150);
    }

    #[test]
    fn test_component_metrics_custom_metrics() {
        let mut metrics = ComponentMetrics::new("test");

        metrics.set_custom_metric("active_windows", 5.0);
        metrics.set_custom_metric("tab_count", 10.0);

        assert_eq!(metrics.custom_metric("active_windows"), Some(&5.0));
        assert_eq!(metrics.custom_metric("tab_count"), Some(&10.0));
        assert_eq!(metrics.custom_metric("nonexistent"), None);

        assert_eq!(metrics.custom_metrics().len(), 2);
    }

    #[test]
    fn test_component_metrics_reset_counters() {
        let mut metrics = ComponentMetrics::new("test");

        metrics.increment_messages_received();
        metrics.increment_messages_sent();
        metrics.increment_errors();
        metrics.set_custom_metric("custom", 42.0);

        metrics.reset_counters();

        assert_eq!(metrics.messages_received(), 0);
        assert_eq!(metrics.messages_sent(), 0);
        assert_eq!(metrics.errors_count(), 0);
        // Custom metrics should be preserved
        assert_eq!(metrics.custom_metric("custom"), Some(&42.0));
    }

    #[test]
    fn test_component_metrics_default() {
        let metrics = ComponentMetrics::default();
        assert_eq!(metrics.component_name(), "unknown");
    }

    #[test]
    fn test_component_metrics_serialization() {
        let mut metrics = ComponentMetrics::new("test");
        metrics.increment_messages_received();
        metrics.set_custom_metric("test", 1.0);

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: ComponentMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(metrics.component_name(), deserialized.component_name());
        assert_eq!(metrics.messages_received(), deserialized.messages_received());
    }

    // Test BrowserComponent trait implementation

    struct MockComponent {
        name: String,
        state: ComponentState,
        metrics: ComponentMetrics,
    }

    impl MockComponent {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                state: ComponentState::Created,
                metrics: ComponentMetrics::new(name),
            }
        }
    }

    #[async_trait]
    impl BrowserComponent for MockComponent {
        fn name(&self) -> &str {
            &self.name
        }

        fn state(&self) -> ComponentState {
            self.state
        }

        async fn initialize(&mut self) -> Result<(), ComponentError> {
            if !self.state.can_initialize() {
                return Err(ComponentError::InvalidState(
                    "Cannot initialize in current state".to_string(),
                ));
            }
            self.state = ComponentState::Initializing;
            // Simulate some initialization work
            self.state = ComponentState::Running;
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<(), ComponentError> {
            if !self.state.can_shutdown() {
                return Err(ComponentError::InvalidState(
                    "Cannot shutdown in current state".to_string(),
                ));
            }
            self.state = ComponentState::Stopping;
            // Simulate some shutdown work
            self.state = ComponentState::Stopped;
            Ok(())
        }

        async fn handle_message(&mut self, message: &[u8]) -> Result<Vec<u8>, ComponentError> {
            if !self.state.can_process_messages() {
                return Err(ComponentError::InvalidState(
                    "Cannot process messages in current state".to_string(),
                ));
            }
            self.metrics.increment_messages_received();
            // Echo the message back
            let response = message.to_vec();
            self.metrics.increment_messages_sent();
            Ok(response)
        }

        async fn health_check(&self) -> ComponentHealth {
            if self.state.is_running() {
                ComponentHealth::Healthy
            } else {
                ComponentHealth::unhealthy(format!("Component is in {:?} state", self.state))
            }
        }

        fn get_metrics(&self) -> ComponentMetrics {
            self.metrics.clone()
        }
    }

    #[tokio::test]
    async fn test_browser_component_lifecycle() {
        let mut component = MockComponent::new("test_component");

        assert!(component.state().is_created());
        assert!(component.state().can_initialize());

        // Initialize
        component.initialize().await.unwrap();
        assert!(component.state().is_running());
        assert!(component.state().can_process_messages());

        // Handle a message
        let response = component.handle_message(b"hello").await.unwrap();
        assert_eq!(response, b"hello");

        // Check health
        let health = component.health_check().await;
        assert!(health.is_healthy());

        // Shutdown
        component.shutdown().await.unwrap();
        assert!(component.state().is_stopped());
    }

    #[tokio::test]
    async fn test_browser_component_invalid_state_transitions() {
        let mut component = MockComponent::new("test");

        // Cannot handle messages before initialization
        let result = component.handle_message(b"test").await;
        assert!(result.is_err());

        // Cannot shutdown before initialization
        let result = component.shutdown().await;
        assert!(result.is_err());

        // Initialize
        component.initialize().await.unwrap();

        // Cannot initialize again
        let result = component.initialize().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_browser_component_metrics_tracking() {
        let mut component = MockComponent::new("test");
        component.initialize().await.unwrap();

        // Send some messages
        for _ in 0..5 {
            component.handle_message(b"test").await.unwrap();
        }

        let metrics = component.get_metrics();
        assert_eq!(metrics.messages_received(), 5);
        assert_eq!(metrics.messages_sent(), 5);
    }
}
