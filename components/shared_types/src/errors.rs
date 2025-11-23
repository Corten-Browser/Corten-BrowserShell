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

/// Errors that can occur during storage operations
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum StorageError {
    /// Storage initialization failed
    #[error("Storage initialization failed: {0}")]
    InitializationFailed(String),

    /// Database operation failed
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Key not found in storage
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Serialization/deserialization failed
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Migration failed
    #[error("Migration failed: {0}")]
    MigrationError(String),

    /// IO operation failed
    #[error("IO error: {0}")]
    IoError(String),

    /// Connection pool exhausted
    #[error("Connection pool exhausted")]
    PoolExhausted,

    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Invalid data type for key
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },
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
    fn test_storage_error_is_error() {
        let error = StorageError::InitializationFailed("test".to_string());
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

    #[test]
    fn test_storage_error_messages() {
        let init_err = StorageError::InitializationFailed("init failed".to_string());
        assert!(init_err.to_string().contains("init failed"));

        let db_err = StorageError::DatabaseError("db error".to_string());
        assert!(db_err.to_string().contains("db error"));

        let key_err = StorageError::KeyNotFound("my_key".to_string());
        assert!(key_err.to_string().contains("my_key"));

        let ser_err = StorageError::SerializationError("json error".to_string());
        assert!(ser_err.to_string().contains("json error"));

        let mig_err = StorageError::MigrationError("migration v2 failed".to_string());
        assert!(mig_err.to_string().contains("migration v2 failed"));

        let io_err = StorageError::IoError("file not found".to_string());
        assert!(io_err.to_string().contains("file not found"));

        let pool_err = StorageError::PoolExhausted;
        assert!(pool_err.to_string().contains("exhausted"));

        let tx_err = StorageError::TransactionError("commit failed".to_string());
        assert!(tx_err.to_string().contains("commit failed"));

        let type_err = StorageError::TypeMismatch {
            expected: "String".to_string(),
            found: "Integer".to_string(),
        };
        assert!(type_err.to_string().contains("String"));
        assert!(type_err.to_string().contains("Integer"));
    }
}
