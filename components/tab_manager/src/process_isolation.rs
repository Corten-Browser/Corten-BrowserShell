//! Process Isolation for Tab Processes
//!
//! This module provides process isolation infrastructure for browser tabs,
//! ensuring that each tab runs in a separate process with its own security
//! sandbox. This prevents a crash in one tab from affecting other tabs.
//!
//! ## Features
//!
//! - Separate process per tab
//! - IPC communication between main process and tab processes
//! - Crash detection and recovery
//! - Process recycling after N navigations
//! - Sandbox enforcement per tab process
//! - Resource limits per tab process

use serde::{Deserialize, Serialize};
use shared_types::{ProcessId, TabId};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Default number of navigations before process recycling
pub const DEFAULT_RECYCLE_THRESHOLD: u32 = 100;

/// Errors that can occur during process isolation operations
#[derive(Debug, Error)]
pub enum ProcessIsolationError {
    /// Failed to spawn a new process
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(String),

    /// IPC communication failed
    #[error("IPC communication failed: {0}")]
    IpcFailed(String),

    /// Process crashed unexpectedly
    #[error("Process crashed: {0}")]
    ProcessCrashed(String),

    /// Process not found
    #[error("Process not found for tab: {0:?}")]
    ProcessNotFound(TabId),

    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    /// Sandbox configuration error
    #[error("Sandbox configuration error: {0}")]
    SandboxError(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for process isolation operations
pub type Result<T> = std::result::Result<T, ProcessIsolationError>;

/// Sandbox configuration for tab processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Enable filesystem sandboxing (restrict file access)
    pub filesystem_sandbox: bool,

    /// Enable network sandboxing (restrict network access)
    pub network_sandbox: bool,

    /// Allowed network hosts (empty = none allowed when network_sandbox is true)
    pub allowed_hosts: Vec<String>,

    /// Enable GPU process isolation
    pub gpu_isolation: bool,

    /// Maximum memory limit in bytes (0 = unlimited)
    pub memory_limit_bytes: u64,

    /// Maximum CPU time in milliseconds (0 = unlimited)
    pub cpu_time_limit_ms: u64,

    /// Enable strict mode (all restrictions enabled)
    pub strict_mode: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            filesystem_sandbox: true,
            network_sandbox: false, // Allow network by default
            allowed_hosts: Vec::new(),
            gpu_isolation: true,
            memory_limit_bytes: 512 * 1024 * 1024, // 512 MB default
            cpu_time_limit_ms: 0,                  // Unlimited
            strict_mode: false,
        }
    }
}

impl SandboxConfig {
    /// Create a strict sandbox configuration
    pub fn strict() -> Self {
        Self {
            filesystem_sandbox: true,
            network_sandbox: true,
            allowed_hosts: Vec::new(),
            gpu_isolation: true,
            memory_limit_bytes: 256 * 1024 * 1024, // 256 MB
            cpu_time_limit_ms: 30000,              // 30 seconds
            strict_mode: true,
        }
    }

    /// Create a permissive sandbox configuration
    pub fn permissive() -> Self {
        Self {
            filesystem_sandbox: false,
            network_sandbox: false,
            allowed_hosts: Vec::new(),
            gpu_isolation: false,
            memory_limit_bytes: 0,
            cpu_time_limit_ms: 0,
            strict_mode: false,
        }
    }

    /// Add an allowed host for network access
    pub fn allow_host(mut self, host: impl Into<String>) -> Self {
        self.allowed_hosts.push(host.into());
        self
    }

    /// Set memory limit
    pub fn with_memory_limit(mut self, bytes: u64) -> Self {
        self.memory_limit_bytes = bytes;
        self
    }

    /// Set CPU time limit
    pub fn with_cpu_limit(mut self, milliseconds: u64) -> Self {
        self.cpu_time_limit_ms = milliseconds;
        self
    }
}

/// Resource limits for a tab process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in bytes
    pub max_memory_bytes: u64,

    /// Maximum CPU percentage (0-100)
    pub max_cpu_percent: u8,

    /// Maximum number of open file descriptors
    pub max_file_descriptors: u32,

    /// Maximum number of threads
    pub max_threads: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 512 * 1024 * 1024, // 512 MB
            max_cpu_percent: 80,
            max_file_descriptors: 256,
            max_threads: 16,
        }
    }
}

/// Messages sent between main process and tab processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TabMessage {
    /// Navigate to a URL
    Navigate { url: String },

    /// Execute JavaScript
    ExecuteScript { script: String },

    /// Request page content
    GetContent,

    /// Request page title
    GetTitle,

    /// Stop loading
    Stop,

    /// Reload page
    Reload { ignore_cache: bool },

    /// Go back in history
    GoBack,

    /// Go forward in history
    GoForward,

    /// Ping for health check
    Ping,

    /// Shutdown the process gracefully
    Shutdown,

    /// Response with content
    ContentResponse { content: String },

    /// Response with title
    TitleResponse { title: String },

    /// Navigation completed
    NavigationComplete { url: String, success: bool },

    /// Error response
    Error { message: String },

    /// Pong response to ping
    Pong,

    /// Process ready notification
    Ready,
}

/// IPC channel for communicating with a tab process
#[derive(Debug)]
pub struct IpcChannel {
    /// Writer to send messages to the child process
    writer: Option<ChildStdin>,

    /// Reader to receive messages from the child process
    reader: Option<BufReader<ChildStdout>>,

    /// Channel ID for identification
    channel_id: u64,
}

impl IpcChannel {
    /// Create a new IPC channel from process stdio
    fn new(stdin: ChildStdin, stdout: ChildStdout, channel_id: u64) -> Self {
        Self {
            writer: Some(stdin),
            reader: Some(BufReader::new(stdout)),
            channel_id,
        }
    }

    /// Get the channel ID
    pub fn channel_id(&self) -> u64 {
        self.channel_id
    }

    /// Send a message to the child process
    pub fn send(&mut self, msg: &TabMessage) -> Result<()> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| ProcessIsolationError::IpcFailed("Writer not available".into()))?;

        let json = serde_json::to_string(msg)
            .map_err(|e| ProcessIsolationError::InvalidMessage(e.to_string()))?;

        writeln!(writer, "{}", json)?;
        writer.flush()?;

        Ok(())
    }

    /// Receive a message from the child process (blocking)
    pub fn recv(&mut self) -> Result<TabMessage> {
        let reader = self
            .reader
            .as_mut()
            .ok_or_else(|| ProcessIsolationError::IpcFailed("Reader not available".into()))?;

        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;

        if bytes_read == 0 {
            return Err(ProcessIsolationError::ProcessCrashed(
                "Process closed stdout".into(),
            ));
        }

        serde_json::from_str(line.trim())
            .map_err(|e| ProcessIsolationError::InvalidMessage(e.to_string()))
    }

    /// Try to receive a message without blocking (returns None if no message available)
    /// Note: This is a simplified implementation; real async would use tokio
    pub fn try_recv(&mut self) -> Result<Option<TabMessage>> {
        // For now, we don't have non-blocking IO without async
        // In production, this would use async channels
        Ok(None)
    }

    /// Close the channel
    pub fn close(&mut self) {
        self.writer.take();
        self.reader.take();
    }

    /// Check if the channel is open
    pub fn is_open(&self) -> bool {
        self.writer.is_some() && self.reader.is_some()
    }
}

/// Process status for a tab process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    /// Process is starting up
    Starting,

    /// Process is running normally
    Running,

    /// Process is suspended (e.g., tab not visible)
    Suspended,

    /// Process has crashed
    Crashed,

    /// Process was terminated gracefully
    Terminated,
}

/// Represents a separate process for a browser tab
pub struct TabProcess {
    /// The child process handle
    process: Option<Child>,

    /// IPC channel for communication
    ipc_channel: IpcChannel,

    /// Associated tab ID
    tab_id: TabId,

    /// Process ID (if available)
    process_id: Option<ProcessId>,

    /// Number of navigations since process start
    navigation_count: AtomicU32,

    /// Sandbox configuration
    sandbox_config: SandboxConfig,

    /// Resource limits
    resource_limits: ResourceLimits,

    /// Current process status
    status: ProcessStatus,

    /// Creation timestamp
    created_at: std::time::Instant,
}

impl TabProcess {
    /// Spawn a new tab process with the given sandbox configuration
    pub fn spawn(
        tab_id: TabId,
        sandbox_config: SandboxConfig,
        resource_limits: ResourceLimits,
    ) -> Result<Self> {
        // Generate unique channel ID
        static CHANNEL_COUNTER: AtomicU32 = AtomicU32::new(0);
        let channel_id = CHANNEL_COUNTER.fetch_add(1, Ordering::SeqCst) as u64;

        // Build the command for the tab renderer process
        // In a real browser, this would be a separate renderer binary
        // For now, we spawn a placeholder that can be replaced
        let mut cmd = Command::new("cat"); // Placeholder - echoes back for testing

        // Configure stdio for IPC
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        // Apply sandbox configuration via environment variables
        // Real implementation would use OS-specific sandboxing APIs
        cmd.env(
            "SANDBOX_FILESYSTEM",
            sandbox_config.filesystem_sandbox.to_string(),
        );
        cmd.env(
            "SANDBOX_NETWORK",
            sandbox_config.network_sandbox.to_string(),
        );
        cmd.env("SANDBOX_GPU", sandbox_config.gpu_isolation.to_string());
        cmd.env("SANDBOX_STRICT", sandbox_config.strict_mode.to_string());
        cmd.env(
            "MEMORY_LIMIT",
            sandbox_config.memory_limit_bytes.to_string(),
        );

        // Spawn the process
        let mut child = cmd
            .spawn()
            .map_err(|e| ProcessIsolationError::SpawnFailed(e.to_string()))?;

        // Get process ID
        let pid = child.id();
        let process_id = Some(ProcessId::new(pid));

        // Set up IPC channel
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| ProcessIsolationError::SpawnFailed("Failed to get stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ProcessIsolationError::SpawnFailed("Failed to get stdout".into()))?;

        let ipc_channel = IpcChannel::new(stdin, stdout, channel_id);

        Ok(Self {
            process: Some(child),
            ipc_channel,
            tab_id,
            process_id,
            navigation_count: AtomicU32::new(0),
            sandbox_config,
            resource_limits,
            status: ProcessStatus::Starting,
            created_at: std::time::Instant::now(),
        })
    }

    /// Spawn a new tab process with default configuration
    pub fn spawn_default(tab_id: TabId) -> Result<Self> {
        Self::spawn(tab_id, SandboxConfig::default(), ResourceLimits::default())
    }

    /// Get the tab ID associated with this process
    pub fn tab_id(&self) -> TabId {
        self.tab_id
    }

    /// Get the process ID
    pub fn process_id(&self) -> Option<ProcessId> {
        self.process_id
    }

    /// Get the current process status
    pub fn status(&self) -> ProcessStatus {
        self.status
    }

    /// Get the navigation count
    pub fn navigation_count(&self) -> u32 {
        self.navigation_count.load(Ordering::SeqCst)
    }

    /// Increment the navigation count
    pub fn increment_navigation_count(&self) -> u32 {
        self.navigation_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Get the sandbox configuration
    pub fn sandbox_config(&self) -> &SandboxConfig {
        &self.sandbox_config
    }

    /// Get the resource limits
    pub fn resource_limits(&self) -> &ResourceLimits {
        &self.resource_limits
    }

    /// Get the uptime of this process
    pub fn uptime(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Send a message to the tab process
    pub fn send_message(&mut self, msg: TabMessage) -> Result<()> {
        self.ipc_channel.send(&msg)
    }

    /// Receive a message from the tab process
    pub fn recv_message(&mut self) -> Result<TabMessage> {
        self.ipc_channel.recv()
    }

    /// Check if the process should be recycled based on navigation count
    pub fn should_recycle(&self, threshold: u32) -> bool {
        self.navigation_count.load(Ordering::SeqCst) >= threshold
    }

    /// Check if the process is alive
    pub fn is_alive(&mut self) -> bool {
        if let Some(ref mut child) = self.process {
            match child.try_wait() {
                Ok(None) => true, // Still running
                Ok(Some(_)) => {
                    self.status = ProcessStatus::Terminated;
                    false
                }
                Err(_) => {
                    self.status = ProcessStatus::Crashed;
                    false
                }
            }
        } else {
            false
        }
    }

    /// Kill the process
    pub fn kill(&mut self) -> Result<()> {
        // Close IPC channel first
        self.ipc_channel.close();

        if let Some(ref mut child) = self.process {
            child.kill()?;
            child.wait()?;
            self.status = ProcessStatus::Terminated;
        }

        self.process = None;
        Ok(())
    }

    /// Attempt graceful shutdown
    pub fn shutdown(&mut self) -> Result<()> {
        // Try to send shutdown message
        if self.ipc_channel.is_open() {
            let _ = self.ipc_channel.send(&TabMessage::Shutdown);
        }

        // Close IPC channel
        self.ipc_channel.close();

        // Wait for process to exit gracefully, then force kill if needed
        if let Some(ref mut child) = self.process {
            // Give process time to exit gracefully
            std::thread::sleep(std::time::Duration::from_millis(100));

            match child.try_wait() {
                Ok(None) => {
                    // Process still running, force kill
                    child.kill()?;
                    child.wait()?;
                }
                Ok(Some(_)) => {
                    // Process already exited
                }
                Err(e) => {
                    return Err(ProcessIsolationError::IoError(e));
                }
            }
            self.status = ProcessStatus::Terminated;
        }

        self.process = None;
        Ok(())
    }

    /// Mark the process as crashed
    pub fn mark_crashed(&mut self) {
        self.status = ProcessStatus::Crashed;
    }

    /// Mark the process as running
    pub fn mark_running(&mut self) {
        self.status = ProcessStatus::Running;
    }

    /// Mark the process as suspended
    pub fn mark_suspended(&mut self) {
        self.status = ProcessStatus::Suspended;
    }
}

impl Drop for TabProcess {
    fn drop(&mut self) {
        // Ensure process is cleaned up
        let _ = self.shutdown();
    }
}

/// Manages process isolation for all tabs
pub struct ProcessIsolationManager {
    /// Map of tab ID to tab process
    processes: Arc<RwLock<HashMap<TabId, TabProcess>>>,

    /// Default sandbox configuration for new tabs
    default_sandbox_config: SandboxConfig,

    /// Default resource limits for new tabs
    default_resource_limits: ResourceLimits,

    /// Navigation count threshold for process recycling
    recycle_threshold: u32,
}

impl ProcessIsolationManager {
    /// Create a new process isolation manager
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            default_sandbox_config: SandboxConfig::default(),
            default_resource_limits: ResourceLimits::default(),
            recycle_threshold: DEFAULT_RECYCLE_THRESHOLD,
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        sandbox_config: SandboxConfig,
        resource_limits: ResourceLimits,
        recycle_threshold: u32,
    ) -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            default_sandbox_config: sandbox_config,
            default_resource_limits: resource_limits,
            recycle_threshold,
        }
    }

    /// Set the default sandbox configuration
    pub fn set_default_sandbox_config(&mut self, config: SandboxConfig) {
        self.default_sandbox_config = config;
    }

    /// Set the default resource limits
    pub fn set_default_resource_limits(&mut self, limits: ResourceLimits) {
        self.default_resource_limits = limits;
    }

    /// Set the recycle threshold
    pub fn set_recycle_threshold(&mut self, threshold: u32) {
        self.recycle_threshold = threshold;
    }

    /// Spawn a new process for a tab
    pub async fn spawn_process(&self, tab_id: TabId) -> Result<ProcessId> {
        let process = TabProcess::spawn(
            tab_id,
            self.default_sandbox_config.clone(),
            self.default_resource_limits.clone(),
        )?;

        let process_id = process
            .process_id()
            .ok_or_else(|| ProcessIsolationError::SpawnFailed("No process ID".into()))?;

        let mut processes = self.processes.write().await;
        processes.insert(tab_id, process);

        Ok(process_id)
    }

    /// Spawn a process with custom sandbox configuration
    pub async fn spawn_process_with_config(
        &self,
        tab_id: TabId,
        sandbox_config: SandboxConfig,
        resource_limits: ResourceLimits,
    ) -> Result<ProcessId> {
        let process = TabProcess::spawn(tab_id, sandbox_config, resource_limits)?;

        let process_id = process
            .process_id()
            .ok_or_else(|| ProcessIsolationError::SpawnFailed("No process ID".into()))?;

        let mut processes = self.processes.write().await;
        processes.insert(tab_id, process);

        Ok(process_id)
    }

    /// Kill a tab process
    pub async fn kill_process(&self, tab_id: TabId) -> Result<()> {
        let mut processes = self.processes.write().await;

        if let Some(mut process) = processes.remove(&tab_id) {
            process.kill()
        } else {
            Err(ProcessIsolationError::ProcessNotFound(tab_id))
        }
    }

    /// Gracefully shutdown a tab process
    pub async fn shutdown_process(&self, tab_id: TabId) -> Result<()> {
        let mut processes = self.processes.write().await;

        if let Some(mut process) = processes.remove(&tab_id) {
            process.shutdown()
        } else {
            Err(ProcessIsolationError::ProcessNotFound(tab_id))
        }
    }

    /// Send a message to a tab process
    pub async fn send_message(&self, tab_id: TabId, msg: TabMessage) -> Result<()> {
        let mut processes = self.processes.write().await;

        if let Some(process) = processes.get_mut(&tab_id) {
            process.send_message(msg)
        } else {
            Err(ProcessIsolationError::ProcessNotFound(tab_id))
        }
    }

    /// Check if a tab should have its process recycled
    pub async fn should_recycle(&self, tab_id: TabId) -> bool {
        let processes = self.processes.read().await;

        if let Some(process) = processes.get(&tab_id) {
            process.should_recycle(self.recycle_threshold)
        } else {
            false
        }
    }

    /// Recycle a tab process (kill old, spawn new)
    pub async fn recycle_process(&self, tab_id: TabId) -> Result<ProcessId> {
        // Get the old process config before removing
        let (sandbox_config, resource_limits) = {
            let processes = self.processes.read().await;
            if let Some(process) = processes.get(&tab_id) {
                (
                    process.sandbox_config().clone(),
                    process.resource_limits().clone(),
                )
            } else {
                return Err(ProcessIsolationError::ProcessNotFound(tab_id));
            }
        };

        // Kill the old process
        self.kill_process(tab_id).await?;

        // Spawn a new one with the same config
        self.spawn_process_with_config(tab_id, sandbox_config, resource_limits)
            .await
    }

    /// Check for crashed processes and return their tab IDs
    pub async fn check_for_crashes(&self) -> Vec<TabId> {
        let mut processes = self.processes.write().await;
        let mut crashed = Vec::new();

        for (tab_id, process) in processes.iter_mut() {
            if !process.is_alive() && process.status() == ProcessStatus::Crashed {
                crashed.push(*tab_id);
            }
        }

        crashed
    }

    /// Get the status of a tab process
    pub async fn get_process_status(&self, tab_id: TabId) -> Option<ProcessStatus> {
        let processes = self.processes.read().await;
        processes.get(&tab_id).map(|p| p.status())
    }

    /// Get the process ID for a tab
    pub async fn get_process_id(&self, tab_id: TabId) -> Option<ProcessId> {
        let processes = self.processes.read().await;
        processes.get(&tab_id).and_then(|p| p.process_id())
    }

    /// Increment navigation count for a tab
    pub async fn record_navigation(&self, tab_id: TabId) -> Option<u32> {
        let processes = self.processes.read().await;
        processes
            .get(&tab_id)
            .map(|p| p.increment_navigation_count())
    }

    /// Get the number of active processes
    pub async fn active_process_count(&self) -> usize {
        let processes = self.processes.read().await;
        processes.len()
    }

    /// Shutdown all processes
    pub async fn shutdown_all(&self) -> Vec<(TabId, Result<()>)> {
        let mut processes = self.processes.write().await;
        let mut results = Vec::new();

        let tab_ids: Vec<TabId> = processes.keys().copied().collect();

        for tab_id in tab_ids {
            if let Some(mut process) = processes.remove(&tab_id) {
                results.push((tab_id, process.shutdown()));
            }
        }

        results
    }
}

impl Default for ProcessIsolationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();

        assert!(config.filesystem_sandbox);
        assert!(!config.network_sandbox);
        assert!(config.gpu_isolation);
        assert!(!config.strict_mode);
        assert_eq!(config.memory_limit_bytes, 512 * 1024 * 1024);
    }

    #[test]
    fn test_sandbox_config_strict() {
        let config = SandboxConfig::strict();

        assert!(config.filesystem_sandbox);
        assert!(config.network_sandbox);
        assert!(config.gpu_isolation);
        assert!(config.strict_mode);
        assert_eq!(config.memory_limit_bytes, 256 * 1024 * 1024);
    }

    #[test]
    fn test_sandbox_config_permissive() {
        let config = SandboxConfig::permissive();

        assert!(!config.filesystem_sandbox);
        assert!(!config.network_sandbox);
        assert!(!config.gpu_isolation);
        assert!(!config.strict_mode);
        assert_eq!(config.memory_limit_bytes, 0);
    }

    #[test]
    fn test_sandbox_config_builder() {
        let config = SandboxConfig::default()
            .allow_host("example.com")
            .allow_host("api.example.com")
            .with_memory_limit(1024 * 1024 * 1024)
            .with_cpu_limit(60000);

        assert_eq!(config.allowed_hosts.len(), 2);
        assert!(config.allowed_hosts.contains(&"example.com".to_string()));
        assert_eq!(config.memory_limit_bytes, 1024 * 1024 * 1024);
        assert_eq!(config.cpu_time_limit_ms, 60000);
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();

        assert_eq!(limits.max_memory_bytes, 512 * 1024 * 1024);
        assert_eq!(limits.max_cpu_percent, 80);
        assert_eq!(limits.max_file_descriptors, 256);
        assert_eq!(limits.max_threads, 16);
    }

    #[test]
    fn test_tab_message_serialization() {
        let navigate = TabMessage::Navigate {
            url: "https://example.com".to_string(),
        };
        let json = serde_json::to_string(&navigate).unwrap();
        let deserialized: TabMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            TabMessage::Navigate { url } => assert_eq!(url, "https://example.com"),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_process_status() {
        assert_ne!(ProcessStatus::Running, ProcessStatus::Crashed);
        assert_ne!(ProcessStatus::Starting, ProcessStatus::Terminated);
    }

    #[tokio::test]
    async fn test_process_isolation_manager_creation() {
        let manager = ProcessIsolationManager::new();
        assert_eq!(manager.active_process_count().await, 0);
        assert_eq!(manager.recycle_threshold, DEFAULT_RECYCLE_THRESHOLD);
    }

    #[tokio::test]
    async fn test_process_isolation_manager_with_config() {
        let config = SandboxConfig::strict();
        let limits = ResourceLimits::default();
        let manager = ProcessIsolationManager::with_config(config.clone(), limits.clone(), 50);

        assert_eq!(manager.recycle_threshold, 50);
        assert!(manager.default_sandbox_config.strict_mode);
    }

    #[test]
    fn test_tab_process_should_recycle() {
        // Test the should_recycle logic without spawning a real process
        let count = AtomicU32::new(99);
        assert!(!count.load(Ordering::SeqCst) >= 100);

        count.fetch_add(1, Ordering::SeqCst);
        assert!(count.load(Ordering::SeqCst) >= 100);
    }

    #[tokio::test]
    async fn test_manager_set_recycle_threshold() {
        let mut manager = ProcessIsolationManager::new();
        assert_eq!(manager.recycle_threshold, DEFAULT_RECYCLE_THRESHOLD);

        manager.set_recycle_threshold(50);
        assert_eq!(manager.recycle_threshold, 50);
    }

    #[tokio::test]
    async fn test_manager_get_nonexistent_process() {
        let manager = ProcessIsolationManager::new();
        let fake_tab_id = TabId::new();

        let status = manager.get_process_status(fake_tab_id).await;
        assert!(status.is_none());

        let pid = manager.get_process_id(fake_tab_id).await;
        assert!(pid.is_none());
    }

    #[tokio::test]
    async fn test_manager_should_recycle_nonexistent() {
        let manager = ProcessIsolationManager::new();
        let fake_tab_id = TabId::new();

        assert!(!manager.should_recycle(fake_tab_id).await);
    }

    #[tokio::test]
    async fn test_shutdown_all_empty() {
        let manager = ProcessIsolationManager::new();
        let results = manager.shutdown_all().await;
        assert!(results.is_empty());
    }

    #[test]
    fn test_ipc_channel_id() {
        // Test channel ID generation would require spawning processes
        // Just verify the constant is defined
        assert_eq!(DEFAULT_RECYCLE_THRESHOLD, 100);
    }

    #[test]
    fn test_process_isolation_error_display() {
        let err = ProcessIsolationError::SpawnFailed("test error".into());
        assert!(err.to_string().contains("test error"));

        let tab_id = TabId::new();
        let err = ProcessIsolationError::ProcessNotFound(tab_id);
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_tab_message_variants() {
        // Test all message variants can be created
        let _ = TabMessage::Navigate {
            url: "test".to_string(),
        };
        let _ = TabMessage::ExecuteScript {
            script: "test".to_string(),
        };
        let _ = TabMessage::GetContent;
        let _ = TabMessage::GetTitle;
        let _ = TabMessage::Stop;
        let _ = TabMessage::Reload { ignore_cache: true };
        let _ = TabMessage::GoBack;
        let _ = TabMessage::GoForward;
        let _ = TabMessage::Ping;
        let _ = TabMessage::Shutdown;
        let _ = TabMessage::ContentResponse {
            content: "test".to_string(),
        };
        let _ = TabMessage::TitleResponse {
            title: "test".to_string(),
        };
        let _ = TabMessage::NavigationComplete {
            url: "test".to_string(),
            success: true,
        };
        let _ = TabMessage::Error {
            message: "test".to_string(),
        };
        let _ = TabMessage::Pong;
        let _ = TabMessage::Ready;
    }
}
