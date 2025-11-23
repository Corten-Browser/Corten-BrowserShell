//! Chrome-compatible Extension APIs
//!
//! This module provides Chrome extension API compatibility for the browser shell.
//! APIs are registered with extensions and can be called via extension messaging.
//!
//! ## Available APIs
//!
//! - [`TabsApi`] - chrome.tabs compatible API for tab manipulation
//! - [`WindowsApi`] - chrome.windows compatible API for window management
//! - [`BookmarksApi`] - chrome.bookmarks compatible API for bookmark operations
//!
//! ## Permission Checking
//!
//! All API calls are gated by the extension's granted permissions. The permission
//! system follows Chrome's Manifest V3 model.

mod bookmarks;
mod tabs;
mod windows;

pub use bookmarks::{
    BookmarkTreeNode, BookmarksApi, CreateBookmarkDetails, MoveBookmarkDetails,
    UpdateBookmarkDetails,
};
pub use tabs::{
    CreateTabProperties, QueryTabInfo, TabsApi, TabsApiError, UpdateTabProperties,
};
pub use windows::{
    CreateWindowOptions, UpdateWindowOptions, WindowState, WindowType, WindowsApi, WindowsApiError,
};

use crate::permissions::{Permission, PermissionSet};
use crate::types::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Trait for extension API implementations
///
/// Each Chrome-compatible API implements this trait to provide
/// standardized registration and permission checking.
pub trait ExtensionApi: Send + Sync {
    /// Get the API namespace (e.g., "tabs", "windows", "bookmarks")
    fn namespace(&self) -> &str;

    /// Get the required permission for this API
    fn required_permission(&self) -> Permission;

    /// Check if an extension has access to this API
    fn check_permission(&self, permissions: &PermissionSet) -> bool {
        permissions.contains(&self.required_permission())
    }
}

/// API registry for managing extension APIs
#[derive(Default)]
pub struct ApiRegistry {
    /// Registered APIs by namespace
    apis: std::collections::HashMap<String, Arc<dyn ExtensionApi>>,
}

impl ApiRegistry {
    /// Create a new empty API registry
    pub fn new() -> Self {
        Self {
            apis: std::collections::HashMap::new(),
        }
    }

    /// Register an API
    pub fn register(&mut self, api: Arc<dyn ExtensionApi>) {
        self.apis.insert(api.namespace().to_string(), api);
    }

    /// Get an API by namespace
    pub fn get(&self, namespace: &str) -> Option<Arc<dyn ExtensionApi>> {
        self.apis.get(namespace).cloned()
    }

    /// Check if an extension has permission to use an API
    pub fn can_use_api(&self, namespace: &str, permissions: &PermissionSet) -> bool {
        if let Some(api) = self.apis.get(namespace) {
            api.check_permission(permissions)
        } else {
            false
        }
    }

    /// Get all registered API namespaces
    pub fn namespaces(&self) -> Vec<&str> {
        self.apis.keys().map(|s| s.as_str()).collect()
    }
}

/// Result of an API call
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum ApiResult<T> {
    /// Successful result
    #[serde(rename = "success")]
    Success { data: T },
    /// Error result
    #[serde(rename = "error")]
    Error { message: String, code: Option<String> },
}

impl<T> ApiResult<T> {
    /// Create a success result
    pub fn success(data: T) -> Self {
        Self::Success { data }
    }

    /// Create an error result
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            code: None,
        }
    }

    /// Create an error result with code
    pub fn error_with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            code: Some(code.into()),
        }
    }

    /// Check if result is success
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if result is error
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }
}

impl<T> From<Result<T>> for ApiResult<T> {
    fn from(result: Result<T>) -> Self {
        match result {
            Ok(data) => Self::success(data),
            Err(e) => Self::error(e.to_string()),
        }
    }
}

/// Common query options for listing APIs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryOptions {
    /// Filter by active state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,

    /// Filter by pinned state (tabs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,

    /// Filter by URL pattern
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Filter by window ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_id: Option<u64>,

    /// Filter by current window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_window: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_result_success() {
        let result: ApiResult<i32> = ApiResult::success(42);
        assert!(result.is_success());
        assert!(!result.is_error());
    }

    #[test]
    fn test_api_result_error() {
        let result: ApiResult<i32> = ApiResult::error("Something went wrong");
        assert!(result.is_error());
        assert!(!result.is_success());
    }

    #[test]
    fn test_api_registry() {
        let registry = ApiRegistry::new();
        assert!(registry.namespaces().is_empty());
    }
}
