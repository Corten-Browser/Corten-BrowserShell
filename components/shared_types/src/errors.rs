//! Error types for browser shell components
//!
//! This module provides error types for various component operations.
//! All error types implement std::error::Error for proper error handling.

use crate::{TabId, WindowId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during component operations
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum ComponentError {
    /// Component failed to initialize
    #[error("Component initialization failed: {0}")]
    InitializationFailed(String),

    /// Message routing between components failed
    #[error("Message routing failed: {0}")]
    MessageRoutingFailed(String),

    /// Component is in an invalid state
    #[error("Invalid component state: {0}")]
    InvalidState(String),

    /// Requested resource was not found
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Operation was denied due to insufficient permissions
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// Errors that can occur during window operations
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum WindowError {
    /// Window creation failed
    #[error("Window creation failed: {0}")]
    CreationFailed(String),

    /// Window with the specified ID was not found
    #[error("Window not found: {0:?}")]
    NotFound(WindowId),

    /// Invalid window configuration
    #[error("Invalid window configuration: {0}")]
    InvalidConfig(String),

    /// Platform-specific error occurred
    #[error("Platform error: {0}")]
    PlatformError(String),
}

/// Errors that can occur during tab operations
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum TabError {
    /// Tab creation failed
    #[error("Tab creation failed: {0}")]
    CreationFailed(String),

    /// Tab with the specified ID was not found
    #[error("Tab not found: {0:?}")]
    NotFound(TabId),

    /// Navigation to URL failed
    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    /// Process isolation setup failed
    #[error("Process isolation failed: {0}")]
    ProcessIsolationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_component_error_is_error() {
        let error = ComponentError::InitializationFailed("test".to_string());
        let _: &dyn Error = &error;
    }

    #[test]
    fn test_window_error_is_error() {
        let error = WindowError::CreationFailed("test".to_string());
        let _: &dyn Error = &error;
    }

    #[test]
    fn test_tab_error_is_error() {
        let error = TabError::CreationFailed("test".to_string());
        let _: &dyn Error = &error;
    }

    #[test]
    fn test_error_messages() {
        let component_err = ComponentError::InitializationFailed("init error".to_string());
        assert!(component_err.to_string().contains("init error"));

        let window_err = WindowError::InvalidConfig("bad config".to_string());
        assert!(window_err.to_string().contains("bad config"));

        let tab_err = TabError::NavigationFailed("nav error".to_string());
        assert!(tab_err.to_string().contains("nav error"));
    }
}
