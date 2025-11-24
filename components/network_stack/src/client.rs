//! Network client implementation.
//!
//! This module provides the main [`NetworkClient`] trait and its implementation
//! using reqwest for HTTP requests.

use crate::error::{NetworkError, NetworkResult};
use crate::interceptor::{
    InterceptorOutcome, RequestInterceptor, RequestInterceptorChain, ResponseInterceptor,
    ResponseInterceptorChain,
};
use crate::request::NetworkRequest;
use crate::response::{CacheStatus, NetworkResponse, StatusCode};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use url::Url;

/// Configuration for the network client.
#[derive(Debug, Clone)]
pub struct NetworkClientConfig {
    /// Default timeout for requests.
    pub default_timeout: Duration,
    /// Maximum number of redirects to follow.
    pub max_redirects: u32,
    /// Whether to accept invalid certificates (for development).
    pub accept_invalid_certs: bool,
    /// Connection pool idle timeout.
    pub pool_idle_timeout: Duration,
    /// Maximum idle connections per host.
    pub pool_max_idle_per_host: usize,
    /// User agent string.
    pub user_agent: String,
    /// Enable HTTP/2.
    pub http2_enabled: bool,
    /// Enable gzip decompression.
    pub gzip_enabled: bool,
    /// Enable brotli decompression.
    pub brotli_enabled: bool,
    /// Maximum response body size in bytes.
    pub max_response_size: usize,
}

impl Default for NetworkClientConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_redirects: 10,
            accept_invalid_certs: false,
            pool_idle_timeout: Duration::from_secs(90),
            pool_max_idle_per_host: 10,
            user_agent: format!("CortenBrowser/{}", env!("CARGO_PKG_VERSION")),
            http2_enabled: true,
            gzip_enabled: true,
            brotli_enabled: true,
            max_response_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

/// A network client for making HTTP requests.
#[async_trait]
pub trait NetworkClient: Send + Sync {
    /// Fetch a resource.
    async fn fetch(&self, request: NetworkRequest) -> NetworkResult<NetworkResponse>;

    /// Add a request interceptor.
    async fn add_request_interceptor(&self, interceptor: Arc<dyn RequestInterceptor>);

    /// Add a response interceptor.
    async fn add_response_interceptor(&self, interceptor: Arc<dyn ResponseInterceptor>);

    /// Get the current configuration.
    fn config(&self) -> &NetworkClientConfig;
}

/// HTTP client implementation using reqwest.
pub struct HttpClient {
    inner: reqwest::Client,
    config: NetworkClientConfig,
    request_interceptors: RwLock<RequestInterceptorChain>,
    response_interceptors: RwLock<ResponseInterceptorChain>,
}

impl std::fmt::Debug for HttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClient")
            .field("config", &self.config)
            .finish()
    }
}

impl HttpClient {
    /// Create a new HTTP client with default configuration.
    pub fn new() -> NetworkResult<Self> {
        Self::with_config(NetworkClientConfig::default())
    }

    /// Create a new HTTP client with custom configuration.
    pub fn with_config(config: NetworkClientConfig) -> NetworkResult<Self> {
        let mut builder = reqwest::Client::builder()
            .timeout(config.default_timeout)
            .pool_idle_timeout(config.pool_idle_timeout)
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .user_agent(&config.user_agent)
            .redirect(reqwest::redirect::Policy::limited(
                config.max_redirects as usize,
            ))
            .danger_accept_invalid_certs(config.accept_invalid_certs);

        if config.gzip_enabled {
            builder = builder.gzip(true);
        }

        if config.brotli_enabled {
            builder = builder.brotli(true);
        }

        let inner = builder
            .build()
            .map_err(|e| NetworkError::Internal(e.to_string()))?;

        Ok(Self {
            inner,
            config,
            request_interceptors: RwLock::new(RequestInterceptorChain::new()),
            response_interceptors: RwLock::new(ResponseInterceptorChain::new()),
        })
    }

    /// Execute the actual HTTP request.
    async fn execute_request(&self, request: &NetworkRequest) -> NetworkResult<NetworkResponse> {
        let start = Instant::now();

        // Build reqwest request
        let mut req_builder = self
            .inner
            .request(request.method.into(), request.url.clone())
            .timeout(request.timeout);

        // Add headers
        for (key, value) in &request.headers {
            req_builder = req_builder.header(key, value);
        }

        // Add body
        if let Some(body) = &request.body {
            req_builder = req_builder.body(body.clone());
        }

        // Note: Redirect policy is handled at the client level in reqwest.
        // Per-request redirect policy would require creating a new client,
        // which is expensive. For now, we use the client-level setting.
        // If specific redirect handling is needed, the response will include
        // redirect information that can be handled by the caller.
        #[allow(clippy::let_underscore_untyped)]
        let _ = &request.redirect_policy;

        // Execute request
        let response = req_builder.send().await?;

        let elapsed = start.elapsed();
        let status = StatusCode::from(response.status());
        let final_url = response.url().clone();

        // Convert headers
        let mut headers = crate::request::HeaderMap::new();
        for (key, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.to_string(), v.to_string());
            }
        }

        // Check response size before reading body
        if let Some(content_length) = response.content_length() {
            if content_length as usize > self.config.max_response_size {
                return Err(NetworkError::ResponseTooLarge {
                    size: content_length as usize,
                    max_size: self.config.max_response_size,
                });
            }
        }

        // Read body
        let body = response.bytes().await?;

        if body.len() > self.config.max_response_size {
            return Err(NetworkError::ResponseTooLarge {
                size: body.len(),
                max_size: self.config.max_response_size,
            });
        }

        Ok(NetworkResponse::new(status, final_url)
            .headers(headers)
            .body(body.to_vec())
            .elapsed(elapsed)
            .cache_status(CacheStatus::Miss))
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HttpClient")
    }
}

#[async_trait]
impl NetworkClient for HttpClient {
    async fn fetch(&self, request: NetworkRequest) -> NetworkResult<NetworkResponse> {
        // Run request interceptors
        let interceptors = self.request_interceptors.read().await;
        let request = match interceptors.intercept(request).await? {
            InterceptorOutcome::Continue(req) => req,
            InterceptorOutcome::ShortCircuit(response) => return Ok(response),
            InterceptorOutcome::Cancel(reason) => {
                return Err(NetworkError::RequestCancelled { reason })
            }
        };
        drop(interceptors);

        // Execute the actual request
        let response = self.execute_request(&request).await?;

        // Run response interceptors
        let interceptors = self.response_interceptors.read().await;
        interceptors.intercept(&request, response).await
    }

    async fn add_request_interceptor(&self, interceptor: Arc<dyn RequestInterceptor>) {
        let mut chain = self.request_interceptors.write().await;
        chain.add(interceptor);
    }

    async fn add_response_interceptor(&self, interceptor: Arc<dyn ResponseInterceptor>) {
        let mut chain = self.response_interceptors.write().await;
        chain.add(interceptor);
    }

    fn config(&self) -> &NetworkClientConfig {
        &self.config
    }
}

/// Builder for creating an HttpClient with custom configuration.
#[derive(Debug, Clone, Default)]
pub struct HttpClientBuilder {
    config: NetworkClientConfig,
    request_interceptors: Vec<Arc<dyn RequestInterceptor>>,
    response_interceptors: Vec<Arc<dyn ResponseInterceptor>>,
}

impl HttpClientBuilder {
    /// Create a new builder with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the default timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.default_timeout = timeout;
        self
    }

    /// Set the maximum number of redirects.
    pub fn max_redirects(mut self, max: u32) -> Self {
        self.config.max_redirects = max;
        self
    }

    /// Accept invalid SSL certificates (for development only).
    pub fn accept_invalid_certs(mut self, accept: bool) -> Self {
        self.config.accept_invalid_certs = accept;
        self
    }

    /// Set the connection pool idle timeout.
    pub fn pool_idle_timeout(mut self, timeout: Duration) -> Self {
        self.config.pool_idle_timeout = timeout;
        self
    }

    /// Set the maximum idle connections per host.
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.config.pool_max_idle_per_host = max;
        self
    }

    /// Set the User-Agent string.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    /// Enable or disable HTTP/2.
    pub fn http2(mut self, enabled: bool) -> Self {
        self.config.http2_enabled = enabled;
        self
    }

    /// Enable or disable gzip decompression.
    pub fn gzip(mut self, enabled: bool) -> Self {
        self.config.gzip_enabled = enabled;
        self
    }

    /// Enable or disable brotli decompression.
    pub fn brotli(mut self, enabled: bool) -> Self {
        self.config.brotli_enabled = enabled;
        self
    }

    /// Set the maximum response body size.
    pub fn max_response_size(mut self, size: usize) -> Self {
        self.config.max_response_size = size;
        self
    }

    /// Add a request interceptor.
    pub fn request_interceptor(mut self, interceptor: Arc<dyn RequestInterceptor>) -> Self {
        self.request_interceptors.push(interceptor);
        self
    }

    /// Add a response interceptor.
    pub fn response_interceptor(mut self, interceptor: Arc<dyn ResponseInterceptor>) -> Self {
        self.response_interceptors.push(interceptor);
        self
    }

    /// Build the HTTP client.
    pub fn build(self) -> NetworkResult<HttpClient> {
        let client = HttpClient::with_config(self.config)?;

        // Add interceptors synchronously during build
        let mut req_chain = RequestInterceptorChain::new();
        for interceptor in self.request_interceptors {
            req_chain.add(interceptor);
        }

        let mut resp_chain = ResponseInterceptorChain::new();
        for interceptor in self.response_interceptors {
            resp_chain.add(interceptor);
        }

        // We need to replace the chains, which requires some unsafe dance
        // Since we just created the client, this is safe
        *client.request_interceptors.try_write().unwrap() = req_chain;
        *client.response_interceptors.try_write().unwrap() = resp_chain;

        Ok(client)
    }
}

/// Cookie management interface.
#[async_trait]
pub trait CookieStore: Send + Sync {
    /// Get cookies for a URL.
    async fn get_cookies(&self, url: &Url) -> Vec<Cookie>;

    /// Set a cookie for a URL.
    async fn set_cookie(&self, cookie: Cookie, url: &Url);

    /// Remove a cookie.
    async fn remove_cookie(&self, name: &str, url: &Url);

    /// Clear all cookies.
    async fn clear(&self);
}

/// A cookie representation.
#[derive(Debug, Clone)]
pub struct Cookie {
    /// Cookie name.
    pub name: String,
    /// Cookie value.
    pub value: String,
    /// Domain the cookie applies to.
    pub domain: Option<String>,
    /// Path the cookie applies to.
    pub path: Option<String>,
    /// Expiration time (Unix timestamp).
    pub expires: Option<i64>,
    /// Whether the cookie is secure-only.
    pub secure: bool,
    /// Whether the cookie is HTTP-only.
    pub http_only: bool,
    /// SameSite attribute.
    pub same_site: SameSite,
}

/// SameSite cookie attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SameSite {
    /// Cookie is sent in all contexts.
    #[default]
    None,
    /// Cookie is sent for same-site and top-level navigation.
    Lax,
    /// Cookie is only sent for same-site requests.
    Strict,
}

impl Cookie {
    /// Create a new session cookie.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            domain: None,
            path: None,
            expires: None,
            secure: false,
            http_only: false,
            same_site: SameSite::default(),
        }
    }

    /// Set the domain.
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Set the path.
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the expiration time.
    pub fn expires(mut self, timestamp: i64) -> Self {
        self.expires = Some(timestamp);
        self
    }

    /// Mark as secure-only.
    pub fn secure(mut self) -> Self {
        self.secure = true;
        self
    }

    /// Mark as HTTP-only.
    pub fn http_only(mut self) -> Self {
        self.http_only = true;
        self
    }

    /// Set the SameSite attribute.
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = same_site;
        self
    }

    /// Check if the cookie is expired.
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires {
            let now = chrono::Utc::now().timestamp();
            expires < now
        } else {
            false // Session cookies don't expire
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = NetworkClientConfig::default();
        assert_eq!(config.default_timeout, Duration::from_secs(30));
        assert_eq!(config.max_redirects, 10);
        assert!(!config.accept_invalid_certs);
        assert!(config.gzip_enabled);
        assert!(config.brotli_enabled);
    }

    #[test]
    fn test_http_client_builder() {
        let client = HttpClientBuilder::new()
            .timeout(Duration::from_secs(60))
            .max_redirects(5)
            .user_agent("TestAgent/1.0")
            .gzip(false)
            .build()
            .unwrap();

        assert_eq!(client.config().default_timeout, Duration::from_secs(60));
        assert_eq!(client.config().max_redirects, 5);
        assert_eq!(client.config().user_agent, "TestAgent/1.0");
        assert!(!client.config().gzip_enabled);
    }

    #[test]
    fn test_cookie_builder() {
        let cookie = Cookie::new("session", "abc123")
            .domain("example.com")
            .path("/app")
            .secure()
            .http_only()
            .same_site(SameSite::Strict);

        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.domain, Some("example.com".to_string()));
        assert_eq!(cookie.path, Some("/app".to_string()));
        assert!(cookie.secure);
        assert!(cookie.http_only);
        assert_eq!(cookie.same_site, SameSite::Strict);
    }

    #[test]
    fn test_cookie_expiration() {
        // Not expired (1 year from now)
        let future_cookie = Cookie::new("test", "value")
            .expires(chrono::Utc::now().timestamp() + 31536000);
        assert!(!future_cookie.is_expired());

        // Expired (1 year ago)
        let past_cookie = Cookie::new("test", "value")
            .expires(chrono::Utc::now().timestamp() - 31536000);
        assert!(past_cookie.is_expired());

        // Session cookie (never expires)
        let session_cookie = Cookie::new("test", "value");
        assert!(!session_cookie.is_expired());
    }
}
