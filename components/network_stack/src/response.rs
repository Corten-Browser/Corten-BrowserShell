//! Network response types.

use crate::request::HeaderMap;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// HTTP status code wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatusCode(u16);

impl StatusCode {
    /// Create a new status code.
    pub fn new(code: u16) -> Self {
        Self(code)
    }

    /// Get the raw status code value.
    pub fn as_u16(&self) -> u16 {
        self.0
    }

    /// Check if this is an informational (1xx) status.
    pub fn is_informational(&self) -> bool {
        (100..200).contains(&self.0)
    }

    /// Check if this is a success (2xx) status.
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.0)
    }

    /// Check if this is a redirect (3xx) status.
    pub fn is_redirect(&self) -> bool {
        (300..400).contains(&self.0)
    }

    /// Check if this is a client error (4xx) status.
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.0)
    }

    /// Check if this is a server error (5xx) status.
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.0)
    }

    /// Check if this is an error status (4xx or 5xx).
    pub fn is_error(&self) -> bool {
        self.is_client_error() || self.is_server_error()
    }

    // Common status codes as constants
    pub const OK: StatusCode = StatusCode(200);
    pub const CREATED: StatusCode = StatusCode(201);
    pub const NO_CONTENT: StatusCode = StatusCode(204);
    pub const MOVED_PERMANENTLY: StatusCode = StatusCode(301);
    pub const FOUND: StatusCode = StatusCode(302);
    pub const NOT_MODIFIED: StatusCode = StatusCode(304);
    pub const BAD_REQUEST: StatusCode = StatusCode(400);
    pub const UNAUTHORIZED: StatusCode = StatusCode(401);
    pub const FORBIDDEN: StatusCode = StatusCode(403);
    pub const NOT_FOUND: StatusCode = StatusCode(404);
    pub const INTERNAL_SERVER_ERROR: StatusCode = StatusCode(500);
    pub const BAD_GATEWAY: StatusCode = StatusCode(502);
    pub const SERVICE_UNAVAILABLE: StatusCode = StatusCode(503);
}

impl From<u16> for StatusCode {
    fn from(code: u16) -> Self {
        Self(code)
    }
}

impl From<reqwest::StatusCode> for StatusCode {
    fn from(status: reqwest::StatusCode) -> Self {
        Self(status.as_u16())
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Cache status indicating how the response was served.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CacheStatus {
    /// Response was not cached.
    #[default]
    Miss,
    /// Response was served from cache.
    Hit,
    /// Response was served from cache but revalidated.
    Revalidated,
    /// Response was served from stale cache.
    Stale,
}

impl CacheStatus {
    /// Check if this response was served from cache
    pub fn is_hit(&self) -> bool {
        matches!(self, CacheStatus::Hit | CacheStatus::Revalidated)
    }
}

/// A network response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponse {
    /// HTTP status code.
    pub status: StatusCode,
    /// Response headers.
    pub headers: HeaderMap,
    /// Response body.
    pub body: Vec<u8>,
    /// Time elapsed for the request.
    #[serde(with = "duration_serde")]
    pub elapsed: Duration,
    /// Final URL (after redirects).
    #[serde(with = "url_serde")]
    pub url: Url,
    /// Whether the response came from cache.
    pub cache_status: CacheStatus,
    /// Content type from headers (convenience).
    pub content_type: Option<String>,
    /// Content length from headers (convenience).
    pub content_length: Option<usize>,
}

// Helper modules for serializing Duration and Url
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

mod url_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use url::Url;

    pub fn serialize<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        url.as_str().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Url, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Url::parse(&s).map_err(serde::de::Error::custom)
    }
}

impl NetworkResponse {
    /// Create a new response.
    pub fn new(status: StatusCode, url: Url) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Vec::new(),
            elapsed: Duration::ZERO,
            url,
            cache_status: CacheStatus::Miss,
            content_type: None,
            content_length: None,
        }
    }

    /// Set the response headers.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        // Extract convenience fields
        self.content_type = headers.get("content-type").cloned();
        self.content_length = headers
            .get("content-length")
            .and_then(|v| v.parse().ok());
        self.headers = headers;
        self
    }

    /// Set the response body.
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    /// Set the elapsed time.
    pub fn elapsed(mut self, elapsed: Duration) -> Self {
        self.elapsed = elapsed;
        self
    }

    /// Set the cache status.
    pub fn cache_status(mut self, status: CacheStatus) -> Self {
        self.cache_status = status;
        self
    }

    /// Check if the response indicates success.
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Check if the response indicates an error.
    pub fn is_error(&self) -> bool {
        self.status.is_error()
    }

    /// Get the body as a string (UTF-8).
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    /// Parse the body as JSON.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    /// Get a header value.
    pub fn header(&self, key: &str) -> Option<&String> {
        // Case-insensitive header lookup
        let key_lower = key.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == key_lower)
            .map(|(_, v)| v)
    }

    /// Check if response is cacheable based on cache-control headers.
    pub fn is_cacheable(&self) -> bool {
        if let Some(cache_control) = self.header("cache-control") {
            let cc_lower = cache_control.to_lowercase();
            if cc_lower.contains("no-store") || cc_lower.contains("no-cache") {
                return false;
            }
            if cc_lower.contains("max-age") || cc_lower.contains("public") {
                return true;
            }
        }
        // Check for Expires header
        if self.header("expires").is_some() {
            return true;
        }
        // By default, success responses are cacheable
        self.status.is_success()
    }

    /// Get max-age from cache-control header (in seconds).
    pub fn max_age(&self) -> Option<u64> {
        self.header("cache-control").and_then(|cc| {
            cc.split(',')
                .map(|s| s.trim())
                .find(|s| s.starts_with("max-age="))
                .and_then(|s| s.strip_prefix("max-age="))
                .and_then(|s| s.parse().ok())
        })
    }

    /// Get ETag header value.
    pub fn etag(&self) -> Option<&String> {
        self.header("etag")
    }

    /// Get Last-Modified header value.
    pub fn last_modified(&self) -> Option<&String> {
        self.header("last-modified")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_code_categories() {
        assert!(StatusCode::new(100).is_informational());
        assert!(StatusCode::new(200).is_success());
        assert!(StatusCode::new(301).is_redirect());
        assert!(StatusCode::new(404).is_client_error());
        assert!(StatusCode::new(500).is_server_error());
        assert!(StatusCode::new(404).is_error());
        assert!(StatusCode::new(500).is_error());
    }

    #[test]
    fn test_network_response_builder() {
        let url = Url::parse("https://example.com").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("cache-control".to_string(), "max-age=3600".to_string());

        let response = NetworkResponse::new(StatusCode::OK, url)
            .headers(headers)
            .body(b"{}".to_vec())
            .elapsed(Duration::from_millis(100));

        assert!(response.is_success());
        assert!(!response.is_error());
        assert_eq!(response.content_type, Some("application/json".to_string()));
        assert_eq!(response.elapsed, Duration::from_millis(100));
    }

    #[test]
    fn test_cache_control_parsing() {
        let url = Url::parse("https://example.com").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            "cache-control".to_string(),
            "public, max-age=3600".to_string(),
        );
        headers.insert("etag".to_string(), "\"abc123\"".to_string());

        let response = NetworkResponse::new(StatusCode::OK, url).headers(headers);

        assert!(response.is_cacheable());
        assert_eq!(response.max_age(), Some(3600));
        assert_eq!(response.etag(), Some(&"\"abc123\"".to_string()));
    }

    #[test]
    fn test_no_store_not_cacheable() {
        let url = Url::parse("https://example.com").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("cache-control".to_string(), "no-store".to_string());

        let response = NetworkResponse::new(StatusCode::OK, url).headers(headers);

        assert!(!response.is_cacheable());
    }

    #[test]
    fn test_json_parsing() {
        let url = Url::parse("https://example.com").unwrap();
        let body = serde_json::json!({"key": "value"}).to_string().into_bytes();

        let response = NetworkResponse::new(StatusCode::OK, url).body(body);

        let parsed: serde_json::Value = response.json().unwrap();
        assert_eq!(parsed["key"], "value");
    }
}
