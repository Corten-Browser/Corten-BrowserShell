// @implements: REQ-002
//! Process isolation management (mocked)
//!
//! Provides mock process isolation for tabs. In a real browser,
//! this would manage separate OS processes for security and stability.

use shared_types::{ProcessId, TabId};
use std::collections::HashMap;

/// Mock process manager for tab isolation
#[derive(Debug)]
pub struct MockProcessManager {
    processes: HashMap<TabId, ProcessId>,
    next_process_id: u32,
}

impl MockProcessManager {
    /// Create a new mock process manager
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
            next_process_id: 1000, // Start at 1000 to distinguish from real PIDs
        }
    }

    /// Allocate a process for a tab (mocked)
    pub fn allocate_process(&mut self, tab_id: TabId) -> ProcessId {
        let process_id = ProcessId(self.next_process_id);
        self.next_process_id += 1;
        self.processes.insert(tab_id, process_id);
        process_id
    }

    /// Get process ID for a tab
    pub fn get_process(&self, tab_id: TabId) -> Option<ProcessId> {
        self.processes.get(&tab_id).copied()
    }

    /// Release process for a tab (mocked)
    pub fn release_process(&mut self, tab_id: TabId) -> Option<ProcessId> {
        self.processes.remove(&tab_id)
    }

    /// Get all active processes
    pub fn active_processes(&self) -> Vec<ProcessId> {
        self.processes.values().copied().collect()
    }

    /// Get process count
    pub fn process_count(&self) -> usize {
        self.processes.len()
    }
}

impl Default for MockProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_process() {
        let mut manager = MockProcessManager::new();
        let tab_id = TabId::new();

        let process_id = manager.allocate_process(tab_id);
        assert_eq!(process_id.0, 1000);

        let retrieved = manager.get_process(tab_id);
        assert_eq!(retrieved, Some(process_id));
    }

    #[test]
    fn test_allocate_multiple_processes() {
        let mut manager = MockProcessManager::new();
        let tab1 = TabId::new();
        let tab2 = TabId::new();

        let pid1 = manager.allocate_process(tab1);
        let pid2 = manager.allocate_process(tab2);

        assert_ne!(pid1, pid2);
        assert_eq!(pid1.0, 1000);
        assert_eq!(pid2.0, 1001);
    }

    #[test]
    fn test_release_process() {
        let mut manager = MockProcessManager::new();
        let tab_id = TabId::new();

        let process_id = manager.allocate_process(tab_id);
        assert_eq!(manager.process_count(), 1);

        let released = manager.release_process(tab_id);
        assert_eq!(released, Some(process_id));
        assert_eq!(manager.process_count(), 0);
        assert_eq!(manager.get_process(tab_id), None);
    }

    #[test]
    fn test_active_processes() {
        let mut manager = MockProcessManager::new();
        let tab1 = TabId::new();
        let tab2 = TabId::new();

        manager.allocate_process(tab1);
        manager.allocate_process(tab2);

        let active = manager.active_processes();
        assert_eq!(active.len(), 2);
    }
}
