//! Protocol handler types and traits.

use async_trait::async_trait;
use std::path::PathBuf;
use thiserror::Error;
use url::Url;

use super::ProtocolResponse;

/// Errors that can occur during protocol handling.
#[derive(Error, Debug)]
pub enum ProtocolError {
    /// The protocol scheme is not supported.
    #[error("Unsupported protocol scheme: {scheme}")]
    UnsupportedScheme {
        /// The unsupported scheme.
        scheme: String,
    },

    /// No handler registered for the given URL.
    #[error("No handler registered for URL: {url}")]
    NoHandler {
        /// The URL without a handler.
        url: String,
    },

    /// File not found.
    #[error("File not found: {path}")]
    FileNotFound {
        /// The path that was not found.
        path: PathBuf,
    },

    /// File access denied.
    #[error("Access denied: {path}")]
    AccessDenied {
        /// The path that was denied access.
        path: PathBuf,
    },

    /// Extension not found.
    #[error("Extension not found: {extension_id}")]
    ExtensionNotFound {
        /// The extension ID that was not found.
        extension_id: String,
    },

    /// Extension resource not found.
    #[error("Extension resource not found: {extension_id}/{resource_path}")]
    ExtensionResourceNotFound {
        /// The extension ID.
        extension_id: String,
        /// The resource path within the extension.
        resource_path: String,
    },

    /// Internal page not found.
    #[error("Internal page not found: {page}")]
    InternalPageNotFound {
        /// The internal page name.
        page: String,
    },

    /// Network error during HTTP request.
    #[error("Network error: {0}")]
    NetworkError(#[from] crate::NetworkError),

    /// IO error during file operations.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Invalid URL format.
    #[error("Invalid URL: {reason}")]
    InvalidUrl {
        /// The reason the URL is invalid.
        reason: String,
    },

    /// Security violation.
    #[error("Security violation: {reason}")]
    SecurityViolation {
        /// The reason for the security violation.
        reason: String,
    },
}

impl ProtocolError {
    /// Create a new unsupported scheme error.
    pub fn unsupported_scheme(scheme: impl Into<String>) -> Self {
        Self::UnsupportedScheme {
            scheme: scheme.into(),
        }
    }

    /// Create a new no handler error.
    pub fn no_handler(url: &Url) -> Self {
        Self::NoHandler {
            url: url.to_string(),
        }
    }

    /// Create a new file not found error.
    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Create a new access denied error.
    pub fn access_denied(path: impl Into<PathBuf>) -> Self {
        Self::AccessDenied { path: path.into() }
    }

    /// Create a new extension not found error.
    pub fn extension_not_found(extension_id: impl Into<String>) -> Self {
        Self::ExtensionNotFound {
            extension_id: extension_id.into(),
        }
    }

    /// Create a new extension resource not found error.
    pub fn extension_resource_not_found(
        extension_id: impl Into<String>,
        resource_path: impl Into<String>,
    ) -> Self {
        Self::ExtensionResourceNotFound {
            extension_id: extension_id.into(),
            resource_path: resource_path.into(),
        }
    }

    /// Create a new internal page not found error.
    pub fn internal_page_not_found(page: impl Into<String>) -> Self {
        Self::InternalPageNotFound { page: page.into() }
    }

    /// Create a new invalid URL error.
    pub fn invalid_url(reason: impl Into<String>) -> Self {
        Self::InvalidUrl {
            reason: reason.into(),
        }
    }

    /// Create a new security violation error.
    pub fn security_violation(reason: impl Into<String>) -> Self {
        Self::SecurityViolation {
            reason: reason.into(),
        }
    }

    /// Check if this error indicates the resource was not found.
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::FileNotFound { .. }
                | Self::ExtensionNotFound { .. }
                | Self::ExtensionResourceNotFound { .. }
                | Self::InternalPageNotFound { .. }
                | Self::NoHandler { .. }
        )
    }

    /// Check if this error is a security-related error.
    pub fn is_security_error(&self) -> bool {
        matches!(self, Self::AccessDenied { .. } | Self::SecurityViolation { .. })
    }
}

/// Result type for protocol operations.
pub type ProtocolResult<T> = Result<T, ProtocolError>;

/// Trait for protocol handlers.
///
/// Protocol handlers are responsible for fetching resources for specific URL schemes.
/// Each handler declares which scheme(s) it supports and provides an async method
/// to handle requests.
///
/// # Example
///
/// ```rust,ignore
/// use async_trait::async_trait;
/// use url::Url;
/// use network_stack::protocol::{ProtocolHandler, ProtocolResponse, ProtocolResult};
///
/// struct CustomProtocolHandler;
///
/// #[async_trait]
/// impl ProtocolHandler for CustomProtocolHandler {
///     fn scheme(&self) -> &str {
///         "custom"
///     }
///
///     fn can_handle(&self, url: &Url) -> bool {
///         url.scheme() == "custom"
///     }
///
///     async fn handle(&self, url: &Url) -> ProtocolResult<ProtocolResponse> {
///         // Handle custom:// URLs
///         Ok(ProtocolResponse::text("Custom response"))
///     }
/// }
/// ```
#[async_trait]
pub trait ProtocolHandler: Send + Sync {
    /// Returns the primary scheme this handler supports (e.g., "http", "file").
    fn scheme(&self) -> &str;

    /// Returns all schemes this handler supports.
    ///
    /// Default implementation returns a single scheme from `scheme()`.
    /// Override this for handlers that support multiple schemes (e.g., http and https).
    fn schemes(&self) -> Vec<&str> {
        vec![self.scheme()]
    }

    /// Check if this handler can handle the given URL.
    ///
    /// Default implementation checks if the URL scheme matches any supported scheme.
    fn can_handle(&self, url: &Url) -> bool {
        self.schemes().contains(&url.scheme())
    }

    /// Handle a request for the given URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch.
    ///
    /// # Returns
    ///
    /// A `ProtocolResponse` containing the fetched resource, or a `ProtocolError`.
    async fn handle(&self, url: &Url) -> ProtocolResult<ProtocolResponse>;

    /// Returns a human-readable name for this handler.
    fn name(&self) -> &str {
        self.scheme()
    }

    /// Check if this handler supports streaming responses.
    fn supports_streaming(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_error_unsupported_scheme() {
        let error = ProtocolError::unsupported_scheme("ftp");
        assert!(matches!(&error, ProtocolError::UnsupportedScheme { scheme } if scheme == "ftp"));
        assert!(error.to_string().contains("ftp"));
    }

    #[test]
    fn test_protocol_error_no_handler() {
        let url = Url::parse("custom://example").unwrap();
        let error = ProtocolError::no_handler(&url);
        assert!(error.is_not_found());
        assert!(error.to_string().contains("custom://example"));
    }

    #[test]
    fn test_protocol_error_file_not_found() {
        let error = ProtocolError::file_not_found("/path/to/file");
        assert!(error.is_not_found());
        assert!(error.to_string().contains("/path/to/file"));
    }

    #[test]
    fn test_protocol_error_access_denied() {
        let error = ProtocolError::access_denied("/path/to/file");
        assert!(error.is_security_error());
        assert!(error.to_string().contains("/path/to/file"));
    }

    #[test]
    fn test_protocol_error_extension_not_found() {
        let error = ProtocolError::extension_not_found("my-extension");
        assert!(error.is_not_found());
        assert!(error.to_string().contains("my-extension"));
    }

    #[test]
    fn test_protocol_error_extension_resource_not_found() {
        let error = ProtocolError::extension_resource_not_found("my-extension", "icon.png");
        assert!(error.is_not_found());
        assert!(error.to_string().contains("my-extension"));
        assert!(error.to_string().contains("icon.png"));
    }

    #[test]
    fn test_protocol_error_internal_page_not_found() {
        let error = ProtocolError::internal_page_not_found("settings");
        assert!(error.is_not_found());
        assert!(error.to_string().contains("settings"));
    }

    #[test]
    fn test_protocol_error_invalid_url() {
        let error = ProtocolError::invalid_url("missing host");
        assert!(!error.is_not_found());
        assert!(error.to_string().contains("missing host"));
    }

    #[test]
    fn test_protocol_error_security_violation() {
        let error = ProtocolError::security_violation("path traversal");
        assert!(error.is_security_error());
        assert!(error.to_string().contains("path traversal"));
    }
}
