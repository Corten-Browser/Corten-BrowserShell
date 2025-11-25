//! Security Audit Logging
//!
//! Provides comprehensive security event logging for audit trails,
//! compliance monitoring, and security incident investigation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Security event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Informational event
    Info,
    /// Low severity event
    Low,
    /// Medium severity event
    Medium,
    /// High severity event
    High,
    /// Critical security event
    Critical,
}

/// Security event categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventCategory {
    /// Authentication events (login, logout, failed attempts)
    Authentication,
    /// Authorization events (permission checks, access denials)
    Authorization,
    /// Input validation events (XSS attempts, SQL injection)
    InputValidation,
    /// Network security events (blocked connections, suspicious traffic)
    NetworkSecurity,
    /// Data access events (sensitive data access, modifications)
    DataAccess,
    /// Configuration changes
    ConfigurationChange,
    /// Sandbox violations
    SandboxViolation,
    /// IPC security events
    IPCSecurity,
    /// Cryptographic operations
    Cryptography,
    /// General security event
    General,
}

/// Security audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    /// Unique event ID
    pub event_id: String,
    /// Timestamp (seconds since UNIX epoch)
    pub timestamp_secs: u64,
    /// Event category
    pub category: EventCategory,
    /// Event severity
    pub severity: Severity,
    /// Event description
    pub description: String,
    /// Source component or process
    pub source: String,
    /// Target resource or component
    pub target: Option<String>,
    /// User or entity associated with event
    pub user: Option<String>,
    /// IP address (if applicable)
    pub ip_address: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Whether this was a successful operation
    pub success: bool,
}

impl SecurityEvent {
    /// Create a new security event
    pub fn new(
        category: EventCategory,
        severity: Severity,
        description: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};

        static EVENT_COUNTER: AtomicU64 = AtomicU64::new(0);
        let event_id = format!("SEC-{:08}", EVENT_COUNTER.fetch_add(1, Ordering::SeqCst));

        Self {
            event_id,
            timestamp_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            category,
            severity,
            description: description.into(),
            source: source.into(),
            target: None,
            user: None,
            ip_address: None,
            metadata: HashMap::new(),
            success: true,
        }
    }

    /// Set target
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Set user
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Set IP address
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set success flag
    pub fn with_success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }
}

/// Audit log filter criteria
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    /// Filter by category
    pub category: Option<EventCategory>,
    /// Filter by minimum severity
    pub min_severity: Option<Severity>,
    /// Filter by source
    pub source: Option<String>,
    /// Filter by target
    pub target: Option<String>,
    /// Filter by user
    pub user: Option<String>,
    /// Filter by success/failure
    pub success: Option<bool>,
    /// Filter by time range (start timestamp)
    pub start_time: Option<u64>,
    /// Filter by time range (end timestamp)
    pub end_time: Option<u64>,
}

impl AuditFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by category
    pub fn category(mut self, category: EventCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Filter by minimum severity
    pub fn min_severity(mut self, severity: Severity) -> Self {
        self.min_severity = Some(severity);
        self
    }

    /// Filter by source
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Filter by target
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Filter by user
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Filter by success status
    pub fn success(mut self, success: bool) -> Self {
        self.success = Some(success);
        self
    }

    /// Filter by time range
    pub fn time_range(mut self, start: u64, end: u64) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Check if an event matches this filter
    pub fn matches(&self, event: &SecurityEvent) -> bool {
        if let Some(cat) = self.category {
            if event.category != cat {
                return false;
            }
        }

        if let Some(min_sev) = self.min_severity {
            if event.severity < min_sev {
                return false;
            }
        }

        if let Some(ref source) = self.source {
            if &event.source != source {
                return false;
            }
        }

        if let Some(ref target) = self.target {
            if event.target.as_ref() != Some(target) {
                return false;
            }
        }

        if let Some(ref user) = self.user {
            if event.user.as_ref() != Some(user) {
                return false;
            }
        }

        if let Some(success) = self.success {
            if event.success != success {
                return false;
            }
        }

        if let Some(start) = self.start_time {
            if event.timestamp_secs < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if event.timestamp_secs > end {
                return false;
            }
        }

        true
    }
}

/// Security audit logger
pub struct AuditLogger {
    /// Event log
    events: Arc<RwLock<Vec<SecurityEvent>>>,
    /// Maximum events to keep in memory
    max_events: usize,
    /// Whether to automatically rotate logs
    auto_rotate: bool,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events: 10_000,
            auto_rotate: true,
        }
    }

    /// Create audit logger with custom max events
    pub fn with_capacity(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
            auto_rotate: true,
        }
    }

    /// Set auto-rotate behavior
    pub fn set_auto_rotate(&mut self, auto_rotate: bool) {
        self.auto_rotate = auto_rotate;
    }

    /// Log a security event
    pub async fn log(&self, event: SecurityEvent) {
        let mut events = self.events.write().await;

        // Auto-rotate if needed
        if self.auto_rotate && events.len() >= self.max_events {
            // Remove oldest 10% of events
            let remove_count = self.max_events / 10;
            events.drain(0..remove_count);
        }

        events.push(event);
    }

    /// Log a simple event
    pub async fn log_simple(
        &self,
        category: EventCategory,
        severity: Severity,
        description: impl Into<String>,
        source: impl Into<String>,
    ) {
        let event = SecurityEvent::new(category, severity, description, source);
        self.log(event).await;
    }

    /// Log an authentication event
    pub async fn log_auth(
        &self,
        description: impl Into<String>,
        user: impl Into<String>,
        success: bool,
    ) {
        let event = SecurityEvent::new(
            EventCategory::Authentication,
            if success { Severity::Info } else { Severity::Medium },
            description,
            "auth_system",
        )
        .with_user(user)
        .with_success(success);

        self.log(event).await;
    }

    /// Log an authorization event
    pub async fn log_authz(
        &self,
        description: impl Into<String>,
        user: impl Into<String>,
        resource: impl Into<String>,
        success: bool,
    ) {
        let event = SecurityEvent::new(
            EventCategory::Authorization,
            if success { Severity::Info } else { Severity::Medium },
            description,
            "authz_system",
        )
        .with_user(user)
        .with_target(resource)
        .with_success(success);

        self.log(event).await;
    }

    /// Log a validation failure
    pub async fn log_validation_failure(
        &self,
        description: impl Into<String>,
        source: impl Into<String>,
        attack_type: impl Into<String>,
    ) {
        let event = SecurityEvent::new(
            EventCategory::InputValidation,
            Severity::High,
            description,
            source,
        )
        .with_metadata("attack_type", attack_type)
        .with_success(false);

        self.log(event).await;
    }

    /// Log a sandbox violation
    pub async fn log_sandbox_violation(
        &self,
        process_id: impl Into<String>,
        violation: impl Into<String>,
    ) {
        let event = SecurityEvent::new(
            EventCategory::SandboxViolation,
            Severity::High,
            violation,
            process_id,
        )
        .with_success(false);

        self.log(event).await;
    }

    /// Log a network security event
    pub async fn log_network_event(
        &self,
        description: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        severity: Severity,
    ) {
        let event = SecurityEvent::new(
            EventCategory::NetworkSecurity,
            severity,
            description,
            source,
        )
        .with_target(target);

        self.log(event).await;
    }

    /// Log a configuration change
    pub async fn log_config_change(
        &self,
        description: impl Into<String>,
        user: impl Into<String>,
        config_key: impl Into<String>,
    ) {
        let event = SecurityEvent::new(
            EventCategory::ConfigurationChange,
            Severity::Medium,
            description,
            "config_system",
        )
        .with_user(user)
        .with_metadata("config_key", config_key);

        self.log(event).await;
    }

    /// Get all events
    pub async fn get_events(&self) -> Vec<SecurityEvent> {
        self.events.read().await.clone()
    }

    /// Get events matching a filter
    pub async fn get_filtered_events(&self, filter: &AuditFilter) -> Vec<SecurityEvent> {
        self.events
            .read()
            .await
            .iter()
            .filter(|e| filter.matches(e))
            .cloned()
            .collect()
    }

    /// Get events by category
    pub async fn get_events_by_category(&self, category: EventCategory) -> Vec<SecurityEvent> {
        let filter = AuditFilter::new().category(category);
        self.get_filtered_events(&filter).await
    }

    /// Get events by severity
    pub async fn get_events_by_severity(&self, min_severity: Severity) -> Vec<SecurityEvent> {
        let filter = AuditFilter::new().min_severity(min_severity);
        self.get_filtered_events(&filter).await
    }

    /// Get failed events
    pub async fn get_failed_events(&self) -> Vec<SecurityEvent> {
        let filter = AuditFilter::new().success(false);
        self.get_filtered_events(&filter).await
    }

    /// Get event count
    pub async fn event_count(&self) -> usize {
        self.events.read().await.len()
    }

    /// Clear all events
    pub async fn clear(&self) {
        self.events.write().await.clear();
    }

    /// Get events in time range
    pub async fn get_events_in_range(&self, start: u64, end: u64) -> Vec<SecurityEvent> {
        let filter = AuditFilter::new().time_range(start, end);
        self.get_filtered_events(&filter).await
    }

    /// Get recent events
    pub async fn get_recent_events(&self, count: usize) -> Vec<SecurityEvent> {
        let events = self.events.read().await;
        let start = events.len().saturating_sub(count);
        events[start..].to_vec()
    }

    /// Get summary statistics
    pub async fn get_summary(&self) -> AuditSummary {
        let events = self.events.read().await;

        let mut summary = AuditSummary {
            total_events: events.len(),
            by_category: HashMap::new(),
            by_severity: HashMap::new(),
            failed_events: 0,
        };

        for event in events.iter() {
            *summary.by_category.entry(event.category).or_insert(0) += 1;
            *summary.by_severity.entry(event.severity).or_insert(0) += 1;

            if !event.success {
                summary.failed_events += 1;
            }
        }

        summary
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit log summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    /// Total number of events
    pub total_events: usize,
    /// Event counts by category
    pub by_category: HashMap<EventCategory, usize>,
    /// Event counts by severity
    pub by_severity: HashMap<Severity, usize>,
    /// Number of failed events
    pub failed_events: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_event_creation() {
        let event = SecurityEvent::new(
            EventCategory::Authentication,
            Severity::Info,
            "User logged in",
            "auth_module",
        );

        assert_eq!(event.category, EventCategory::Authentication);
        assert_eq!(event.severity, Severity::Info);
        assert_eq!(event.description, "User logged in");
        assert_eq!(event.source, "auth_module");
        assert!(event.success);
    }

    #[tokio::test]
    async fn test_security_event_builder() {
        let event = SecurityEvent::new(
            EventCategory::Authorization,
            Severity::Medium,
            "Access denied",
            "authz_system",
        )
        .with_user("john_doe")
        .with_target("/admin/settings")
        .with_ip("192.168.1.100")
        .with_metadata("reason", "insufficient_permissions")
        .with_success(false);

        assert_eq!(event.user, Some("john_doe".to_string()));
        assert_eq!(event.target, Some("/admin/settings".to_string()));
        assert_eq!(event.ip_address, Some("192.168.1.100".to_string()));
        assert_eq!(event.metadata.get("reason"), Some(&"insufficient_permissions".to_string()));
        assert!(!event.success);
    }

    #[tokio::test]
    async fn test_audit_logger_basic() {
        let logger = AuditLogger::new();

        logger
            .log_simple(
                EventCategory::Authentication,
                Severity::Info,
                "User login",
                "auth",
            )
            .await;

        let events = logger.get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].description, "User login");
    }

    #[tokio::test]
    async fn test_log_auth() {
        let logger = AuditLogger::new();

        logger.log_auth("Successful login", "john_doe", true).await;
        logger.log_auth("Failed login", "jane_smith", false).await;

        let events = logger.get_events().await;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].category, EventCategory::Authentication);
        assert!(events[0].success);
        assert!(!events[1].success);
    }

    #[tokio::test]
    async fn test_filter_by_category() {
        let logger = AuditLogger::new();

        logger
            .log_simple(EventCategory::Authentication, Severity::Info, "Login", "auth")
            .await;
        logger
            .log_simple(EventCategory::NetworkSecurity, Severity::High, "Block", "network")
            .await;

        let auth_events = logger
            .get_events_by_category(EventCategory::Authentication)
            .await;
        assert_eq!(auth_events.len(), 1);
        assert_eq!(auth_events[0].category, EventCategory::Authentication);
    }

    #[tokio::test]
    async fn test_filter_by_severity() {
        let logger = AuditLogger::new();

        logger
            .log_simple(EventCategory::General, Severity::Info, "Info event", "test")
            .await;
        logger
            .log_simple(EventCategory::General, Severity::High, "High event", "test")
            .await;
        logger
            .log_simple(EventCategory::General, Severity::Critical, "Critical event", "test")
            .await;

        let high_severity = logger.get_events_by_severity(Severity::High).await;
        assert_eq!(high_severity.len(), 2); // High and Critical
    }

    #[tokio::test]
    async fn test_get_failed_events() {
        let logger = AuditLogger::new();

        logger.log_auth("Success", "user1", true).await;
        logger.log_auth("Failed", "user2", false).await;
        logger.log_auth("Failed", "user3", false).await;

        let failed = logger.get_failed_events().await;
        assert_eq!(failed.len(), 2);
        assert!(!failed[0].success);
    }

    #[tokio::test]
    async fn test_audit_filter() {
        let filter = AuditFilter::new()
            .category(EventCategory::Authentication)
            .min_severity(Severity::Medium)
            .source("auth_system")
            .success(false);

        let matching_event = SecurityEvent::new(
            EventCategory::Authentication,
            Severity::High,
            "Failed login",
            "auth_system",
        )
        .with_success(false);

        let non_matching_event = SecurityEvent::new(
            EventCategory::NetworkSecurity,
            Severity::Info,
            "Connection",
            "network",
        );

        assert!(filter.matches(&matching_event));
        assert!(!filter.matches(&non_matching_event));
    }

    #[tokio::test]
    async fn test_auto_rotate() {
        let logger = AuditLogger::with_capacity(10);

        // Add more than capacity
        for i in 0..15 {
            logger
                .log_simple(EventCategory::General, Severity::Info, format!("Event {}", i), "test")
                .await;
        }

        // Should have rotated
        let events = logger.get_events().await;
        assert!(events.len() <= 10);
    }

    #[tokio::test]
    async fn test_get_summary() {
        let logger = AuditLogger::new();

        logger.log_auth("Login", "user1", true).await;
        logger.log_auth("Failed login", "user2", false).await;
        logger
            .log_network_event("Blocked connection", "firewall", "malicious.com", Severity::High)
            .await;

        let summary = logger.get_summary().await;

        assert_eq!(summary.total_events, 3);
        assert_eq!(summary.failed_events, 1);
        assert_eq!(
            *summary.by_category.get(&EventCategory::Authentication).unwrap(),
            2
        );
    }

    #[tokio::test]
    async fn test_clear_events() {
        let logger = AuditLogger::new();

        logger.log_simple(EventCategory::General, Severity::Info, "Test", "test").await;
        assert_eq!(logger.event_count().await, 1);

        logger.clear().await;
        assert_eq!(logger.event_count().await, 0);
    }

    #[tokio::test]
    async fn test_get_recent_events() {
        let logger = AuditLogger::new();

        for i in 0..10 {
            logger
                .log_simple(EventCategory::General, Severity::Info, format!("Event {}", i), "test")
                .await;
        }

        let recent = logger.get_recent_events(3).await;
        assert_eq!(recent.len(), 3);
        assert!(recent[0].description.contains("Event 7"));
    }

    #[tokio::test]
    async fn test_log_validation_failure() {
        let logger = AuditLogger::new();

        logger
            .log_validation_failure("XSS attempt detected", "web_input", "xss")
            .await;

        let events = logger.get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].category, EventCategory::InputValidation);
        assert_eq!(events[0].severity, Severity::High);
        assert!(!events[0].success);
    }

    #[tokio::test]
    async fn test_log_sandbox_violation() {
        let logger = AuditLogger::new();

        logger
            .log_sandbox_violation("proc_123", "Attempted to access forbidden path")
            .await;

        let events = logger.get_events().await;
        assert_eq!(events[0].category, EventCategory::SandboxViolation);
        assert_eq!(events[0].severity, Severity::High);
    }

    #[tokio::test]
    async fn test_log_config_change() {
        let logger = AuditLogger::new();

        logger
            .log_config_change("Changed security timeout", "admin", "security.timeout")
            .await;

        let events = logger.get_events().await;
        assert_eq!(events[0].category, EventCategory::ConfigurationChange);
        assert_eq!(events[0].user, Some("admin".to_string()));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Low);
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }
}
