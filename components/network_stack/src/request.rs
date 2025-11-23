//! Network request types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

/// HTTP methods supported by the network stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    /// GET method.
    Get,
    /// POST method.
    Post,
    /// PUT method.
    Put,
    /// DELETE method.
    Delete,
    /// HEAD method.
    Head,
    /// OPTIONS method.
    Options,
    /// PATCH method.
    Patch,
    /// CONNECT method.
    Connect,
    /// TRACE method.
    Trace,
}

impl Default for Method {
    fn default() -> Self {
        Self::Get
    }
}

impl From<Method> for reqwest::Method {
    fn from(method: Method) -> Self {
        match method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Head => reqwest::Method::HEAD,
            Method::Options => reqwest::Method::OPTIONS,
            Method::Patch => reqwest::Method::PATCH,
            Method::Connect => reqwest::Method::CONNECT,
            Method::Trace => reqwest::Method::TRACE,
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
            Method::Head => write!(f, "HEAD"),
            Method::Options => write!(f, "OPTIONS"),
            Method::Patch => write!(f, "PATCH"),
            Method::Connect => write!(f, "CONNECT"),
            Method::Trace => write!(f, "TRACE"),
        }
    }
}

/// HTTP header map type alias.
pub type HeaderMap = HashMap<String, String>;

/// Resource type hint for the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// HTML document.
    Document,
    /// Stylesheet (CSS).
    Stylesheet,
    /// JavaScript.
    Script,
    /// Image (PNG, JPEG, GIF, etc.).
    Image,
    /// Font file.
    Font,
    /// XMLHttpRequest/Fetch.
    Xhr,
    /// Media (audio/video).
    Media,
    /// WebSocket.
    WebSocket,
    /// Other/unknown.
    Other,
}

impl Default for ResourceType {
    fn default() -> Self {
        Self::Other
    }
}

/// Cache mode for requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CacheMode {
    /// Use standard HTTP cache semantics.
    #[default]
    Default,
    /// Bypass cache, always fetch from network.
    NoStore,
    /// Only use cached response, fail if not cached.
    OnlyIfCached,
    /// Revalidate cached response with server.
    Reload,
    /// Force cache, only revalidate if no cache entry.
    ForceCache,
}

/// Credentials mode for cross-origin requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CredentialsMode {
    /// Never send credentials.
    Omit,
    /// Send credentials for same-origin requests only.
    #[default]
    SameOrigin,
    /// Always send credentials.
    Include,
}

/// Redirect policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RedirectPolicy {
    /// Follow redirects automatically (with limit).
    Follow {
        /// Maximum number of redirects to follow.
        max_redirects: u32,
    },
    /// Error on redirect.
    Error,
    /// Return redirect response without following.
    Manual,
}

impl Default for RedirectPolicy {
    fn default() -> Self {
        Self::Follow { max_redirects: 10 }
    }
}

/// A network request.
#[derive(Debug, Clone)]
pub struct NetworkRequest {
    /// Request URL.
    pub url: Url,
    /// HTTP method.
    pub method: Method,
    /// Request headers.
    pub headers: HeaderMap,
    /// Request body (if any).
    pub body: Option<Vec<u8>>,
    /// Request timeout.
    pub timeout: Duration,
    /// Resource type hint.
    pub resource_type: ResourceType,
    /// Cache mode.
    pub cache_mode: CacheMode,
    /// Credentials mode.
    pub credentials_mode: CredentialsMode,
    /// Redirect policy.
    pub redirect_policy: RedirectPolicy,
    /// Priority (0-255, higher = more important).
    pub priority: u8,
    /// Custom metadata attached to the request.
    pub metadata: HashMap<String, String>,
}

impl NetworkRequest {
    /// Default timeout duration.
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

    /// Create a new GET request.
    pub fn get(url: Url) -> Self {
        Self::new(Method::Get, url)
    }

    /// Create a new POST request.
    pub fn post(url: Url) -> Self {
        Self::new(Method::Post, url)
    }

    /// Create a new request with the given method and URL.
    pub fn new(method: Method, url: Url) -> Self {
        Self {
            url,
            method,
            headers: HeaderMap::new(),
            body: None,
            timeout: Self::DEFAULT_TIMEOUT,
            resource_type: ResourceType::Other,
            cache_mode: CacheMode::Default,
            credentials_mode: CredentialsMode::SameOrigin,
            redirect_policy: RedirectPolicy::default(),
            priority: 128,
            metadata: HashMap::new(),
        }
    }

    /// Set a header value.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set multiple headers.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers.extend(headers);
        self
    }

    /// Set the request body.
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Set the request body as JSON.
    pub fn json<T: serde::Serialize>(mut self, value: &T) -> Result<Self, serde_json::Error> {
        let body = serde_json::to_vec(value)?;
        self.body = Some(body);
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        Ok(self)
    }

    /// Set the timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the resource type.
    pub fn resource_type(mut self, resource_type: ResourceType) -> Self {
        self.resource_type = resource_type;
        self
    }

    /// Set the cache mode.
    pub fn cache_mode(mut self, cache_mode: CacheMode) -> Self {
        self.cache_mode = cache_mode;
        self
    }

    /// Set the credentials mode.
    pub fn credentials_mode(mut self, credentials_mode: CredentialsMode) -> Self {
        self.credentials_mode = credentials_mode;
        self
    }

    /// Set the redirect policy.
    pub fn redirect_policy(mut self, redirect_policy: RedirectPolicy) -> Self {
        self.redirect_policy = redirect_policy;
        self
    }

    /// Set the priority.
    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Add custom metadata.
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get the host from the URL.
    pub fn host(&self) -> Option<&str> {
        self.url.host_str()
    }

    /// Check if this is a secure (HTTPS) request.
    pub fn is_secure(&self) -> bool {
        self.url.scheme() == "https"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_request_get() {
        let url = Url::parse("https://example.com/path").unwrap();
        let request = NetworkRequest::get(url.clone());

        assert_eq!(request.method, Method::Get);
        assert_eq!(request.url, url);
        assert!(request.body.is_none());
    }

    #[test]
    fn test_network_request_post_with_json() {
        let url = Url::parse("https://api.example.com/data").unwrap();
        let data = serde_json::json!({"key": "value"});
        let request = NetworkRequest::post(url)
            .json(&data)
            .unwrap();

        assert_eq!(request.method, Method::Post);
        assert!(request.body.is_some());
        assert_eq!(
            request.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_network_request_builder() {
        let url = Url::parse("https://example.com").unwrap();
        let request = NetworkRequest::get(url)
            .header("Accept", "application/json")
            .timeout(Duration::from_secs(10))
            .resource_type(ResourceType::Xhr)
            .cache_mode(CacheMode::NoStore)
            .priority(255);

        assert_eq!(
            request.headers.get("Accept"),
            Some(&"application/json".to_string())
        );
        assert_eq!(request.timeout, Duration::from_secs(10));
        assert_eq!(request.resource_type, ResourceType::Xhr);
        assert_eq!(request.cache_mode, CacheMode::NoStore);
        assert_eq!(request.priority, 255);
    }

    #[test]
    fn test_is_secure() {
        let https_url = Url::parse("https://example.com").unwrap();
        let http_url = Url::parse("http://example.com").unwrap();

        assert!(NetworkRequest::get(https_url).is_secure());
        assert!(!NetworkRequest::get(http_url).is_secure());
    }
}
