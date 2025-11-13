// @implements: REQ-001
//! Error types for browser components
//!
//! This module provides error types used across all browser components for
//! consistent error handling and propagation.

use thiserror::Error;

/// Component-level errors
#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Component not initialized")]
    NotInitialized,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Message handling error: {0}")]
    MessageError(String),

    #[error("Component shutdown error: {0}")]
    ShutdownError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Tab management errors
#[derive(Debug, Error)]
pub enum TabError {
    #[error("Tab not found: {0:?}")]
    NotFound(crate::tab::TabId),

    #[error("Tab creation failed: {0}")]
    CreationFailed(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    #[error("Window not found: {0:?}")]
    WindowNotFound(crate::window::WindowId),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Window management errors
#[derive(Debug, Error)]
pub enum WindowError {
    #[error("Window not found: {0:?}")]
    NotFound(crate::window::WindowId),

    #[error("Window creation failed: {0}")]
    CreationFailed(String),

    #[error("Invalid window configuration: {0}")]
    InvalidConfig(String),

    #[error("Platform error: {0}")]
    PlatformError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_errors_display_correctly() {
        let err = ComponentError::InitializationFailed("test".to_string());
        assert!(err.to_string().contains("Initialization failed"));
    }

    #[test]
    fn tab_errors_display_correctly() {
        let err = TabError::CreationFailed("test".to_string());
        assert!(err.to_string().contains("Tab creation failed"));
    }

    #[test]
    fn window_errors_display_correctly() {
        let err = WindowError::CreationFailed("test".to_string());
        assert!(err.to_string().contains("Window creation failed"));
    }
}
