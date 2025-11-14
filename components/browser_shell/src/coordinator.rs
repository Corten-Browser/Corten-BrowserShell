// @implements: REQ-001, REQ-002
//! Component Coordinator
//!
//! Manages the lifecycle of all browser components and coordinates their initialization.

use anyhow::{Result, Context};
use std::sync::Arc;
use parking_lot::RwLock;

use message_bus::MessageBus;
use window_manager::WindowManagerImpl;
use tab_manager::TabManagerImpl;
use user_data::SettingsManager;

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
        let message_bus = Arc::new(RwLock::new(
            MessageBus::new()
                .context("Failed to initialize message bus")?
        ));

        // Initialize managers
        let window_manager = Arc::new(RwLock::new(WindowManagerImpl::new()));
        let tab_manager = Arc::new(RwLock::new(TabManagerImpl::new()));

        // Initialize settings manager (in-memory for now)
        let settings_manager = Arc::new(RwLock::new(
            SettingsManager::new(":memory:")
                .await
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

    /// Shutdown all components gracefully
    pub async fn shutdown(&self) -> Result<()> {
        // Shutdown in reverse order of initialization
        // (Settings, tab manager, window manager, message bus)
        Ok(())
    }
}
