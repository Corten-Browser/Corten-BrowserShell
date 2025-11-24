//! Core extension types and traits

use crate::browser_action::BrowserAction;
use crate::context_menu::ContextMenuItem;
use crate::manifest::ExtensionManifest;
use crate::messaging::ExtensionMessage;
use crate::permissions::PermissionSet;

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;
use uuid::Uuid;

/// Result type for extension operations
pub type Result<T> = std::result::Result<T, ExtensionError>;

/// Extension-specific error types
#[derive(Error, Debug)]
pub enum ExtensionError {
    #[error("Extension not found: {0}")]
    NotFound(ExtensionId),

    #[error("Extension already registered: {0}")]
    AlreadyRegistered(ExtensionId),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("Invalid extension: {0}")]
    InvalidExtension(String),

    #[error("Messaging error: {0}")]
    MessagingError(String),

    #[error("Content script error: {0}")]
    ContentScriptError(String),
}

/// Unique identifier for extensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExtensionId(Uuid);

impl ExtensionId {
    /// Create a new random ExtensionId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create an ExtensionId from a string identifier
    pub fn from_string(id: &str) -> Self {
        // Generate a deterministic UUID from the string
        Self(Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes()))
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ExtensionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ExtensionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Extension lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionState {
    /// Extension is installed but not enabled
    Disabled,
    /// Extension is enabled and running
    Enabled,
    /// Extension is being installed
    Installing,
    /// Extension encountered an error
    Error,
}

impl Default for ExtensionState {
    fn default() -> Self {
        Self::Disabled
    }
}

/// Content script injection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScript {
    /// Script files to inject
    pub js: Vec<String>,
    /// CSS files to inject
    pub css: Vec<String>,
    /// URL patterns to match
    pub matches: Vec<ContentScriptMatch>,
    /// URL patterns to exclude
    pub exclude_matches: Vec<ContentScriptMatch>,
    /// When to inject the script
    pub run_at: ContentScriptRunAt,
    /// Whether to run in all frames
    pub all_frames: bool,
}

impl Default for ContentScript {
    fn default() -> Self {
        Self {
            js: Vec::new(),
            css: Vec::new(),
            matches: Vec::new(),
            exclude_matches: Vec::new(),
            run_at: ContentScriptRunAt::DocumentIdle,
            all_frames: false,
        }
    }
}

/// URL pattern for content script matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScriptMatch {
    /// The URL pattern (supports * wildcards)
    pub pattern: String,
}

impl ContentScriptMatch {
    /// Create a new content script match pattern
    pub fn new(pattern: String) -> Self {
        Self { pattern }
    }

    /// Check if a URL matches this pattern
    pub fn matches(&self, url: &str) -> bool {
        // Simple wildcard matching
        // Pattern: scheme://host/path where * matches any characters
        let pattern = &self.pattern;

        if pattern == "<all_urls>" {
            return true;
        }

        // Convert pattern to regex-like matching
        let regex_pattern = pattern
            .replace('.', r"\.")
            .replace('*', ".*")
            .replace('?', ".");

        regex::Regex::new(&format!("^{}$", regex_pattern))
            .map(|re| re.is_match(url))
            .unwrap_or(false)
    }
}

/// When to inject content scripts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentScriptRunAt {
    /// Inject as soon as possible
    DocumentStart,
    /// Inject after DOM is ready but before images load
    DocumentEnd,
    /// Inject after page is fully loaded
    DocumentIdle,
}

impl Default for ContentScriptRunAt {
    fn default() -> Self {
        Self::DocumentIdle
    }
}

/// Browser extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    /// Unique identifier
    pub id: ExtensionId,
    /// Internal name (from manifest)
    pub name: String,
    /// Display name
    pub display_name: String,
    /// Version string
    pub version: String,
    /// Description
    pub description: String,
    /// Current state
    pub state: ExtensionState,
    /// Granted permissions
    pub permissions: PermissionSet,
    /// Browser action (toolbar button)
    pub browser_action: Option<BrowserAction>,
    /// Content scripts
    pub content_scripts: Vec<ContentScript>,
    /// Context menu items
    pub context_menu_items: Vec<ContextMenuItem>,
    /// Homepage URL
    pub homepage_url: Option<String>,
    /// Icons (size -> path)
    pub icons: std::collections::HashMap<u32, String>,
}

impl Extension {
    /// Create a new extension with minimal configuration
    pub fn new(name: String, display_name: String, version: String) -> Self {
        Self {
            id: ExtensionId::from_string(&name),
            name,
            display_name,
            version,
            description: String::new(),
            state: ExtensionState::default(),
            permissions: PermissionSet::default(),
            browser_action: None,
            content_scripts: Vec::new(),
            context_menu_items: Vec::new(),
            homepage_url: None,
            icons: std::collections::HashMap::new(),
        }
    }

    /// Create an extension from a parsed manifest
    pub fn from_manifest(manifest: ExtensionManifest) -> Result<Self> {
        let id = ExtensionId::from_string(&manifest.name);

        Ok(Self {
            id,
            name: manifest.name,
            display_name: manifest.display_name,
            version: manifest.version,
            description: manifest.description,
            state: ExtensionState::Disabled,
            permissions: manifest.permissions,
            browser_action: manifest.browser_action,
            content_scripts: manifest.content_scripts,
            context_menu_items: Vec::new(), // Added programmatically
            homepage_url: manifest.homepage_url,
            icons: manifest.icons,
        })
    }

    /// Validate the extension configuration
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(ExtensionError::InvalidExtension(
                "Extension name cannot be empty".to_string(),
            ));
        }

        if self.version.is_empty() {
            return Err(ExtensionError::InvalidExtension(
                "Extension version cannot be empty".to_string(),
            ));
        }

        // Validate content script patterns
        for script in &self.content_scripts {
            for match_pattern in &script.matches {
                if match_pattern.pattern.is_empty() {
                    return Err(ExtensionError::InvalidExtension(
                        "Content script match pattern cannot be empty".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check if the extension has a specific permission
    pub fn has_permission(&self, permission: &crate::permissions::Permission) -> bool {
        self.permissions.contains(permission)
    }

    /// Add a content script
    pub fn add_content_script(&mut self, script: ContentScript) {
        self.content_scripts.push(script);
    }

    /// Add a context menu item
    pub fn add_context_menu_item(&mut self, item: ContextMenuItem) {
        self.context_menu_items.push(item);
    }
}

/// Trait for hosting and managing extensions
pub trait ExtensionHost: Send + Sync {
    /// Register a new extension
    fn register(
        &mut self,
        extension: Extension,
    ) -> Pin<Box<dyn Future<Output = Result<ExtensionId>> + Send + '_>>;

    /// Unregister an extension
    fn unregister(
        &mut self,
        id: ExtensionId,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Get an extension by ID
    fn get_extension(
        &self,
        id: ExtensionId,
    ) -> Pin<Box<dyn Future<Output = Option<Extension>> + Send + '_>>;

    /// Send a message from an extension
    fn send_message(
        &self,
        from: ExtensionId,
        message: ExtensionMessage,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_id_from_string() {
        let id1 = ExtensionId::from_string("test-extension");
        let id2 = ExtensionId::from_string("test-extension");
        let id3 = ExtensionId::from_string("other-extension");

        // Same string should produce same ID
        assert_eq!(id1, id2);
        // Different strings should produce different IDs
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_extension_creation() {
        let ext = Extension::new(
            "my-ext".to_string(),
            "My Extension".to_string(),
            "1.0.0".to_string(),
        );

        assert_eq!(ext.name, "my-ext");
        assert_eq!(ext.display_name, "My Extension");
        assert_eq!(ext.version, "1.0.0");
        assert_eq!(ext.state, ExtensionState::Disabled);
    }

    #[test]
    fn test_extension_validation() {
        let valid = Extension::new(
            "test".to_string(),
            "Test".to_string(),
            "1.0.0".to_string(),
        );
        assert!(valid.validate().is_ok());

        let invalid_name = Extension::new(String::new(), "Test".to_string(), "1.0.0".to_string());
        assert!(invalid_name.validate().is_err());

        let invalid_version =
            Extension::new("test".to_string(), "Test".to_string(), String::new());
        assert!(invalid_version.validate().is_err());
    }

    #[test]
    fn test_content_script_match() {
        let all_urls = ContentScriptMatch::new("<all_urls>".to_string());
        assert!(all_urls.matches("https://example.com"));
        assert!(all_urls.matches("http://test.org/page"));

        let specific = ContentScriptMatch::new("https://example.com/*".to_string());
        assert!(specific.matches("https://example.com/page"));
        assert!(specific.matches("https://example.com/"));
        assert!(!specific.matches("https://other.com/page"));
    }
}
