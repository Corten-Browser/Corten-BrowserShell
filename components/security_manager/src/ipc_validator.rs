//! IPC Message Validation
//!
//! Provides validation for inter-process communication messages including:
//! - Message source verification (component identity)
//! - Size limits (configurable max message size)
//! - Content validation (script injection prevention)
//! - Permission checking (component-to-component messaging)
//! - Rate limiting (token bucket algorithm)
//! - Suspicious message logging

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Errors that can occur during IPC message validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IpcValidationError {
    /// Message source component is not registered/known
    UnknownSource(String),

    /// Message exceeds maximum allowed size
    MessageTooLarge { actual: usize, max: usize },

    /// Message contains potentially dangerous content (script injection)
    DangerousContent(String),

    /// Source component lacks permission to send this message type
    PermissionDenied {
        source: String,
        target: String,
        message_type: String,
    },

    /// Rate limit exceeded for source component
    RateLimitExceeded { component: String, message: String },

    /// Message payload is malformed
    MalformedMessage(String),
}

impl std::fmt::Display for IpcValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownSource(source) => write!(f, "Unknown message source: {}", source),
            Self::MessageTooLarge { actual, max } => {
                write!(f, "Message size {} bytes exceeds maximum {} bytes", actual, max)
            }
            Self::DangerousContent(msg) => write!(f, "Message content validation failed: {}", msg),
            Self::PermissionDenied { source, target, message_type } => {
                write!(f, "Permission denied: {} cannot send {} to {}", source, message_type, target)
            }
            Self::RateLimitExceeded { component, message } => {
                write!(f, "Rate limit exceeded for component {}: {}", component, message)
            }
            Self::MalformedMessage(msg) => write!(f, "Malformed message: {}", msg),
        }
    }
}

impl std::error::Error for IpcValidationError {}

/// Permission level for component-to-component messaging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentPermission {
    /// No access allowed
    None,
    /// Read-only access (receive messages only)
    ReadOnly,
    /// Write-only access (send messages only)
    WriteOnly,
    /// Full access (send and receive)
    Full,
}

impl Default for ComponentPermission {
    fn default() -> Self {
        Self::None
    }
}

/// Rate limiting configuration using token bucket algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of tokens in the bucket
    pub max_tokens: u32,
    /// Rate at which tokens are refilled (tokens per second)
    pub refill_rate: f64,
    /// Initial number of tokens
    pub initial_tokens: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_tokens: 100,
            refill_rate: 10.0, // 10 tokens per second
            initial_tokens: 100,
        }
    }
}

/// Token bucket state for rate limiting
#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    config: RateLimitConfig,
}

impl TokenBucket {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            tokens: config.initial_tokens as f64,
            last_refill: Instant::now(),
            config,
        }
    }

    /// Try to consume a token. Returns true if successful, false if rate limited.
    fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let new_tokens = elapsed * self.config.refill_rate;
        self.tokens = (self.tokens + new_tokens).min(self.config.max_tokens as f64);
        self.last_refill = now;
    }

    /// Get remaining tokens (for testing/monitoring)
    fn remaining_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }
}

/// Configuration for message validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageValidationConfig {
    /// Maximum allowed message size in bytes
    pub max_message_size: usize,
    /// Maximum allowed payload size in bytes (for nested data)
    pub max_payload_size: usize,
    /// Enable content validation (script injection detection)
    pub enable_content_validation: bool,
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Rate limit configuration
    pub rate_limit_config: RateLimitConfig,
    /// Log suspicious messages
    pub log_suspicious: bool,
    /// Known dangerous patterns to detect
    #[serde(skip)]
    pub dangerous_patterns: Vec<String>,
}

impl Default for MessageValidationConfig {
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024,      // 1 MB default
            max_payload_size: 512 * 1024,       // 512 KB default
            enable_content_validation: true,
            enable_rate_limiting: true,
            rate_limit_config: RateLimitConfig::default(),
            log_suspicious: true,
            dangerous_patterns: vec![
                r"(?i)<script".to_string(),
                r"(?i)javascript:".to_string(),
                r"(?i)on\w+\s*=".to_string(),
                r#"(?i)eval\s*\("#.to_string(),
                r"(?i)document\.cookie".to_string(),
                r"(?i)document\.write".to_string(),
                r"(?i)innerHTML\s*=".to_string(),
                r"(?i)\.exec\s*\(".to_string(),
            ],
        }
    }
}

/// IPC message for validation
#[derive(Debug, Clone)]
pub struct IpcMessage {
    /// Source component identifier
    pub source: String,
    /// Target component identifier
    pub target: String,
    /// Message type/action
    pub message_type: String,
    /// Message payload (serialized data)
    pub payload: Vec<u8>,
    /// Optional string payload for content validation
    pub string_payload: Option<String>,
}

impl IpcMessage {
    /// Create a new IPC message
    pub fn new(source: impl Into<String>, target: impl Into<String>, message_type: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            message_type: message_type.into(),
            payload: Vec::new(),
            string_payload: None,
        }
    }

    /// Set the binary payload
    pub fn with_payload(mut self, payload: Vec<u8>) -> Self {
        self.payload = payload;
        self
    }

    /// Set the string payload (for content validation)
    pub fn with_string_payload(mut self, payload: impl Into<String>) -> Self {
        let s = payload.into();
        self.payload = s.as_bytes().to_vec();
        self.string_payload = Some(s);
        self
    }

    /// Get total message size (approximate)
    pub fn total_size(&self) -> usize {
        self.source.len() + self.target.len() + self.message_type.len() + self.payload.len()
    }
}

/// Log entry for suspicious messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousMessageLog {
    /// Timestamp (as duration since UNIX epoch)
    pub timestamp_secs: u64,
    /// Source component
    pub source: String,
    /// Target component
    pub target: String,
    /// Message type
    pub message_type: String,
    /// Reason for flagging
    pub reason: String,
    /// Truncated payload preview (first 100 chars)
    pub payload_preview: String,
}

/// Trait for message validation that MessageBus can use
pub trait MessageValidator: Send + Sync {
    /// Validate an IPC message
    ///
    /// Returns Ok(()) if the message is valid, or an error describing why it's invalid.
    fn validate_message(&self, message: &IpcMessage) -> impl std::future::Future<Output = Result<(), IpcValidationError>> + Send;

    /// Check if a component can send a specific message type to another component
    fn can_send(&self, source: &str, target: &str, message_type: &str) -> impl std::future::Future<Output = bool> + Send;

    /// Record a suspicious message for logging/auditing
    fn log_suspicious(&self, message: &IpcMessage, reason: &str) -> impl std::future::Future<Output = ()> + Send;
}

/// IPC message validator with rate limiting and permission checking
pub struct IpcValidator {
    /// Configuration
    config: Arc<RwLock<MessageValidationConfig>>,
    /// Registered components (known valid sources)
    registered_components: Arc<RwLock<HashSet<String>>>,
    /// Permission matrix: (source, target) -> allowed message types
    /// If a source-target pair is present, only listed message types are allowed
    /// If absent and both components are registered, all types are allowed (permissive default)
    permission_matrix: Arc<RwLock<HashMap<(String, String), HashSet<String>>>>,
    /// Component-level permissions
    component_permissions: Arc<RwLock<HashMap<String, ComponentPermission>>>,
    /// Rate limiters per component
    rate_limiters: Arc<RwLock<HashMap<String, TokenBucket>>>,
    /// Compiled dangerous patterns
    dangerous_patterns: Vec<Regex>,
    /// Suspicious message log
    suspicious_log: Arc<RwLock<Vec<SuspiciousMessageLog>>>,
}

impl IpcValidator {
    /// Create a new IpcValidator with default configuration
    pub fn new() -> Self {
        Self::with_config(MessageValidationConfig::default())
    }

    /// Create a new IpcValidator with custom configuration
    pub fn with_config(config: MessageValidationConfig) -> Self {
        // Compile dangerous patterns
        let dangerous_patterns: Vec<Regex> = config
            .dangerous_patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        Self {
            config: Arc::new(RwLock::new(config)),
            registered_components: Arc::new(RwLock::new(HashSet::new())),
            permission_matrix: Arc::new(RwLock::new(HashMap::new())),
            component_permissions: Arc::new(RwLock::new(HashMap::new())),
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            dangerous_patterns,
            suspicious_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a component as a known message source/target
    pub async fn register_component(&self, component_id: impl Into<String>, permission: ComponentPermission) {
        let id = component_id.into();
        let config = self.config.read().await;

        self.registered_components.write().await.insert(id.clone());
        self.component_permissions.write().await.insert(id.clone(), permission);

        // Initialize rate limiter for this component
        if config.enable_rate_limiting {
            self.rate_limiters
                .write()
                .await
                .insert(id, TokenBucket::new(config.rate_limit_config.clone()));
        }
    }

    /// Unregister a component
    pub async fn unregister_component(&self, component_id: &str) {
        self.registered_components.write().await.remove(component_id);
        self.component_permissions.write().await.remove(component_id);
        self.rate_limiters.write().await.remove(component_id);
    }

    /// Set allowed message types between two components
    /// If not set, all message types are allowed between registered components
    pub async fn set_allowed_messages(
        &self,
        source: impl Into<String>,
        target: impl Into<String>,
        message_types: HashSet<String>,
    ) {
        let key = (source.into(), target.into());
        self.permission_matrix.write().await.insert(key, message_types);
    }

    /// Add an allowed message type for a source-target pair
    pub async fn allow_message_type(
        &self,
        source: impl Into<String>,
        target: impl Into<String>,
        message_type: impl Into<String>,
    ) {
        let source = source.into();
        let target = target.into();
        let msg_type = message_type.into();

        let mut matrix = self.permission_matrix.write().await;
        matrix
            .entry((source, target))
            .or_insert_with(HashSet::new)
            .insert(msg_type);
    }

    /// Update the validation configuration
    pub async fn set_config(&self, config: MessageValidationConfig) {
        *self.config.write().await = config;
    }

    /// Get the current configuration
    pub async fn get_config(&self) -> MessageValidationConfig {
        self.config.read().await.clone()
    }

    /// Get suspicious message log
    pub async fn get_suspicious_log(&self) -> Vec<SuspiciousMessageLog> {
        self.suspicious_log.read().await.clone()
    }

    /// Clear suspicious message log
    pub async fn clear_suspicious_log(&self) {
        self.suspicious_log.write().await.clear();
    }

    /// Check if source is a registered component
    async fn validate_source(&self, source: &str) -> Result<(), IpcValidationError> {
        let components = self.registered_components.read().await;
        if !components.contains(source) {
            return Err(IpcValidationError::UnknownSource(source.to_string()));
        }
        Ok(())
    }

    /// Check message size limits
    async fn validate_size(&self, message: &IpcMessage) -> Result<(), IpcValidationError> {
        let config = self.config.read().await;
        let total_size = message.total_size();

        if total_size > config.max_message_size {
            return Err(IpcValidationError::MessageTooLarge {
                actual: total_size,
                max: config.max_message_size,
            });
        }

        if message.payload.len() > config.max_payload_size {
            return Err(IpcValidationError::MessageTooLarge {
                actual: message.payload.len(),
                max: config.max_payload_size,
            });
        }

        Ok(())
    }

    /// Check for dangerous content patterns
    fn validate_content(&self, message: &IpcMessage) -> Result<(), IpcValidationError> {
        if let Some(ref content) = message.string_payload {
            for pattern in &self.dangerous_patterns {
                if pattern.is_match(content) {
                    return Err(IpcValidationError::DangerousContent(
                        format!("Potentially malicious pattern detected in message content"),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Check component permissions
    async fn validate_permission(&self, message: &IpcMessage) -> Result<(), IpcValidationError> {
        let perms = self.component_permissions.read().await;

        // Check source has write permission
        if let Some(perm) = perms.get(&message.source) {
            match perm {
                ComponentPermission::None | ComponentPermission::ReadOnly => {
                    return Err(IpcValidationError::PermissionDenied {
                        source: message.source.clone(),
                        target: message.target.clone(),
                        message_type: message.message_type.clone(),
                    });
                }
                _ => {}
            }
        }

        // Check target can receive
        if let Some(perm) = perms.get(&message.target) {
            match perm {
                ComponentPermission::None | ComponentPermission::WriteOnly => {
                    return Err(IpcValidationError::PermissionDenied {
                        source: message.source.clone(),
                        target: message.target.clone(),
                        message_type: message.message_type.clone(),
                    });
                }
                _ => {}
            }
        }

        // Check message type permission matrix
        let matrix = self.permission_matrix.read().await;
        let key = (message.source.clone(), message.target.clone());

        if let Some(allowed_types) = matrix.get(&key) {
            // If there's a specific rule, message type must be in the allowed set
            if !allowed_types.contains(&message.message_type) {
                return Err(IpcValidationError::PermissionDenied {
                    source: message.source.clone(),
                    target: message.target.clone(),
                    message_type: message.message_type.clone(),
                });
            }
        }
        // If no specific rule exists, allow (permissive by default for registered components)

        Ok(())
    }

    /// Check rate limit for source component
    async fn check_rate_limit(&self, source: &str) -> Result<(), IpcValidationError> {
        let config = self.config.read().await;
        if !config.enable_rate_limiting {
            return Ok(());
        }
        drop(config);

        let mut limiters = self.rate_limiters.write().await;
        if let Some(bucket) = limiters.get_mut(source) {
            if !bucket.try_consume() {
                return Err(IpcValidationError::RateLimitExceeded {
                    component: source.to_string(),
                    message: format!(
                        "Too many messages. Remaining tokens: {:.1}",
                        bucket.remaining_tokens()
                    ),
                });
            }
        }
        Ok(())
    }

    /// Log a suspicious message
    async fn log_suspicious_internal(&self, message: &IpcMessage, reason: &str) {
        let config = self.config.read().await;
        if !config.log_suspicious {
            return;
        }
        drop(config);

        let payload_preview = message
            .string_payload
            .as_ref()
            .map(|s| {
                if s.len() > 100 {
                    format!("{}...", &s[..100])
                } else {
                    s.clone()
                }
            })
            .unwrap_or_else(|| format!("<binary {} bytes>", message.payload.len()));

        let log_entry = SuspiciousMessageLog {
            timestamp_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            source: message.source.clone(),
            target: message.target.clone(),
            message_type: message.message_type.clone(),
            reason: reason.to_string(),
            payload_preview,
        };

        self.suspicious_log.write().await.push(log_entry);
    }

    /// Get remaining rate limit tokens for a component
    pub async fn get_remaining_tokens(&self, component: &str) -> Option<f64> {
        let mut limiters = self.rate_limiters.write().await;
        limiters.get_mut(component).map(|b| b.remaining_tokens())
    }
}

impl Default for IpcValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageValidator for IpcValidator {
    async fn validate_message(&self, message: &IpcMessage) -> Result<(), IpcValidationError> {
        // Step 1: Validate source is registered
        self.validate_source(&message.source).await?;

        // Step 2: Check rate limit
        self.check_rate_limit(&message.source).await?;

        // Step 3: Validate message size
        self.validate_size(message).await?;

        // Step 4: Validate content (if enabled)
        let config = self.config.read().await;
        if config.enable_content_validation {
            drop(config);
            if let Err(e) = self.validate_content(message) {
                self.log_suspicious_internal(message, &e.to_string()).await;
                return Err(e);
            }
        }

        // Step 5: Validate permissions
        self.validate_permission(message).await?;

        Ok(())
    }

    async fn can_send(&self, source: &str, target: &str, message_type: &str) -> bool {
        // Check if both components are registered
        let components = self.registered_components.read().await;
        if !components.contains(source) || !components.contains(target) {
            return false;
        }
        drop(components);

        // Check permissions
        let perms = self.component_permissions.read().await;

        // Source must have write permission
        if let Some(perm) = perms.get(source) {
            match perm {
                ComponentPermission::None | ComponentPermission::ReadOnly => return false,
                _ => {}
            }
        }

        // Target must have read permission
        if let Some(perm) = perms.get(target) {
            match perm {
                ComponentPermission::None | ComponentPermission::WriteOnly => return false,
                _ => {}
            }
        }
        drop(perms);

        // Check permission matrix
        let matrix = self.permission_matrix.read().await;
        let key = (source.to_string(), target.to_string());

        if let Some(allowed_types) = matrix.get(&key) {
            return allowed_types.contains(message_type);
        }

        // No specific restriction, allow by default
        true
    }

    async fn log_suspicious(&self, message: &IpcMessage, reason: &str) {
        self.log_suspicious_internal(message, reason).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_component() {
        let validator = IpcValidator::new();
        validator.register_component("comp_a", ComponentPermission::Full).await;

        let message = IpcMessage::new("comp_a", "comp_b", "test");
        let result = validator.validate_source(&message.source).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unknown_source_rejected() {
        let validator = IpcValidator::new();

        let message = IpcMessage::new("unknown", "comp_b", "test");
        let result = validator.validate_message(&message).await;
        assert!(matches!(result, Err(IpcValidationError::UnknownSource(_))));
    }

    #[tokio::test]
    async fn test_message_size_limit() {
        let mut config = MessageValidationConfig::default();
        config.max_message_size = 100;
        config.enable_rate_limiting = false;

        let validator = IpcValidator::with_config(config);
        validator.register_component("comp_a", ComponentPermission::Full).await;
        validator.register_component("comp_b", ComponentPermission::Full).await;

        let message = IpcMessage::new("comp_a", "comp_b", "test")
            .with_payload(vec![0u8; 200]);

        let result = validator.validate_message(&message).await;
        assert!(matches!(result, Err(IpcValidationError::MessageTooLarge { .. })));
    }

    #[tokio::test]
    async fn test_dangerous_content_detection() {
        let validator = IpcValidator::new();
        validator.register_component("comp_a", ComponentPermission::Full).await;
        validator.register_component("comp_b", ComponentPermission::Full).await;

        let message = IpcMessage::new("comp_a", "comp_b", "test")
            .with_string_payload("<script>alert('xss')</script>");

        let result = validator.validate_message(&message).await;
        assert!(matches!(result, Err(IpcValidationError::DangerousContent(_))));
    }

    #[tokio::test]
    async fn test_dangerous_javascript_uri() {
        let validator = IpcValidator::new();
        validator.register_component("comp_a", ComponentPermission::Full).await;
        validator.register_component("comp_b", ComponentPermission::Full).await;

        let message = IpcMessage::new("comp_a", "comp_b", "navigate")
            .with_string_payload("javascript:document.cookie");

        let result = validator.validate_message(&message).await;
        assert!(matches!(result, Err(IpcValidationError::DangerousContent(_))));
    }

    #[tokio::test]
    async fn test_permission_matrix() {
        let mut config = MessageValidationConfig::default();
        config.enable_rate_limiting = false;

        let validator = IpcValidator::with_config(config);
        validator.register_component("comp_a", ComponentPermission::Full).await;
        validator.register_component("comp_b", ComponentPermission::Full).await;

        // Only allow "read" message type from comp_a to comp_b
        let mut allowed = HashSet::new();
        allowed.insert("read".to_string());
        validator.set_allowed_messages("comp_a", "comp_b", allowed).await;

        // "read" should be allowed
        let msg_read = IpcMessage::new("comp_a", "comp_b", "read");
        assert!(validator.validate_message(&msg_read).await.is_ok());

        // "write" should be denied
        let msg_write = IpcMessage::new("comp_a", "comp_b", "write");
        let result = validator.validate_message(&msg_write).await;
        assert!(matches!(result, Err(IpcValidationError::PermissionDenied { .. })));
    }

    #[tokio::test]
    async fn test_read_only_component_cannot_send() {
        let mut config = MessageValidationConfig::default();
        config.enable_rate_limiting = false;

        let validator = IpcValidator::with_config(config);
        validator.register_component("reader", ComponentPermission::ReadOnly).await;
        validator.register_component("receiver", ComponentPermission::Full).await;

        let message = IpcMessage::new("reader", "receiver", "test");
        let result = validator.validate_message(&message).await;
        assert!(matches!(result, Err(IpcValidationError::PermissionDenied { .. })));
    }

    #[tokio::test]
    async fn test_write_only_component_cannot_receive() {
        let mut config = MessageValidationConfig::default();
        config.enable_rate_limiting = false;

        let validator = IpcValidator::with_config(config);
        validator.register_component("sender", ComponentPermission::Full).await;
        validator.register_component("writer", ComponentPermission::WriteOnly).await;

        let message = IpcMessage::new("sender", "writer", "test");
        let result = validator.validate_message(&message).await;
        assert!(matches!(result, Err(IpcValidationError::PermissionDenied { .. })));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mut config = MessageValidationConfig::default();
        config.rate_limit_config = RateLimitConfig {
            max_tokens: 5,
            refill_rate: 0.0, // No refill for testing
            initial_tokens: 5,
        };

        let validator = IpcValidator::with_config(config);
        validator.register_component("comp_a", ComponentPermission::Full).await;
        validator.register_component("comp_b", ComponentPermission::Full).await;

        // Should allow 5 messages
        for i in 0..5 {
            let message = IpcMessage::new("comp_a", "comp_b", format!("test_{}", i));
            assert!(validator.validate_message(&message).await.is_ok(), "Message {} should succeed", i);
        }

        // 6th message should be rate limited
        let message = IpcMessage::new("comp_a", "comp_b", "test_6");
        let result = validator.validate_message(&message).await;
        assert!(matches!(result, Err(IpcValidationError::RateLimitExceeded { .. })));
    }

    #[tokio::test]
    async fn test_can_send() {
        let mut config = MessageValidationConfig::default();
        config.enable_rate_limiting = false;

        let validator = IpcValidator::with_config(config);
        validator.register_component("comp_a", ComponentPermission::Full).await;
        validator.register_component("comp_b", ComponentPermission::Full).await;
        validator.register_component("reader", ComponentPermission::ReadOnly).await;

        // Full permission can send
        assert!(validator.can_send("comp_a", "comp_b", "any_type").await);

        // ReadOnly cannot send
        assert!(!validator.can_send("reader", "comp_b", "any_type").await);

        // Unknown component cannot send
        assert!(!validator.can_send("unknown", "comp_b", "any_type").await);
    }

    #[tokio::test]
    async fn test_suspicious_logging() {
        let validator = IpcValidator::new();
        validator.register_component("comp_a", ComponentPermission::Full).await;
        validator.register_component("comp_b", ComponentPermission::Full).await;

        let message = IpcMessage::new("comp_a", "comp_b", "test")
            .with_string_payload("<script>malicious</script>");

        // This should fail and log
        let _ = validator.validate_message(&message).await;

        let log = validator.get_suspicious_log().await;
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].source, "comp_a");
        assert!(log[0].reason.contains("malicious pattern"));
    }

    #[tokio::test]
    async fn test_valid_message_passes() {
        let mut config = MessageValidationConfig::default();
        config.enable_rate_limiting = false;

        let validator = IpcValidator::with_config(config);
        validator.register_component("window_manager", ComponentPermission::Full).await;
        validator.register_component("tab_manager", ComponentPermission::Full).await;

        let message = IpcMessage::new("window_manager", "tab_manager", "CreateTab")
            .with_string_payload(r#"{"window_id": "123", "url": "https://example.com"}"#);

        let result = validator.validate_message(&message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_allow_message_type() {
        let mut config = MessageValidationConfig::default();
        config.enable_rate_limiting = false;

        let validator = IpcValidator::with_config(config);
        validator.register_component("comp_a", ComponentPermission::Full).await;
        validator.register_component("comp_b", ComponentPermission::Full).await;

        // Initially allow only "CreateTab"
        validator.allow_message_type("comp_a", "comp_b", "CreateTab").await;

        // Then also allow "CloseTab"
        validator.allow_message_type("comp_a", "comp_b", "CloseTab").await;

        // Both should work
        let msg1 = IpcMessage::new("comp_a", "comp_b", "CreateTab");
        assert!(validator.validate_message(&msg1).await.is_ok());

        let msg2 = IpcMessage::new("comp_a", "comp_b", "CloseTab");
        assert!(validator.validate_message(&msg2).await.is_ok());

        // But not others
        let msg3 = IpcMessage::new("comp_a", "comp_b", "DeleteWindow");
        assert!(matches!(
            validator.validate_message(&msg3).await,
            Err(IpcValidationError::PermissionDenied { .. })
        ));
    }

    #[tokio::test]
    async fn test_unregister_component() {
        let mut config = MessageValidationConfig::default();
        config.enable_rate_limiting = false;

        let validator = IpcValidator::with_config(config);
        validator.register_component("comp_a", ComponentPermission::Full).await;

        // Should be known
        assert!(validator.validate_source("comp_a").await.is_ok());

        // Unregister
        validator.unregister_component("comp_a").await;

        // Should now be unknown
        assert!(matches!(
            validator.validate_source("comp_a").await,
            Err(IpcValidationError::UnknownSource(_))
        ));
    }

    #[tokio::test]
    async fn test_event_handler_injection() {
        let validator = IpcValidator::new();
        validator.register_component("comp_a", ComponentPermission::Full).await;
        validator.register_component("comp_b", ComponentPermission::Full).await;

        let message = IpcMessage::new("comp_a", "comp_b", "test")
            .with_string_payload(r#"<img src="x" onerror="alert(1)">"#);

        let result = validator.validate_message(&message).await;
        assert!(matches!(result, Err(IpcValidationError::DangerousContent(_))));
    }

    #[tokio::test]
    async fn test_eval_injection() {
        let validator = IpcValidator::new();
        validator.register_component("comp_a", ComponentPermission::Full).await;
        validator.register_component("comp_b", ComponentPermission::Full).await;

        let message = IpcMessage::new("comp_a", "comp_b", "execute")
            .with_string_payload(r#"eval("malicious code")"#);

        let result = validator.validate_message(&message).await;
        assert!(matches!(result, Err(IpcValidationError::DangerousContent(_))));
    }

    #[tokio::test]
    async fn test_document_write_injection() {
        let validator = IpcValidator::new();
        validator.register_component("comp_a", ComponentPermission::Full).await;
        validator.register_component("comp_b", ComponentPermission::Full).await;

        let message = IpcMessage::new("comp_a", "comp_b", "render")
            .with_string_payload("document.write('<h1>hacked</h1>')");

        let result = validator.validate_message(&message).await;
        assert!(matches!(result, Err(IpcValidationError::DangerousContent(_))));
    }

    #[tokio::test]
    async fn test_get_remaining_tokens() {
        let config = MessageValidationConfig {
            rate_limit_config: RateLimitConfig {
                max_tokens: 10,
                refill_rate: 0.0,
                initial_tokens: 10,
            },
            ..Default::default()
        };

        let validator = IpcValidator::with_config(config);
        validator.register_component("comp_a", ComponentPermission::Full).await;
        validator.register_component("comp_b", ComponentPermission::Full).await;

        // Initial tokens
        let tokens = validator.get_remaining_tokens("comp_a").await;
        assert!(tokens.is_some());
        assert!((tokens.unwrap() - 10.0).abs() < 0.1);

        // After sending a message
        let message = IpcMessage::new("comp_a", "comp_b", "test");
        validator.validate_message(&message).await.unwrap();

        let tokens = validator.get_remaining_tokens("comp_a").await;
        assert!((tokens.unwrap() - 9.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_clear_suspicious_log() {
        let validator = IpcValidator::new();
        validator.register_component("comp_a", ComponentPermission::Full).await;
        validator.register_component("comp_b", ComponentPermission::Full).await;

        // Generate a suspicious log entry
        let message = IpcMessage::new("comp_a", "comp_b", "test")
            .with_string_payload("<script>bad</script>");
        let _ = validator.validate_message(&message).await;

        // Should have one entry
        assert_eq!(validator.get_suspicious_log().await.len(), 1);

        // Clear it
        validator.clear_suspicious_log().await;

        // Should be empty
        assert!(validator.get_suspicious_log().await.is_empty());
    }

    #[tokio::test]
    async fn test_config_update() {
        let validator = IpcValidator::new();

        let initial_config = validator.get_config().await;
        assert_eq!(initial_config.max_message_size, 1024 * 1024);

        let new_config = MessageValidationConfig {
            max_message_size: 2048,
            ..Default::default()
        };
        validator.set_config(new_config).await;

        let updated_config = validator.get_config().await;
        assert_eq!(updated_config.max_message_size, 2048);
    }

    #[test]
    fn test_ipc_message_builder() {
        let message = IpcMessage::new("source", "target", "type")
            .with_string_payload("test payload");

        assert_eq!(message.source, "source");
        assert_eq!(message.target, "target");
        assert_eq!(message.message_type, "type");
        assert_eq!(message.string_payload, Some("test payload".to_string()));
        assert_eq!(message.payload, b"test payload");
    }

    #[test]
    fn test_ipc_message_total_size() {
        let message = IpcMessage::new("src", "tgt", "typ")
            .with_payload(vec![0u8; 100]);

        // 3 (src) + 3 (tgt) + 3 (typ) + 100 (payload) = 109
        assert_eq!(message.total_size(), 109);
    }

    #[test]
    fn test_token_bucket_consume() {
        let config = RateLimitConfig {
            max_tokens: 5,
            refill_rate: 0.0,
            initial_tokens: 3,
        };
        let mut bucket = TokenBucket::new(config);

        assert!(bucket.try_consume());
        assert!(bucket.try_consume());
        assert!(bucket.try_consume());
        assert!(!bucket.try_consume()); // Should fail, no tokens left
    }

    #[test]
    fn test_component_permission_default() {
        assert_eq!(ComponentPermission::default(), ComponentPermission::None);
    }
}
