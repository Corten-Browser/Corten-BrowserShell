//! Extension permission system
//!
//! Provides a Chrome extension manifest v3 compatible permission system.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Extension permissions (Manifest V3 compatible)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // Core permissions
    /// Access to active tab
    ActiveTab,
    /// Access to browser alarms API
    Alarms,
    /// Access to bookmarks API
    Bookmarks,
    /// Access to browsing data
    BrowsingData,
    /// Access to clipboard read
    ClipboardRead,
    /// Access to clipboard write
    ClipboardWrite,
    /// Access to context menus API
    ContextMenus,
    /// Access to cookies API
    Cookies,
    /// Access to debugger API
    Debugger,
    /// Access to declarative content API
    DeclarativeContent,
    /// Access to downloads API
    Downloads,
    /// Access to history API
    History,
    /// Access to identity API
    Identity,
    /// Access to idle detection
    Idle,
    /// Access to management API
    Management,
    /// Access to native messaging
    NativeMessaging,
    /// Access to notifications API
    Notifications,
    /// Access to page capture
    PageCapture,
    /// Access to power API
    Power,
    /// Access to privacy settings
    Privacy,
    /// Access to proxy settings
    Proxy,
    /// Access to scripting API
    Scripting,
    /// Access to sessions API
    Sessions,
    /// Access to storage API
    Storage,
    /// Access to system CPU info
    SystemCpu,
    /// Access to system memory info
    SystemMemory,
    /// Access to system storage info
    SystemStorage,
    /// Access to tab capture
    TabCapture,
    /// Access to tabs API
    Tabs,
    /// Access to top sites
    TopSites,
    /// Access to tts (text-to-speech)
    Tts,
    /// Access to tts engine
    TtsEngine,
    /// Access to unlimited storage
    UnlimitedStorage,
    /// Access to web navigation API
    WebNavigation,
    /// Access to web request API
    WebRequest,
    /// Access to web request blocking
    WebRequestBlocking,

    // Host permissions (URL patterns)
    /// Access to specific hosts
    Host(String),

    // Optional permissions
    /// Optional permission that can be requested at runtime
    Optional(Box<Permission>),

    // Unknown permission (for forward compatibility)
    /// Unknown permission string
    Unknown(String),
}

impl Permission {
    /// Parse a permission string from manifest
    pub fn from_string(s: &str) -> Self {
        match s {
            "activeTab" => Permission::ActiveTab,
            "alarms" => Permission::Alarms,
            "bookmarks" => Permission::Bookmarks,
            "browsingData" => Permission::BrowsingData,
            "clipboardRead" => Permission::ClipboardRead,
            "clipboardWrite" => Permission::ClipboardWrite,
            "contextMenus" => Permission::ContextMenus,
            "cookies" => Permission::Cookies,
            "debugger" => Permission::Debugger,
            "declarativeContent" => Permission::DeclarativeContent,
            "downloads" => Permission::Downloads,
            "history" => Permission::History,
            "identity" => Permission::Identity,
            "idle" => Permission::Idle,
            "management" => Permission::Management,
            "nativeMessaging" => Permission::NativeMessaging,
            "notifications" => Permission::Notifications,
            "pageCapture" => Permission::PageCapture,
            "power" => Permission::Power,
            "privacy" => Permission::Privacy,
            "proxy" => Permission::Proxy,
            "scripting" => Permission::Scripting,
            "sessions" => Permission::Sessions,
            "storage" => Permission::Storage,
            "system.cpu" => Permission::SystemCpu,
            "system.memory" => Permission::SystemMemory,
            "system.storage" => Permission::SystemStorage,
            "tabCapture" => Permission::TabCapture,
            "tabs" => Permission::Tabs,
            "topSites" => Permission::TopSites,
            "tts" => Permission::Tts,
            "ttsEngine" => Permission::TtsEngine,
            "unlimitedStorage" => Permission::UnlimitedStorage,
            "webNavigation" => Permission::WebNavigation,
            "webRequest" => Permission::WebRequest,
            "webRequestBlocking" => Permission::WebRequestBlocking,
            s if s.contains("://") || s.starts_with("<all_urls>") || s.starts_with('*') => {
                Permission::Host(s.to_string())
            }
            _ => Permission::Unknown(s.to_string()),
        }
    }

    /// Convert to manifest string representation
    pub fn to_manifest_string(&self) -> String {
        match self {
            Permission::ActiveTab => "activeTab".to_string(),
            Permission::Alarms => "alarms".to_string(),
            Permission::Bookmarks => "bookmarks".to_string(),
            Permission::BrowsingData => "browsingData".to_string(),
            Permission::ClipboardRead => "clipboardRead".to_string(),
            Permission::ClipboardWrite => "clipboardWrite".to_string(),
            Permission::ContextMenus => "contextMenus".to_string(),
            Permission::Cookies => "cookies".to_string(),
            Permission::Debugger => "debugger".to_string(),
            Permission::DeclarativeContent => "declarativeContent".to_string(),
            Permission::Downloads => "downloads".to_string(),
            Permission::History => "history".to_string(),
            Permission::Identity => "identity".to_string(),
            Permission::Idle => "idle".to_string(),
            Permission::Management => "management".to_string(),
            Permission::NativeMessaging => "nativeMessaging".to_string(),
            Permission::Notifications => "notifications".to_string(),
            Permission::PageCapture => "pageCapture".to_string(),
            Permission::Power => "power".to_string(),
            Permission::Privacy => "privacy".to_string(),
            Permission::Proxy => "proxy".to_string(),
            Permission::Scripting => "scripting".to_string(),
            Permission::Sessions => "sessions".to_string(),
            Permission::Storage => "storage".to_string(),
            Permission::SystemCpu => "system.cpu".to_string(),
            Permission::SystemMemory => "system.memory".to_string(),
            Permission::SystemStorage => "system.storage".to_string(),
            Permission::TabCapture => "tabCapture".to_string(),
            Permission::Tabs => "tabs".to_string(),
            Permission::TopSites => "topSites".to_string(),
            Permission::Tts => "tts".to_string(),
            Permission::TtsEngine => "ttsEngine".to_string(),
            Permission::UnlimitedStorage => "unlimitedStorage".to_string(),
            Permission::WebNavigation => "webNavigation".to_string(),
            Permission::WebRequest => "webRequest".to_string(),
            Permission::WebRequestBlocking => "webRequestBlocking".to_string(),
            Permission::Host(s) => s.clone(),
            Permission::Optional(p) => p.to_manifest_string(),
            Permission::Unknown(s) => s.clone(),
        }
    }

    /// Check if this is a dangerous permission that requires user approval
    pub fn is_dangerous(&self) -> bool {
        matches!(
            self,
            Permission::Debugger
                | Permission::History
                | Permission::Management
                | Permission::NativeMessaging
                | Permission::PageCapture
                | Permission::Privacy
                | Permission::Proxy
                | Permission::TabCapture
                | Permission::WebRequestBlocking
                | Permission::Host(_)
        )
    }
}

/// A set of permissions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    /// Create a new empty permission set
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }

    /// Create from a list of permission strings
    pub fn from_strings(strings: &[String]) -> Self {
        let permissions = strings.iter().map(|s| Permission::from_string(s)).collect();
        Self { permissions }
    }

    /// Add a permission
    pub fn add(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Remove a permission
    pub fn remove(&mut self, permission: &Permission) -> bool {
        self.permissions.remove(permission)
    }

    /// Check if the set contains a permission
    pub fn contains(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    /// Get all permissions
    pub fn iter(&self) -> impl Iterator<Item = &Permission> {
        self.permissions.iter()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }

    /// Get count
    pub fn len(&self) -> usize {
        self.permissions.len()
    }

    /// Get all dangerous permissions
    pub fn dangerous_permissions(&self) -> Vec<&Permission> {
        self.permissions.iter().filter(|p| p.is_dangerous()).collect()
    }

    /// Check if set contains any host permissions
    pub fn has_host_permissions(&self) -> bool {
        self.permissions
            .iter()
            .any(|p| matches!(p, Permission::Host(_)))
    }

    /// Get all host permission patterns
    pub fn host_patterns(&self) -> Vec<&str> {
        self.permissions
            .iter()
            .filter_map(|p| match p {
                Permission::Host(s) => Some(s.as_str()),
                _ => None,
            })
            .collect()
    }
}

/// Request for runtime permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    /// Extension requesting the permission
    pub extension_id: crate::types::ExtensionId,
    /// Permissions being requested
    pub permissions: Vec<Permission>,
    /// Optional permissions being requested
    pub optional_permissions: Vec<Permission>,
    /// Host patterns being requested
    pub host_patterns: Vec<String>,
}

impl PermissionRequest {
    /// Create a new permission request
    pub fn new(extension_id: crate::types::ExtensionId) -> Self {
        Self {
            extension_id,
            permissions: Vec::new(),
            optional_permissions: Vec::new(),
            host_patterns: Vec::new(),
        }
    }

    /// Add a permission to request
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.push(permission);
    }

    /// Add an optional permission to request
    pub fn add_optional_permission(&mut self, permission: Permission) {
        self.optional_permissions.push(permission);
    }

    /// Add a host pattern to request
    pub fn add_host_pattern(&mut self, pattern: String) {
        self.host_patterns.push(pattern);
    }

    /// Check if request contains any dangerous permissions
    pub fn has_dangerous_permissions(&self) -> bool {
        self.permissions.iter().any(|p| p.is_dangerous())
            || self.optional_permissions.iter().any(|p| p.is_dangerous())
            || !self.host_patterns.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_from_string() {
        assert_eq!(Permission::from_string("activeTab"), Permission::ActiveTab);
        assert_eq!(Permission::from_string("storage"), Permission::Storage);
        assert_eq!(
            Permission::from_string("https://example.com/*"),
            Permission::Host("https://example.com/*".to_string())
        );
        assert!(matches!(
            Permission::from_string("unknown_perm"),
            Permission::Unknown(_)
        ));
    }

    #[test]
    fn test_permission_set() {
        let mut set = PermissionSet::new();
        assert!(set.is_empty());

        set.add(Permission::Storage);
        set.add(Permission::Tabs);

        assert_eq!(set.len(), 2);
        assert!(set.contains(&Permission::Storage));
        assert!(set.contains(&Permission::Tabs));
        assert!(!set.contains(&Permission::History));
    }

    #[test]
    fn test_dangerous_permissions() {
        assert!(Permission::Debugger.is_dangerous());
        assert!(Permission::History.is_dangerous());
        assert!(Permission::Host("*://*/*".to_string()).is_dangerous());
        assert!(!Permission::Storage.is_dangerous());
        assert!(!Permission::Tabs.is_dangerous());
    }

    #[test]
    fn test_permission_set_from_strings() {
        let strings = vec![
            "storage".to_string(),
            "tabs".to_string(),
            "https://example.com/*".to_string(),
        ];

        let set = PermissionSet::from_strings(&strings);
        assert_eq!(set.len(), 3);
        assert!(set.contains(&Permission::Storage));
        assert!(set.contains(&Permission::Tabs));
        assert!(set.has_host_permissions());
    }

    #[test]
    fn test_host_patterns() {
        let strings = vec![
            "storage".to_string(),
            "https://example.com/*".to_string(),
            "https://test.org/*".to_string(),
        ];

        let set = PermissionSet::from_strings(&strings);
        let patterns = set.host_patterns();

        assert_eq!(patterns.len(), 2);
        assert!(patterns.contains(&"https://example.com/*"));
        assert!(patterns.contains(&"https://test.org/*"));
    }
}
