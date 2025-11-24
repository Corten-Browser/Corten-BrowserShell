//! Memory audit utilities for the CortenBrowser Browser Shell
//!
//! This module provides memory tracking and audit capabilities for monitoring
//! memory usage across browser shell components.
//!
//! # Features
//!
//! - **MemoryAudit**: Main struct for collecting memory statistics
//! - **AllocationStats**: Detailed allocation tracking
//! - **Heap usage estimation**: Platform-independent heap monitoring
//!
//! # Example
//!
//! ```rust,ignore
//! use shared_types::memory_audit::{MemoryAudit, AllocationStats};
//!
//! let audit = MemoryAudit::capture();
//! println!("Heap usage: {} bytes", audit.heap_bytes);
//! println!("Allocation count: {}", audit.allocation_count);
//! ```

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Memory audit snapshot containing current memory statistics
///
/// This struct captures a point-in-time view of memory usage,
/// useful for tracking memory consumption and detecting leaks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryAudit {
    /// Estimated heap memory usage in bytes
    pub heap_bytes: u64,
    /// Number of active allocations (if tracked)
    pub allocation_count: u64,
    /// Peak memory usage since last reset (in bytes)
    pub peak_bytes: u64,
    /// Timestamp when this audit was captured (as duration since process start)
    #[serde(with = "duration_serde")]
    pub timestamp: Duration,
    /// Component identifier (if applicable)
    pub component_id: Option<String>,
}

/// Serialization support for Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

impl MemoryAudit {
    /// Create a new memory audit with specified values
    pub fn new(heap_bytes: u64, allocation_count: u64) -> Self {
        Self {
            heap_bytes,
            allocation_count,
            peak_bytes: heap_bytes,
            timestamp: Duration::ZERO,
            component_id: None,
        }
    }

    /// Capture current memory statistics
    ///
    /// This function attempts to gather memory usage information from the
    /// current process. The accuracy depends on platform support.
    ///
    /// # Returns
    ///
    /// A `MemoryAudit` snapshot of current memory usage.
    pub fn capture() -> Self {
        let heap_bytes = get_heap_usage();
        let stats = get_allocation_stats();

        Self {
            heap_bytes,
            allocation_count: stats.allocation_count,
            peak_bytes: stats.peak_bytes,
            timestamp: get_process_uptime(),
            component_id: None,
        }
    }

    /// Capture memory audit for a specific component
    pub fn capture_for_component(component_id: impl Into<String>) -> Self {
        let mut audit = Self::capture();
        audit.component_id = Some(component_id.into());
        audit
    }

    /// Calculate the difference between two memory audits
    ///
    /// Returns the change in memory usage (positive = increase, negative = decrease)
    pub fn diff(&self, other: &MemoryAudit) -> MemoryDiff {
        MemoryDiff {
            heap_bytes_delta: self.heap_bytes as i64 - other.heap_bytes as i64,
            allocation_count_delta: self.allocation_count as i64 - other.allocation_count as i64,
            time_delta: self.timestamp.saturating_sub(other.timestamp),
        }
    }

    /// Check if memory usage exceeds a threshold
    pub fn exceeds_threshold(&self, threshold_bytes: u64) -> bool {
        self.heap_bytes > threshold_bytes
    }

    /// Get memory usage as a human-readable string
    pub fn format_heap_usage(&self) -> String {
        format_bytes(self.heap_bytes)
    }
}

impl Default for MemoryAudit {
    fn default() -> Self {
        Self::capture()
    }
}

/// Difference between two memory audit snapshots
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryDiff {
    /// Change in heap bytes (positive = increase)
    pub heap_bytes_delta: i64,
    /// Change in allocation count (positive = increase)
    pub allocation_count_delta: i64,
    /// Time elapsed between snapshots
    #[serde(with = "duration_serde")]
    pub time_delta: Duration,
}

impl MemoryDiff {
    /// Check if memory increased
    pub fn is_growth(&self) -> bool {
        self.heap_bytes_delta > 0
    }

    /// Get the rate of memory change in bytes per second
    pub fn bytes_per_second(&self) -> f64 {
        let secs = self.time_delta.as_secs_f64();
        if secs > 0.0 {
            self.heap_bytes_delta as f64 / secs
        } else {
            0.0
        }
    }
}

/// Detailed allocation statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllocationStats {
    /// Total number of allocations since tracking began
    pub allocation_count: u64,
    /// Total number of deallocations since tracking began
    pub deallocation_count: u64,
    /// Peak memory usage in bytes
    pub peak_bytes: u64,
    /// Total bytes currently allocated
    pub current_bytes: u64,
    /// Largest single allocation in bytes
    pub largest_allocation: u64,
}

impl AllocationStats {
    /// Create new empty allocation stats
    pub fn new() -> Self {
        Self {
            allocation_count: 0,
            deallocation_count: 0,
            peak_bytes: 0,
            current_bytes: 0,
            largest_allocation: 0,
        }
    }

    /// Check if there are potential memory leaks
    ///
    /// Returns true if allocations significantly exceed deallocations
    pub fn has_potential_leak(&self) -> bool {
        // Simple heuristic: if allocations exceed deallocations by more than 10%
        // and we have a significant number of allocations
        if self.allocation_count < 100 {
            return false;
        }
        let ratio = self.deallocation_count as f64 / self.allocation_count as f64;
        ratio < 0.9
    }

    /// Get the net allocation count (allocations - deallocations)
    pub fn net_allocations(&self) -> i64 {
        self.allocation_count as i64 - self.deallocation_count as i64
    }
}

impl Default for AllocationStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Get estimated heap usage in bytes
///
/// This function attempts to estimate the current heap memory usage.
/// The implementation is platform-dependent and may not be perfectly accurate.
///
/// # Returns
///
/// Estimated heap usage in bytes, or 0 if unavailable.
pub fn get_heap_usage() -> u64 {
    // Platform-independent estimation using Rust's allocator info
    // In a production system, this would use platform-specific APIs
    // For now, we use a simple heuristic based on process info

    #[cfg(target_os = "linux")]
    {
        // On Linux, try to read from /proc/self/statm
        if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
            let parts: Vec<&str> = statm.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(resident_pages) = parts[1].parse::<u64>() {
                    // Pages are typically 4KB
                    return resident_pages * 4096;
                }
            }
        }
    }

    // Fallback: estimate based on system allocator
    // This is a placeholder that returns 0 when we can't measure
    0
}

/// Get detailed allocation statistics
///
/// Returns current allocation tracking statistics. Note that detailed
/// tracking may not be available on all platforms or configurations.
///
/// # Returns
///
/// `AllocationStats` with available allocation information.
pub fn get_allocation_stats() -> AllocationStats {
    // In a real implementation, this would integrate with a custom allocator
    // or use platform-specific APIs to track allocations

    let heap = get_heap_usage();

    AllocationStats {
        allocation_count: 0, // Would require custom allocator to track
        deallocation_count: 0,
        peak_bytes: heap, // Best estimate without tracking
        current_bytes: heap,
        largest_allocation: 0,
    }
}

/// Get process uptime as Duration
fn get_process_uptime() -> Duration {
    // Use a static start time to calculate uptime
    use std::sync::OnceLock;
    static START: OnceLock<Instant> = OnceLock::new();
    let start = START.get_or_init(Instant::now);
    start.elapsed()
}

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Memory tracker for monitoring memory changes over time
///
/// This struct provides a simple way to track memory usage changes
/// by taking periodic snapshots.
#[derive(Debug)]
pub struct MemoryTracker {
    /// Initial baseline snapshot
    baseline: MemoryAudit,
    /// History of snapshots
    snapshots: Vec<MemoryAudit>,
    /// Maximum number of snapshots to keep
    max_snapshots: usize,
}

impl MemoryTracker {
    /// Create a new memory tracker with current state as baseline
    pub fn new() -> Self {
        Self {
            baseline: MemoryAudit::capture(),
            snapshots: Vec::new(),
            max_snapshots: 100,
        }
    }

    /// Create a tracker with custom snapshot limit
    pub fn with_max_snapshots(max_snapshots: usize) -> Self {
        Self {
            baseline: MemoryAudit::capture(),
            snapshots: Vec::new(),
            max_snapshots,
        }
    }

    /// Take a snapshot of current memory usage
    pub fn snapshot(&mut self) -> &MemoryAudit {
        let audit = MemoryAudit::capture();
        self.snapshots.push(audit);

        // Trim history if needed
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }

        self.snapshots.last().unwrap()
    }

    /// Get the change since baseline
    pub fn change_since_baseline(&self) -> MemoryDiff {
        let current = MemoryAudit::capture();
        current.diff(&self.baseline)
    }

    /// Get the baseline snapshot
    pub fn baseline(&self) -> &MemoryAudit {
        &self.baseline
    }

    /// Reset the baseline to current state
    pub fn reset_baseline(&mut self) {
        self.baseline = MemoryAudit::capture();
        self.snapshots.clear();
    }

    /// Get all snapshots
    pub fn snapshots(&self) -> &[MemoryAudit] {
        &self.snapshots
    }

    /// Get the latest snapshot (or baseline if no snapshots taken)
    pub fn latest(&self) -> &MemoryAudit {
        self.snapshots.last().unwrap_or(&self.baseline)
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_audit_new() {
        let audit = MemoryAudit::new(1024, 10);
        assert_eq!(audit.heap_bytes, 1024);
        assert_eq!(audit.allocation_count, 10);
        assert_eq!(audit.peak_bytes, 1024);
    }

    #[test]
    fn test_memory_audit_capture() {
        let audit = MemoryAudit::capture();
        // Just verify it doesn't panic and returns reasonable values
        assert!(audit.timestamp >= Duration::ZERO);
    }

    #[test]
    fn test_memory_audit_capture_for_component() {
        let audit = MemoryAudit::capture_for_component("test_component");
        assert_eq!(audit.component_id, Some("test_component".to_string()));
    }

    #[test]
    fn test_memory_audit_diff() {
        let audit1 = MemoryAudit::new(1000, 10);
        let audit2 = MemoryAudit::new(2000, 15);

        let diff = audit2.diff(&audit1);
        assert_eq!(diff.heap_bytes_delta, 1000);
        assert_eq!(diff.allocation_count_delta, 5);
    }

    #[test]
    fn test_memory_audit_exceeds_threshold() {
        let audit = MemoryAudit::new(1000, 10);
        assert!(audit.exceeds_threshold(500));
        assert!(!audit.exceeds_threshold(1500));
    }

    #[test]
    fn test_memory_diff_is_growth() {
        let diff = MemoryDiff {
            heap_bytes_delta: 100,
            allocation_count_delta: 5,
            time_delta: Duration::from_secs(1),
        };
        assert!(diff.is_growth());

        let diff_shrink = MemoryDiff {
            heap_bytes_delta: -100,
            allocation_count_delta: -5,
            time_delta: Duration::from_secs(1),
        };
        assert!(!diff_shrink.is_growth());
    }

    #[test]
    fn test_memory_diff_bytes_per_second() {
        let diff = MemoryDiff {
            heap_bytes_delta: 1000,
            allocation_count_delta: 10,
            time_delta: Duration::from_secs(2),
        };
        assert!((diff.bytes_per_second() - 500.0).abs() < 0.1);
    }

    #[test]
    fn test_allocation_stats_new() {
        let stats = AllocationStats::new();
        assert_eq!(stats.allocation_count, 0);
        assert_eq!(stats.deallocation_count, 0);
        assert_eq!(stats.current_bytes, 0);
    }

    #[test]
    fn test_allocation_stats_has_potential_leak() {
        let stats_ok = AllocationStats {
            allocation_count: 1000,
            deallocation_count: 950,
            peak_bytes: 0,
            current_bytes: 0,
            largest_allocation: 0,
        };
        assert!(!stats_ok.has_potential_leak());

        let stats_leak = AllocationStats {
            allocation_count: 1000,
            deallocation_count: 500,
            peak_bytes: 0,
            current_bytes: 0,
            largest_allocation: 0,
        };
        assert!(stats_leak.has_potential_leak());
    }

    #[test]
    fn test_allocation_stats_net_allocations() {
        let stats = AllocationStats {
            allocation_count: 100,
            deallocation_count: 30,
            peak_bytes: 0,
            current_bytes: 0,
            largest_allocation: 0,
        };
        assert_eq!(stats.net_allocations(), 70);
    }

    #[test]
    fn test_get_heap_usage() {
        // Just verify it doesn't panic
        let _heap = get_heap_usage();
    }

    #[test]
    fn test_get_allocation_stats() {
        let stats = get_allocation_stats();
        // Verify it returns valid stats
        assert!(stats.peak_bytes >= stats.current_bytes || stats.current_bytes == 0);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(2 * 1024 * 1024), "2.00 MB");
    }

    #[test]
    fn test_memory_tracker_new() {
        let tracker = MemoryTracker::new();
        assert!(tracker.snapshots().is_empty());
    }

    #[test]
    fn test_memory_tracker_snapshot() {
        let mut tracker = MemoryTracker::new();
        let snapshot = tracker.snapshot();
        assert!(snapshot.timestamp >= Duration::ZERO);
        assert_eq!(tracker.snapshots().len(), 1);
    }

    #[test]
    fn test_memory_tracker_change_since_baseline() {
        let tracker = MemoryTracker::new();
        let diff = tracker.change_since_baseline();
        // Time should have increased
        assert!(diff.time_delta >= Duration::ZERO);
    }

    #[test]
    fn test_memory_tracker_reset_baseline() {
        let mut tracker = MemoryTracker::new();
        tracker.snapshot();
        tracker.snapshot();
        assert_eq!(tracker.snapshots().len(), 2);

        tracker.reset_baseline();
        assert!(tracker.snapshots().is_empty());
    }

    #[test]
    fn test_memory_tracker_max_snapshots() {
        let mut tracker = MemoryTracker::with_max_snapshots(3);
        for _ in 0..5 {
            tracker.snapshot();
        }
        assert_eq!(tracker.snapshots().len(), 3);
    }

    #[test]
    fn test_memory_audit_serialization() {
        let audit = MemoryAudit::new(1024, 10);
        let json = serde_json::to_string(&audit).unwrap();
        let deserialized: MemoryAudit = serde_json::from_str(&json).unwrap();
        assert_eq!(audit.heap_bytes, deserialized.heap_bytes);
        assert_eq!(audit.allocation_count, deserialized.allocation_count);
    }

    #[test]
    fn test_allocation_stats_serialization() {
        let stats = AllocationStats {
            allocation_count: 100,
            deallocation_count: 50,
            peak_bytes: 2048,
            current_bytes: 1024,
            largest_allocation: 512,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: AllocationStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats, deserialized);
    }
}
