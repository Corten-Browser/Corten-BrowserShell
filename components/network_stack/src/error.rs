//! Network error types for the network stack.

use thiserror::Error;
use url::Url;

/// Network-related errors.
#[derive(Error, Debug)]
pub enum NetworkError {
    /// Request timed out.
    #[error("Request to {url} timed out after {timeout_ms}ms")]
    Timeout {
        /// The URL that timed out.
        url: String,
        /// Timeout duration in milliseconds.
        timeout_ms: u64,
    },

    /// Connection failed.
    #[error("Connection failed to {url}: {reason}")]
    ConnectionFailed {
        /// The URL that failed to connect.
        url: String,
        /// Reason for failure.
        reason: String,
    },

    /// Invalid URL.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// SSL/TLS error.
    #[error("TLS error for {url}: {reason}")]
    TlsError {
        /// The URL with the TLS error.
        url: String,
        /// Reason for failure.
        reason: String,
    },

    /// DNS resolution failed.
    #[error("DNS resolution failed for {host}")]
    DnsError {
        /// The host that failed to resolve.
        host: String,
    },

    /// HTTP error status.
    #[error("HTTP error {status_code} for {url}")]
    HttpError {
        /// The URL that returned an error.
        url: String,
        /// HTTP status code.
        status_code: u16,
    },

    /// Request was cancelled by interceptor.
    #[error("Request cancelled by interceptor: {reason}")]
    RequestCancelled {
        /// Reason for cancellation.
        reason: String,
    },

    /// Response body too large.
    #[error("Response body too large: {size} bytes (max: {max_size})")]
    ResponseTooLarge {
        /// Actual size in bytes.
        size: usize,
        /// Maximum allowed size.
        max_size: usize,
    },

    /// Redirect limit exceeded.
    #[error("Redirect limit exceeded ({count} redirects)")]
    TooManyRedirects {
        /// Number of redirects encountered.
        count: u32,
    },

    /// Cookie error.
    #[error("Cookie error: {0}")]
    CookieError(String),

    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Internal client error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl NetworkError {
    /// Create a timeout error.
    pub fn timeout(url: &Url, timeout_ms: u64) -> Self {
        Self::Timeout {
            url: url.to_string(),
            timeout_ms,
        }
    }

    /// Create a connection failed error.
    pub fn connection_failed(url: &Url, reason: impl Into<String>) -> Self {
        Self::ConnectionFailed {
            url: url.to_string(),
            reason: reason.into(),
        }
    }

    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            NetworkError::Timeout { .. }
                | NetworkError::ConnectionFailed { .. }
                | NetworkError::DnsError { .. }
        )
    }
}

impl From<reqwest::Error> for NetworkError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            return NetworkError::Timeout {
                url: err.url().map(|u| u.to_string()).unwrap_or_default(),
                timeout_ms: 0,
            };
        }

        if err.is_connect() {
            return NetworkError::ConnectionFailed {
                url: err.url().map(|u| u.to_string()).unwrap_or_default(),
                reason: err.to_string(),
            };
        }

        if err.is_status() {
            if let Some(status) = err.status() {
                return NetworkError::HttpError {
                    url: err.url().map(|u| u.to_string()).unwrap_or_default(),
                    status_code: status.as_u16(),
                };
            }
        }

        if err.is_redirect() {
            return NetworkError::TooManyRedirects { count: 10 };
        }

        NetworkError::Internal(err.to_string())
    }
}

/// Result type for network operations.
pub type NetworkResult<T> = Result<T, NetworkError>;
