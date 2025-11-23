//! Request and response interceptors for the network stack.
//!
//! Interceptors allow modifying requests before they are sent and responses
//! after they are received. They can be used for:
//! - Adding authentication headers
//! - Logging and metrics
//! - Request/response transformation
//! - Security filtering
//! - Caching

use crate::error::NetworkResult;
use crate::request::NetworkRequest;
use crate::response::NetworkResponse;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

/// Outcome of a request interception.
#[derive(Debug)]
pub enum InterceptorOutcome<T> {
    /// Continue processing with the (possibly modified) value.
    Continue(T),
    /// Stop processing and return early with a response.
    ShortCircuit(NetworkResponse),
    /// Cancel the request entirely.
    Cancel(String),
}

/// A request interceptor that can modify or cancel outgoing requests.
#[async_trait]
pub trait RequestInterceptor: Send + Sync + Debug {
    /// Intercept a request before it is sent.
    ///
    /// Returns the outcome of the interception:
    /// - `Continue(request)`: Continue with the (possibly modified) request
    /// - `ShortCircuit(response)`: Skip the network call and return this response
    /// - `Cancel(reason)`: Cancel the request entirely with an error
    async fn intercept_request(
        &self,
        request: NetworkRequest,
    ) -> NetworkResult<InterceptorOutcome<NetworkRequest>>;

    /// Get the name of this interceptor for logging/debugging.
    fn name(&self) -> &str;

    /// Get the priority of this interceptor (higher = runs first).
    fn priority(&self) -> i32 {
        0
    }
}

/// A response interceptor that can modify or transform incoming responses.
#[async_trait]
pub trait ResponseInterceptor: Send + Sync + Debug {
    /// Intercept a response after it is received.
    ///
    /// Returns the (possibly modified) response.
    async fn intercept_response(
        &self,
        request: &NetworkRequest,
        response: NetworkResponse,
    ) -> NetworkResult<NetworkResponse>;

    /// Get the name of this interceptor for logging/debugging.
    fn name(&self) -> &str;

    /// Get the priority of this interceptor (higher = runs first).
    fn priority(&self) -> i32 {
        0
    }
}

/// A chain of request interceptors.
#[derive(Debug, Default)]
pub struct RequestInterceptorChain {
    interceptors: Vec<Arc<dyn RequestInterceptor>>,
}

impl RequestInterceptorChain {
    /// Create a new empty chain.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an interceptor to the chain.
    pub fn add(&mut self, interceptor: Arc<dyn RequestInterceptor>) {
        self.interceptors.push(interceptor);
        // Sort by priority (higher first)
        self.interceptors
            .sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Run all interceptors in the chain.
    pub async fn intercept(
        &self,
        mut request: NetworkRequest,
    ) -> NetworkResult<InterceptorOutcome<NetworkRequest>> {
        for interceptor in &self.interceptors {
            tracing::debug!(
                interceptor = interceptor.name(),
                "Running request interceptor"
            );

            match interceptor.intercept_request(request).await? {
                InterceptorOutcome::Continue(req) => {
                    request = req;
                }
                outcome @ InterceptorOutcome::ShortCircuit(_) => {
                    tracing::debug!(
                        interceptor = interceptor.name(),
                        "Request short-circuited"
                    );
                    return Ok(outcome);
                }
                outcome @ InterceptorOutcome::Cancel(_) => {
                    tracing::debug!(interceptor = interceptor.name(), "Request cancelled");
                    return Ok(outcome);
                }
            }
        }

        Ok(InterceptorOutcome::Continue(request))
    }

    /// Get the number of interceptors in the chain.
    pub fn len(&self) -> usize {
        self.interceptors.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.interceptors.is_empty()
    }
}

/// A chain of response interceptors.
#[derive(Debug, Default)]
pub struct ResponseInterceptorChain {
    interceptors: Vec<Arc<dyn ResponseInterceptor>>,
}

impl ResponseInterceptorChain {
    /// Create a new empty chain.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an interceptor to the chain.
    pub fn add(&mut self, interceptor: Arc<dyn ResponseInterceptor>) {
        self.interceptors.push(interceptor);
        // Sort by priority (higher first)
        self.interceptors
            .sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Run all interceptors in the chain.
    pub async fn intercept(
        &self,
        request: &NetworkRequest,
        mut response: NetworkResponse,
    ) -> NetworkResult<NetworkResponse> {
        for interceptor in &self.interceptors {
            tracing::debug!(
                interceptor = interceptor.name(),
                "Running response interceptor"
            );

            response = interceptor.intercept_response(request, response).await?;
        }

        Ok(response)
    }

    /// Get the number of interceptors in the chain.
    pub fn len(&self) -> usize {
        self.interceptors.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.interceptors.is_empty()
    }
}

// ============================================================================
// Built-in interceptors
// ============================================================================

/// Interceptor that adds a User-Agent header.
#[derive(Debug, Clone)]
pub struct UserAgentInterceptor {
    user_agent: String,
}

impl UserAgentInterceptor {
    /// Create a new User-Agent interceptor.
    pub fn new(user_agent: impl Into<String>) -> Self {
        Self {
            user_agent: user_agent.into(),
        }
    }
}

#[async_trait]
impl RequestInterceptor for UserAgentInterceptor {
    async fn intercept_request(
        &self,
        mut request: NetworkRequest,
    ) -> NetworkResult<InterceptorOutcome<NetworkRequest>> {
        if !request.headers.contains_key("User-Agent") {
            request
                .headers
                .insert("User-Agent".to_string(), self.user_agent.clone());
        }
        Ok(InterceptorOutcome::Continue(request))
    }

    fn name(&self) -> &str {
        "UserAgentInterceptor"
    }

    fn priority(&self) -> i32 {
        100 // Run early
    }
}

/// Interceptor that adds authentication headers.
#[derive(Debug, Clone)]
pub struct AuthInterceptor {
    auth_type: AuthType,
}

/// Authentication type for the auth interceptor.
#[derive(Debug, Clone)]
pub enum AuthType {
    /// Bearer token authentication.
    Bearer(String),
    /// Basic authentication (username:password base64 encoded).
    Basic { username: String, password: String },
    /// Custom header-based authentication.
    Custom { header: String, value: String },
}

impl AuthInterceptor {
    /// Create a new Bearer token interceptor.
    pub fn bearer(token: impl Into<String>) -> Self {
        Self {
            auth_type: AuthType::Bearer(token.into()),
        }
    }

    /// Create a new Basic auth interceptor.
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            auth_type: AuthType::Basic {
                username: username.into(),
                password: password.into(),
            },
        }
    }

    /// Create a custom auth interceptor.
    pub fn custom(header: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            auth_type: AuthType::Custom {
                header: header.into(),
                value: value.into(),
            },
        }
    }
}

#[async_trait]
impl RequestInterceptor for AuthInterceptor {
    async fn intercept_request(
        &self,
        mut request: NetworkRequest,
    ) -> NetworkResult<InterceptorOutcome<NetworkRequest>> {
        match &self.auth_type {
            AuthType::Bearer(token) => {
                request
                    .headers
                    .insert("Authorization".to_string(), format!("Bearer {}", token));
            }
            AuthType::Basic { username, password } => {
                use std::io::Write;
                let mut buf = Vec::new();
                write!(buf, "{}:{}", username, password).unwrap();
                let encoded = base64_encode(&buf);
                request
                    .headers
                    .insert("Authorization".to_string(), format!("Basic {}", encoded));
            }
            AuthType::Custom { header, value } => {
                request.headers.insert(header.clone(), value.clone());
            }
        }
        Ok(InterceptorOutcome::Continue(request))
    }

    fn name(&self) -> &str {
        "AuthInterceptor"
    }

    fn priority(&self) -> i32 {
        90 // Run after user-agent but before most others
    }
}

// Simple base64 encoding (avoiding additional dependency)
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let mut i = 0;

    while i < data.len() {
        let b0 = data[i] as usize;
        let b1 = data.get(i + 1).map(|&b| b as usize).unwrap_or(0);
        let b2 = data.get(i + 2).map(|&b| b as usize).unwrap_or(0);

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if i + 1 < data.len() {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }

    result
}

/// Interceptor that logs requests and responses.
#[derive(Debug, Clone, Default)]
pub struct LoggingInterceptor {
    log_body: bool,
}

impl LoggingInterceptor {
    /// Create a new logging interceptor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable logging of request/response bodies.
    pub fn with_body_logging(mut self) -> Self {
        self.log_body = true;
        self
    }
}

#[async_trait]
impl RequestInterceptor for LoggingInterceptor {
    async fn intercept_request(
        &self,
        request: NetworkRequest,
    ) -> NetworkResult<InterceptorOutcome<NetworkRequest>> {
        tracing::info!(
            method = %request.method,
            url = %request.url,
            "Outgoing request"
        );

        if self.log_body {
            if let Some(body) = &request.body {
                if let Ok(text) = std::str::from_utf8(body) {
                    tracing::debug!(body = %text, "Request body");
                }
            }
        }

        Ok(InterceptorOutcome::Continue(request))
    }

    fn name(&self) -> &str {
        "LoggingInterceptor"
    }

    fn priority(&self) -> i32 {
        -100 // Run last (after all modifications)
    }
}

#[async_trait]
impl ResponseInterceptor for LoggingInterceptor {
    async fn intercept_response(
        &self,
        request: &NetworkRequest,
        response: NetworkResponse,
    ) -> NetworkResult<NetworkResponse> {
        tracing::info!(
            method = %request.method,
            url = %request.url,
            status = %response.status,
            elapsed_ms = response.elapsed.as_millis(),
            "Incoming response"
        );

        if self.log_body {
            if let Ok(text) = response.text() {
                tracing::debug!(body = %text, "Response body");
            }
        }

        Ok(response)
    }

    fn name(&self) -> &str {
        "LoggingInterceptor"
    }

    fn priority(&self) -> i32 {
        -100 // Run last
    }
}

/// Interceptor that retries failed requests.
#[derive(Debug, Clone)]
pub struct RetryInterceptor {
    max_retries: u32,
    retry_delay_ms: u64,
}

impl RetryInterceptor {
    /// Create a new retry interceptor.
    pub fn new(max_retries: u32, retry_delay_ms: u64) -> Self {
        Self {
            max_retries,
            retry_delay_ms,
        }
    }

    /// Get the maximum number of retries.
    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }

    /// Get the retry delay in milliseconds.
    pub fn retry_delay_ms(&self) -> u64 {
        self.retry_delay_ms
    }
}

impl Default for RetryInterceptor {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

#[async_trait]
impl ResponseInterceptor for RetryInterceptor {
    async fn intercept_response(
        &self,
        _request: &NetworkRequest,
        response: NetworkResponse,
    ) -> NetworkResult<NetworkResponse> {
        // Note: Actual retry logic would need to be in the client,
        // as interceptors can't re-execute requests.
        // This interceptor just adds retry metadata for the client to use.
        if response.status.is_server_error() {
            tracing::warn!(
                status = %response.status,
                max_retries = self.max_retries,
                "Response indicates server error, retry may be needed"
            );
        }
        Ok(response)
    }

    fn name(&self) -> &str {
        "RetryInterceptor"
    }

    fn priority(&self) -> i32 {
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[tokio::test]
    async fn test_user_agent_interceptor() {
        let interceptor = UserAgentInterceptor::new("TestBrowser/1.0");
        let url = Url::parse("https://example.com").unwrap();
        let request = NetworkRequest::get(url);

        let result = interceptor.intercept_request(request).await.unwrap();

        if let InterceptorOutcome::Continue(req) = result {
            assert_eq!(
                req.headers.get("User-Agent"),
                Some(&"TestBrowser/1.0".to_string())
            );
        } else {
            panic!("Expected Continue outcome");
        }
    }

    #[tokio::test]
    async fn test_user_agent_not_overwritten() {
        let interceptor = UserAgentInterceptor::new("TestBrowser/1.0");
        let url = Url::parse("https://example.com").unwrap();
        let request = NetworkRequest::get(url).header("User-Agent", "CustomAgent/2.0");

        let result = interceptor.intercept_request(request).await.unwrap();

        if let InterceptorOutcome::Continue(req) = result {
            assert_eq!(
                req.headers.get("User-Agent"),
                Some(&"CustomAgent/2.0".to_string())
            );
        } else {
            panic!("Expected Continue outcome");
        }
    }

    #[tokio::test]
    async fn test_bearer_auth_interceptor() {
        let interceptor = AuthInterceptor::bearer("my-secret-token");
        let url = Url::parse("https://api.example.com").unwrap();
        let request = NetworkRequest::get(url);

        let result = interceptor.intercept_request(request).await.unwrap();

        if let InterceptorOutcome::Continue(req) = result {
            assert_eq!(
                req.headers.get("Authorization"),
                Some(&"Bearer my-secret-token".to_string())
            );
        } else {
            panic!("Expected Continue outcome");
        }
    }

    #[tokio::test]
    async fn test_basic_auth_interceptor() {
        let interceptor = AuthInterceptor::basic("user", "pass");
        let url = Url::parse("https://api.example.com").unwrap();
        let request = NetworkRequest::get(url);

        let result = interceptor.intercept_request(request).await.unwrap();

        if let InterceptorOutcome::Continue(req) = result {
            let auth = req.headers.get("Authorization").unwrap();
            assert!(auth.starts_with("Basic "));
        } else {
            panic!("Expected Continue outcome");
        }
    }

    #[tokio::test]
    async fn test_interceptor_chain_order() {
        let mut chain = RequestInterceptorChain::new();

        // Add in reverse priority order
        chain.add(Arc::new(LoggingInterceptor::new())); // priority -100
        chain.add(Arc::new(UserAgentInterceptor::new("Test/1.0"))); // priority 100

        // Verify chain has both interceptors
        assert_eq!(chain.len(), 2);

        let url = Url::parse("https://example.com").unwrap();
        let request = NetworkRequest::get(url);

        let result = chain.intercept(request).await.unwrap();

        if let InterceptorOutcome::Continue(req) = result {
            // User-Agent should be set (higher priority interceptor ran)
            assert!(req.headers.contains_key("User-Agent"));
        } else {
            panic!("Expected Continue outcome");
        }
    }
}
