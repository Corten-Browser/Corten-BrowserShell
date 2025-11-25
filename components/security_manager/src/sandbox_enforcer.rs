//! Sandbox Enforcement Hooks
//!
//! Provides hooks and utilities for enforcing sandbox policies on processes,
//! ensuring that processes operate within their security boundaries.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Errors that can occur during sandbox enforcement
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxError {
    /// Process attempted to access prohibited resource
    #[error("Access denied to resource: {resource} by process {process_id}")]
    AccessDenied {
        process_id: String,
        resource: String,
    },

    /// Process attempted prohibited operation
    #[error("Operation {operation} not allowed for process {process_id}")]
    OperationNotAllowed {
        process_id: String,
        operation: String,
    },

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {limit_type} for process {process_id}")]
    ResourceLimitExceeded {
        process_id: String,
        limit_type: String,
    },

    /// Sandbox policy violation
    #[error("Sandbox policy violation: {violation} by process {process_id}")]
    PolicyViolation {
        process_id: String,
        violation: String,
    },
}

/// Resource types that can be restricted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// File system access
    FileSystem,
    /// Network access
    Network,
    /// Process spawning
    ProcessSpawn,
    /// Inter-process communication
    IPC,
    /// System calls
    SystemCall,
    /// GPU access
    GPU,
    /// Memory allocation
    Memory,
}

/// Access level for resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    /// No access allowed
    None,
    /// Read-only access
    ReadOnly,
    /// Write-only access
    WriteOnly,
    /// Full read-write access
    Full,
}

/// Sandbox policy for a process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    /// Process identifier
    pub process_id: String,
    /// Resource access permissions
    pub resource_permissions: HashMap<ResourceType, AccessLevel>,
    /// Allowed file paths (if filesystem access is granted)
    pub allowed_paths: Vec<String>,
    /// Allowed network hosts (if network access is granted)
    pub allowed_hosts: Vec<String>,
    /// Allowed system calls (if system call access is granted)
    pub allowed_syscalls: Vec<String>,
    /// Strict mode (all restrictions enforced, no fallback)
    pub strict_mode: bool,
    /// Custom policy rules (key-value pairs)
    pub custom_rules: HashMap<String, String>,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        let mut resource_permissions = HashMap::new();
        resource_permissions.insert(ResourceType::FileSystem, AccessLevel::ReadOnly);
        resource_permissions.insert(ResourceType::Network, AccessLevel::Full);
        resource_permissions.insert(ResourceType::IPC, AccessLevel::Full);
        resource_permissions.insert(ResourceType::ProcessSpawn, AccessLevel::None);
        resource_permissions.insert(ResourceType::SystemCall, AccessLevel::None);
        resource_permissions.insert(ResourceType::GPU, AccessLevel::ReadOnly);
        resource_permissions.insert(ResourceType::Memory, AccessLevel::Full);

        Self {
            process_id: String::new(),
            resource_permissions,
            allowed_paths: vec!["/tmp".to_string()],
            allowed_hosts: Vec::new(),
            allowed_syscalls: Vec::new(),
            strict_mode: false,
            custom_rules: HashMap::new(),
        }
    }
}

impl SandboxPolicy {
    /// Create a strict sandbox policy
    pub fn strict(process_id: impl Into<String>) -> Self {
        let mut resource_permissions = HashMap::new();
        resource_permissions.insert(ResourceType::FileSystem, AccessLevel::None);
        resource_permissions.insert(ResourceType::Network, AccessLevel::None);
        resource_permissions.insert(ResourceType::IPC, AccessLevel::ReadOnly);
        resource_permissions.insert(ResourceType::ProcessSpawn, AccessLevel::None);
        resource_permissions.insert(ResourceType::SystemCall, AccessLevel::None);
        resource_permissions.insert(ResourceType::GPU, AccessLevel::None);
        resource_permissions.insert(ResourceType::Memory, AccessLevel::ReadOnly);

        Self {
            process_id: process_id.into(),
            resource_permissions,
            allowed_paths: Vec::new(),
            allowed_hosts: Vec::new(),
            allowed_syscalls: Vec::new(),
            strict_mode: true,
            custom_rules: HashMap::new(),
        }
    }

    /// Create a permissive sandbox policy
    pub fn permissive(process_id: impl Into<String>) -> Self {
        Self {
            process_id: process_id.into(),
            strict_mode: false,
            ..Default::default()
        }
    }

    /// Set resource access level
    pub fn set_resource_access(
        mut self,
        resource: ResourceType,
        access: AccessLevel,
    ) -> Self {
        self.resource_permissions.insert(resource, access);
        self
    }

    /// Add allowed file path
    pub fn allow_path(mut self, path: impl Into<String>) -> Self {
        self.allowed_paths.push(path.into());
        self
    }

    /// Add allowed network host
    pub fn allow_host(mut self, host: impl Into<String>) -> Self {
        self.allowed_hosts.push(host.into());
        self
    }

    /// Add allowed system call
    pub fn allow_syscall(mut self, syscall: impl Into<String>) -> Self {
        self.allowed_syscalls.push(syscall.into());
        self
    }

    /// Add custom rule
    pub fn add_rule(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_rules.insert(key.into(), value.into());
        self
    }
}

/// Sandbox enforcement event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxEvent {
    /// Timestamp (seconds since UNIX epoch)
    pub timestamp_secs: u64,
    /// Process identifier
    pub process_id: String,
    /// Resource type accessed
    pub resource_type: ResourceType,
    /// Resource identifier (path, host, etc.)
    pub resource_id: String,
    /// Operation attempted
    pub operation: String,
    /// Whether access was allowed
    pub allowed: bool,
    /// Reason for decision
    pub reason: String,
}

/// Sandbox enforcer that validates process operations against policies
pub struct SandboxEnforcer {
    /// Process policies: process_id -> policy
    policies: Arc<RwLock<HashMap<String, SandboxPolicy>>>,
    /// Event log
    events: Arc<RwLock<Vec<SandboxEvent>>>,
    /// Whether to log all events (not just violations)
    log_all_events: bool,
}

impl SandboxEnforcer {
    /// Create a new sandbox enforcer
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            log_all_events: false,
        }
    }

    /// Create enforcer with all events logged
    pub fn with_full_logging() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            log_all_events: true,
        }
    }

    /// Set whether to log all events
    pub fn set_log_all_events(&mut self, log_all: bool) {
        self.log_all_events = log_all;
    }

    /// Register a sandbox policy for a process
    pub async fn register_policy(&self, policy: SandboxPolicy) {
        let mut policies = self.policies.write().await;
        policies.insert(policy.process_id.clone(), policy);
    }

    /// Unregister a process policy
    pub async fn unregister_policy(&self, process_id: &str) {
        let mut policies = self.policies.write().await;
        policies.remove(process_id);
    }

    /// Check if a process can access a resource
    pub async fn check_access(
        &self,
        process_id: &str,
        resource_type: ResourceType,
        resource_id: &str,
        operation: &str,
    ) -> Result<(), SandboxError> {
        let policies = self.policies.read().await;

        let policy = policies.get(process_id).ok_or_else(|| {
            SandboxError::PolicyViolation {
                process_id: process_id.to_string(),
                violation: "No policy registered for process".to_string(),
            }
        })?;

        // Check resource access level
        let access_level = policy
            .resource_permissions
            .get(&resource_type)
            .copied()
            .unwrap_or(AccessLevel::None);

        let allowed = match access_level {
            AccessLevel::None => false,
            AccessLevel::ReadOnly => operation == "read" || operation == "list",
            AccessLevel::WriteOnly => operation == "write" || operation == "create" || operation == "delete",
            AccessLevel::Full => true,
        };

        if !allowed {
            let error = SandboxError::OperationNotAllowed {
                process_id: process_id.to_string(),
                operation: operation.to_string(),
            };

            self.log_event(
                process_id,
                resource_type,
                resource_id,
                operation,
                false,
                &error.to_string(),
            )
            .await;

            return Err(error);
        }

        // Additional checks based on resource type
        match resource_type {
            ResourceType::FileSystem => {
                if !policy.allowed_paths.is_empty() {
                    let allowed = policy.allowed_paths.iter().any(|path| {
                        resource_id.starts_with(path) || resource_id == path
                    });

                    if !allowed {
                        let error = SandboxError::AccessDenied {
                            process_id: process_id.to_string(),
                            resource: resource_id.to_string(),
                        };

                        self.log_event(
                            process_id,
                            resource_type,
                            resource_id,
                            operation,
                            false,
                            &error.to_string(),
                        )
                        .await;

                        return Err(error);
                    }
                }
            }
            ResourceType::Network => {
                if !policy.allowed_hosts.is_empty() {
                    let allowed = policy.allowed_hosts.iter().any(|host| {
                        resource_id.contains(host) || resource_id == host
                    });

                    if !allowed {
                        let error = SandboxError::AccessDenied {
                            process_id: process_id.to_string(),
                            resource: resource_id.to_string(),
                        };

                        self.log_event(
                            process_id,
                            resource_type,
                            resource_id,
                            operation,
                            false,
                            &error.to_string(),
                        )
                        .await;

                        return Err(error);
                    }
                }
            }
            ResourceType::SystemCall => {
                if !policy.allowed_syscalls.is_empty() {
                    let allowed = policy.allowed_syscalls.iter().any(|syscall| {
                        resource_id == syscall
                    });

                    if !allowed {
                        let error = SandboxError::AccessDenied {
                            process_id: process_id.to_string(),
                            resource: resource_id.to_string(),
                        };

                        self.log_event(
                            process_id,
                            resource_type,
                            resource_id,
                            operation,
                            false,
                            &error.to_string(),
                        )
                        .await;

                        return Err(error);
                    }
                }
            }
            _ => {}
        }

        // Log successful access if logging all events
        if self.log_all_events {
            self.log_event(
                process_id,
                resource_type,
                resource_id,
                operation,
                true,
                "Access granted",
            )
            .await;
        }

        Ok(())
    }

    /// Log a sandbox event
    async fn log_event(
        &self,
        process_id: &str,
        resource_type: ResourceType,
        resource_id: &str,
        operation: &str,
        allowed: bool,
        reason: &str,
    ) {
        let event = SandboxEvent {
            timestamp_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            process_id: process_id.to_string(),
            resource_type,
            resource_id: resource_id.to_string(),
            operation: operation.to_string(),
            allowed,
            reason: reason.to_string(),
        };

        self.events.write().await.push(event);
    }

    /// Get all sandbox events
    pub async fn get_events(&self) -> Vec<SandboxEvent> {
        self.events.read().await.clone()
    }

    /// Get events for a specific process
    pub async fn get_process_events(&self, process_id: &str) -> Vec<SandboxEvent> {
        self.events
            .read()
            .await
            .iter()
            .filter(|e| e.process_id == process_id)
            .cloned()
            .collect()
    }

    /// Get violation events only
    pub async fn get_violations(&self) -> Vec<SandboxEvent> {
        self.events
            .read()
            .await
            .iter()
            .filter(|e| !e.allowed)
            .cloned()
            .collect()
    }

    /// Clear event log
    pub async fn clear_events(&self) {
        self.events.write().await.clear();
    }

    /// Get policy for a process
    pub async fn get_policy(&self, process_id: &str) -> Option<SandboxPolicy> {
        self.policies.read().await.get(process_id).cloned()
    }
}

impl Default for SandboxEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sandbox_policy_default() {
        let policy = SandboxPolicy::default();

        assert_eq!(
            policy.resource_permissions.get(&ResourceType::FileSystem),
            Some(&AccessLevel::ReadOnly)
        );
        assert_eq!(
            policy.resource_permissions.get(&ResourceType::Network),
            Some(&AccessLevel::Full)
        );
    }

    #[tokio::test]
    async fn test_sandbox_policy_strict() {
        let policy = SandboxPolicy::strict("test_process");

        assert_eq!(policy.process_id, "test_process");
        assert!(policy.strict_mode);
        assert_eq!(
            policy.resource_permissions.get(&ResourceType::FileSystem),
            Some(&AccessLevel::None)
        );
    }

    #[tokio::test]
    async fn test_sandbox_policy_builder() {
        let policy = SandboxPolicy::default()
            .set_resource_access(ResourceType::FileSystem, AccessLevel::Full)
            .allow_path("/home/user")
            .allow_host("example.com")
            .allow_syscall("open");

        assert_eq!(
            policy.resource_permissions.get(&ResourceType::FileSystem),
            Some(&AccessLevel::Full)
        );
        assert!(policy.allowed_paths.contains(&"/home/user".to_string()));
        assert!(policy.allowed_hosts.contains(&"example.com".to_string()));
        assert!(policy.allowed_syscalls.contains(&"open".to_string()));
    }

    #[tokio::test]
    async fn test_enforcer_register_policy() {
        let enforcer = SandboxEnforcer::new();
        let policy = SandboxPolicy::strict("proc1");

        enforcer.register_policy(policy).await;

        let retrieved = enforcer.get_policy("proc1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().process_id, "proc1");
    }

    #[tokio::test]
    async fn test_check_access_no_policy() {
        let enforcer = SandboxEnforcer::new();

        let result = enforcer
            .check_access("unknown", ResourceType::FileSystem, "/test", "read")
            .await;

        assert!(matches!(result, Err(SandboxError::PolicyViolation { .. })));
    }

    #[tokio::test]
    async fn test_check_access_allowed() {
        let enforcer = SandboxEnforcer::new();
        let mut policy = SandboxPolicy::default()
            .set_resource_access(ResourceType::FileSystem, AccessLevel::ReadOnly)
            .allow_path("/test");

        policy.process_id = "proc1".to_string();

        enforcer.register_policy(policy).await;

        let result = enforcer
            .check_access("proc1", ResourceType::FileSystem, "/test/file.txt", "read")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_access_denied_operation() {
        let enforcer = SandboxEnforcer::new();
        let mut policy = SandboxPolicy::default();
        policy.process_id = "proc1".to_string();
        policy
            .resource_permissions
            .insert(ResourceType::FileSystem, AccessLevel::ReadOnly);

        enforcer.register_policy(policy).await;

        let result = enforcer
            .check_access("proc1", ResourceType::FileSystem, "/test/file.txt", "write")
            .await;

        assert!(matches!(result, Err(SandboxError::OperationNotAllowed { .. })));
    }

    #[tokio::test]
    async fn test_check_access_denied_path() {
        let enforcer = SandboxEnforcer::new();
        let mut policy = SandboxPolicy::default();
        policy.process_id = "proc1".to_string();
        policy
            .resource_permissions
            .insert(ResourceType::FileSystem, AccessLevel::ReadOnly);
        policy.allowed_paths = vec!["/allowed".to_string()];

        enforcer.register_policy(policy).await;

        let result = enforcer
            .check_access("proc1", ResourceType::FileSystem, "/forbidden/file.txt", "read")
            .await;

        assert!(matches!(result, Err(SandboxError::AccessDenied { .. })));
    }

    #[tokio::test]
    async fn test_check_access_network_allowed() {
        let enforcer = SandboxEnforcer::new();
        let mut policy = SandboxPolicy::default();
        policy.process_id = "proc1".to_string();
        policy
            .resource_permissions
            .insert(ResourceType::Network, AccessLevel::Full);
        policy.allowed_hosts = vec!["example.com".to_string()];

        enforcer.register_policy(policy).await;

        let result = enforcer
            .check_access("proc1", ResourceType::Network, "example.com", "connect")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_access_network_denied() {
        let enforcer = SandboxEnforcer::new();
        let mut policy = SandboxPolicy::default();
        policy.process_id = "proc1".to_string();
        policy
            .resource_permissions
            .insert(ResourceType::Network, AccessLevel::Full);
        policy.allowed_hosts = vec!["example.com".to_string()];

        enforcer.register_policy(policy).await;

        let result = enforcer
            .check_access("proc1", ResourceType::Network, "malicious.com", "connect")
            .await;

        assert!(matches!(result, Err(SandboxError::AccessDenied { .. })));
    }

    #[tokio::test]
    async fn test_event_logging() {
        let enforcer = SandboxEnforcer::with_full_logging();
        let mut policy = SandboxPolicy::default();
        policy.process_id = "proc1".to_string();
        policy
            .resource_permissions
            .insert(ResourceType::FileSystem, AccessLevel::ReadOnly);
        policy.allowed_paths = vec!["/allowed".to_string()];

        enforcer.register_policy(policy).await;

        // Allowed access
        let _ = enforcer
            .check_access("proc1", ResourceType::FileSystem, "/allowed/file.txt", "read")
            .await;

        // Denied access
        let _ = enforcer
            .check_access("proc1", ResourceType::FileSystem, "/forbidden/file.txt", "read")
            .await;

        let events = enforcer.get_events().await;
        assert_eq!(events.len(), 2);
        assert!(events[0].allowed);
        assert!(!events[1].allowed);
    }

    #[tokio::test]
    async fn test_get_violations() {
        let enforcer = SandboxEnforcer::new();
        let mut policy = SandboxPolicy::default();
        policy.process_id = "proc1".to_string();
        policy
            .resource_permissions
            .insert(ResourceType::FileSystem, AccessLevel::ReadOnly);

        enforcer.register_policy(policy).await;

        // Trigger violation
        let _ = enforcer
            .check_access("proc1", ResourceType::FileSystem, "/test", "write")
            .await;

        let violations = enforcer.get_violations().await;
        assert_eq!(violations.len(), 1);
        assert!(!violations[0].allowed);
    }

    #[tokio::test]
    async fn test_unregister_policy() {
        let enforcer = SandboxEnforcer::new();
        let mut policy = SandboxPolicy::default();
        policy.process_id = "proc1".to_string();

        enforcer.register_policy(policy).await;
        assert!(enforcer.get_policy("proc1").await.is_some());

        enforcer.unregister_policy("proc1").await;
        assert!(enforcer.get_policy("proc1").await.is_none());
    }

    #[tokio::test]
    async fn test_clear_events() {
        let enforcer = SandboxEnforcer::with_full_logging();
        let mut policy = SandboxPolicy::default();
        policy.process_id = "proc1".to_string();

        enforcer.register_policy(policy).await;

        let _ = enforcer
            .check_access("proc1", ResourceType::FileSystem, "/test", "read")
            .await;

        assert!(!enforcer.get_events().await.is_empty());

        enforcer.clear_events().await;
        assert!(enforcer.get_events().await.is_empty());
    }
}
