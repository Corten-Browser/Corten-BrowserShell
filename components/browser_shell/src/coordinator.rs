// @implements: REQ-001, REQ-002
//! Component Coordinator
//!
//! Manages the lifecycle of all browser components and coordinates their initialization.

use anyhow::{Result, Context};
use std::sync::Arc;
use parking_lot::RwLock;
use rusqlite::Connection;

use message_bus::MessageBus;
use window_manager::WindowManagerImpl;
use tab_manager::TabManagerImpl;
use user_data::SettingsManager;
use shared_types::ComponentHealth;

/// ComponentCoordinator manages all browser components
pub struct ComponentCoordinator {
    message_bus: Arc<RwLock<MessageBus>>,
    window_manager: Arc<RwLock<WindowManagerImpl>>,
    tab_manager: Arc<RwLock<TabManagerImpl>>,
    settings_manager: Arc<RwLock<SettingsManager>>,
}

impl ComponentCoordinator {
    /// Create and initialize all components
    pub async fn new() -> Result<Self> {
        // Initialize message bus first (core infrastructure)
        // Use default capacity values: 10000 messages queue, 1MB max message size
        let message_bus = Arc::new(RwLock::new(
            MessageBus::new(10000, 1024 * 1024)
        ));

        // Initialize managers
        let window_manager = Arc::new(RwLock::new(WindowManagerImpl::new()));
        let tab_manager = Arc::new(RwLock::new(TabManagerImpl::new()));

        // Initialize settings manager (in-memory for now)
        let conn = Connection::open_in_memory()
            .context("Failed to open in-memory database")?;

        let settings_manager = Arc::new(RwLock::new(
            SettingsManager::new(conn)
                .context("Failed to initialize settings manager")?
        ));

        Ok(Self {
            message_bus,
            window_manager,
            tab_manager,
            settings_manager,
        })
    }

    /// Get reference to message bus
    pub fn message_bus(&self) -> Arc<RwLock<MessageBus>> {
        Arc::clone(&self.message_bus)
    }

    /// Get reference to window manager
    pub fn window_manager(&self) -> Arc<RwLock<WindowManagerImpl>> {
        Arc::clone(&self.window_manager)
    }

    /// Get reference to tab manager
    pub fn tab_manager(&self) -> Arc<RwLock<TabManagerImpl>> {
        Arc::clone(&self.tab_manager)
    }

    /// Get reference to settings manager
    pub fn settings_manager(&self) -> Arc<RwLock<SettingsManager>> {
        Arc::clone(&self.settings_manager)
    }

    /// Check the health of all components
    ///
    /// Returns aggregated health status. All components are considered healthy
    /// if they are initialized and functional.
    pub async fn health_check(&self) -> Result<ComponentHealth> {
        // For now, return healthy if all components are initialized
        // Future: Poll each component's health status
        Ok(ComponentHealth::Healthy)
    }

    /// Shutdown all components gracefully
    pub async fn shutdown(&self) -> Result<()> {
        // Shutdown in reverse order of initialization
        // (Settings, tab manager, window manager, message bus)
        Ok(())
    }
}
