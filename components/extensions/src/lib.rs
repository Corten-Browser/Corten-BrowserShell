//! Extensions Component
//!
//! Provides extension system hooks and APIs for the CortenBrowser Browser Shell.
//!
//! This component provides:
//! - Extension registration and lifecycle management
//! - Browser action API (toolbar buttons with popups)
//! - Context menu contribution points
//! - Extension messaging (content scripts <-> background)
//! - Permission system for extension capabilities
//! - Manifest parsing (Chrome extension manifest v3 compatible)

mod browser_action;
mod context_menu;
mod manifest;
mod messaging;
mod permissions;
mod types;

pub use browser_action::{BrowserAction, BrowserActionApi, BrowserActionState, PopupConfig};
pub use context_menu::{ContextMenuApi, ContextMenuItem, ContextMenuItemType, MenuContext};
pub use manifest::{ExtensionManifest, ManifestParseError};
pub use messaging::{ExtensionMessage, MessageChannel, MessageSender, MessagingApi};
pub use permissions::{Permission, PermissionRequest, PermissionSet};
pub use types::{
    ContentScript, ContentScriptMatch, Extension, ExtensionError, ExtensionHost, ExtensionId,
    ExtensionState, Result,
};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Extension Manager
///
/// Central manager for all browser extensions. Implements the ExtensionHost trait
/// and coordinates between different extension APIs.
pub struct ExtensionManager {
    /// Registered extensions
    extensions: Arc<RwLock<HashMap<ExtensionId, Extension>>>,
    /// Browser action API
    browser_action_api: Arc<RwLock<BrowserActionApi>>,
    /// Context menu API
    context_menu_api: Arc<RwLock<ContextMenuApi>>,
    /// Messaging API
    messaging_api: Arc<RwLock<MessagingApi>>,
}

impl ExtensionManager {
    /// Create a new ExtensionManager
    pub fn new() -> Self {
        Self {
            extensions: Arc::new(RwLock::new(HashMap::new())),
            browser_action_api: Arc::new(RwLock::new(BrowserActionApi::new())),
            context_menu_api: Arc::new(RwLock::new(ContextMenuApi::new())),
            messaging_api: Arc::new(RwLock::new(MessagingApi::new())),
        }
    }

    /// Load an extension from a manifest
    pub async fn load_from_manifest(&mut self, manifest_json: &str) -> Result<ExtensionId> {
        let manifest = ExtensionManifest::parse(manifest_json)?;
        let extension = Extension::from_manifest(manifest)?;
        self.register(extension).await
    }

    /// Get the browser action API
    pub fn browser_action_api(&self) -> Arc<RwLock<BrowserActionApi>> {
        Arc::clone(&self.browser_action_api)
    }

    /// Get the context menu API
    pub fn context_menu_api(&self) -> Arc<RwLock<ContextMenuApi>> {
        Arc::clone(&self.context_menu_api)
    }

    /// Get the messaging API
    pub fn messaging_api(&self) -> Arc<RwLock<MessagingApi>> {
        Arc::clone(&self.messaging_api)
    }

    /// Get all registered extension IDs
    pub async fn list_extensions(&self) -> Vec<ExtensionId> {
        let extensions = self.extensions.read().await;
        extensions.keys().cloned().collect()
    }

    /// Enable an extension
    pub async fn enable(&self, id: ExtensionId) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        let extension = extensions
            .get_mut(&id)
            .ok_or(ExtensionError::NotFound(id))?;

        extension.state = ExtensionState::Enabled;

        // Register browser action if present
        if let Some(ref browser_action) = extension.browser_action {
            let mut ba_api = self.browser_action_api.write().await;
            ba_api.register(id, browser_action.clone());
        }

        // Register context menu items
        let mut cm_api = self.context_menu_api.write().await;
        for item in &extension.context_menu_items {
            cm_api.add_item(id, item.clone());
        }

        Ok(())
    }

    /// Disable an extension
    pub async fn disable(&self, id: ExtensionId) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        let extension = extensions
            .get_mut(&id)
            .ok_or(ExtensionError::NotFound(id))?;

        extension.state = ExtensionState::Disabled;

        // Unregister browser action
        let mut ba_api = self.browser_action_api.write().await;
        ba_api.unregister(id);

        // Unregister context menu items
        let mut cm_api = self.context_menu_api.write().await;
        cm_api.remove_all_for_extension(id);

        Ok(())
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionHost for ExtensionManager {
    fn register(
        &mut self,
        extension: Extension,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ExtensionId>> + Send + '_>>
    {
        Box::pin(async move {
            let id = extension.id;

            // Validate extension
            extension.validate()?;

            // Check for duplicate
            let extensions = self.extensions.read().await;
            if extensions.contains_key(&id) {
                return Err(ExtensionError::AlreadyRegistered(id));
            }
            drop(extensions);

            // Register the extension
            let mut extensions = self.extensions.write().await;
            extensions.insert(id, extension);

            Ok(id)
        })
    }

    fn unregister(
        &mut self,
        id: ExtensionId,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            // Disable first to clean up APIs
            let _ = self.disable(id).await;

            let mut extensions = self.extensions.write().await;
            extensions
                .remove(&id)
                .ok_or(ExtensionError::NotFound(id))?;

            // Clean up messaging channels
            let mut msg_api = self.messaging_api.write().await;
            msg_api.remove_extension(id);

            Ok(())
        })
    }

    fn get_extension(
        &self,
        id: ExtensionId,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<Extension>> + Send + '_>> {
        Box::pin(async move {
            let extensions = self.extensions.read().await;
            extensions.get(&id).cloned()
        })
    }

    fn send_message(
        &self,
        from: ExtensionId,
        message: ExtensionMessage,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let msg_api = self.messaging_api.read().await;
            msg_api.send(from, message).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extension_manager_creation() {
        let manager = ExtensionManager::new();
        let extensions = manager.list_extensions().await;
        assert!(extensions.is_empty());
    }

    #[tokio::test]
    async fn test_register_extension() {
        let mut manager = ExtensionManager::new();
        let extension = Extension::new(
            "test-extension".to_string(),
            "Test Extension".to_string(),
            "1.0.0".to_string(),
        );
        let id = extension.id;

        let result = manager.register(extension).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), id);

        let extensions = manager.list_extensions().await;
        assert_eq!(extensions.len(), 1);
    }

    #[tokio::test]
    async fn test_unregister_extension() {
        let mut manager = ExtensionManager::new();
        let extension = Extension::new(
            "test-extension".to_string(),
            "Test Extension".to_string(),
            "1.0.0".to_string(),
        );
        let id = extension.id;

        manager.register(extension).await.unwrap();
        let result = manager.unregister(id).await;
        assert!(result.is_ok());

        let extensions = manager.list_extensions().await;
        assert!(extensions.is_empty());
    }

    #[tokio::test]
    async fn test_enable_disable_extension() {
        let mut manager = ExtensionManager::new();
        let extension = Extension::new(
            "test-extension".to_string(),
            "Test Extension".to_string(),
            "1.0.0".to_string(),
        );
        let id = extension.id;

        manager.register(extension).await.unwrap();

        // Enable
        let result = manager.enable(id).await;
        assert!(result.is_ok());

        let ext = manager.get_extension(id).await.unwrap();
        assert_eq!(ext.state, ExtensionState::Enabled);

        // Disable
        let result = manager.disable(id).await;
        assert!(result.is_ok());

        let ext = manager.get_extension(id).await.unwrap();
        assert_eq!(ext.state, ExtensionState::Disabled);
    }
}
