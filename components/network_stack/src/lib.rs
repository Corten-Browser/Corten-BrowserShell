//! Network Stack Component
//!
//! Provides HTTP request handling and resource loading for the CortenBrowser Browser Shell.
//!
//! # Features
//!
//! - **HTTP/HTTPS request handling**: Full support for all HTTP methods with configurable timeouts
//! - **Resource loading**: Type-aware loading for images, scripts, stylesheets, and more
//! - **Request/response interceptors**: Chainable interceptors for authentication, logging, and transformation
//! - **Cookie management interface**: Flexible cookie store abstraction
//! - **Cache control**: Header-based cache semantics with configurable modes
//! - **Connection pooling**: Efficient connection reuse with configurable pool settings
//! - **Timeout configuration**: Per-request and client-level timeout settings
//!
//! # Architecture
//!
//! The network stack is built around the [`NetworkClient`] trait, which defines the core
//! interface for making HTTP requests. The default implementation [`HttpClient`] uses
//! reqwest under the hood.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      NetworkClient                          │
//! │  ┌─────────────────┐    ┌────────────────┐                 │
//! │  │ Request         │───►│ Request        │                 │
//! │  │ Interceptors    │    │ Execution      │                 │
//! │  └─────────────────┘    └───────┬────────┘                 │
//! │                                 │                           │
//! │                                 ▼                           │
//! │  ┌─────────────────┐    ┌────────────────┐                 │
//! │  │ Response        │◄───│ Response       │                 │
//! │  │ Interceptors    │    │ Parsing        │                 │
//! │  └─────────────────┘    └────────────────┘                 │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Quick Start
//!
//! ## Simple GET Request
//!
//! ```rust,ignore
//! use network_stack::{HttpClient, NetworkClient, NetworkRequest};
//! use url::Url;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = HttpClient::new().unwrap();
//!     let url = Url::parse("https://api.example.com/data").unwrap();
//!     let request = NetworkRequest::get(url);
//!
//!     let response = client.fetch(request).await.unwrap();
//!     println!("Status: {}", response.status);
//!     println!("Body: {}", response.text().unwrap());
//! }
//! ```
//!
//! ## POST with JSON Body
//!
//! ```rust,ignore
//! use network_stack::{HttpClient, NetworkClient, NetworkRequest};
//! use url::Url;
//! use serde_json::json;
//!
//! let client = HttpClient::new().unwrap();
//! let url = Url::parse("https://api.example.com/users").unwrap();
//!
//! let request = NetworkRequest::post(url)
//!     .json(&json!({"name": "John", "email": "john@example.com"}))
//!     .unwrap();
//!
//! let response = client.fetch(request).await.unwrap();
//! ```
//!
//! ## Using Interceptors
//!
//! ```rust,ignore
//! use network_stack::{HttpClient, HttpClientBuilder, AuthInterceptor, LoggingInterceptor};
//! use std::sync::Arc;
//!
//! let client = HttpClientBuilder::new()
//!     .timeout(Duration::from_secs(60))
//!     .request_interceptor(Arc::new(AuthInterceptor::bearer("my-token")))
//!     .request_interceptor(Arc::new(LoggingInterceptor::new()))
//!     .response_interceptor(Arc::new(LoggingInterceptor::new()))
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Interceptors
//!
//! Interceptors allow you to modify requests before they are sent and responses
//! after they are received. Common use cases include:
//!
//! - **Authentication**: Add bearer tokens or API keys to requests
//! - **Logging**: Log request/response details for debugging
//! - **Caching**: Implement custom caching logic
//! - **Transformation**: Modify request/response data
//! - **Security**: Block requests to certain domains
//!
//! See the [`interceptor`] module for available interceptors and how to create custom ones.
//!
//! # Cache Control
//!
//! The network stack respects standard HTTP cache headers:
//!
//! - `Cache-Control`: max-age, no-store, no-cache, public, private
//! - `ETag`: For conditional requests
//! - `Last-Modified`: For conditional requests
//! - `Expires`: For cache expiration
//!
//! You can control cache behavior per-request using [`CacheMode`]:
//!
//! ```rust,ignore
//! let request = NetworkRequest::get(url)
//!     .cache_mode(CacheMode::NoStore);  // Bypass cache
//! ```

mod cache;
mod client;
mod error;
mod interceptor;
mod privacy_interceptor;
pub mod protocol;
mod request;
mod resource_loader;
mod response;

// Re-export public types
pub use cache::{CacheEntry, CacheStorage, CachingInterceptor, DiskCache, MemoryCache};
pub use client::{
    Cookie, CookieStore, HttpClient, HttpClientBuilder, NetworkClient, NetworkClientConfig,
    SameSite,
};
pub use error::{NetworkError, NetworkResult};
pub use interceptor::{
    AuthInterceptor, AuthType, InterceptorOutcome, LoggingInterceptor, RequestInterceptor,
    RequestInterceptorChain, ResponseInterceptor, ResponseInterceptorChain, RetryInterceptor,
    UserAgentInterceptor,
};
pub use privacy_interceptor::{PrivacyInterceptor, PrivacyInterceptorConfig};
pub use request::{
    CacheMode, CredentialsMode, HeaderMap, Method, NetworkRequest, RedirectPolicy, ResourceType,
};
pub use resource_loader::{ResourceLoadResult, ResourceLoader, ResourceLoaderBuilder};
pub use response::{CacheStatus, NetworkResponse, StatusCode};

/// Re-export url crate for convenience.
pub use url::Url;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api_exports() {
        // Verify all important types are exported
        let _ = NetworkRequest::get;
        let _ = NetworkResponse::new;
        let _ = HttpClient::new;
        let _ = HttpClientBuilder::new;
        let _ = NetworkError::Timeout { url: String::new(), timeout_ms: 0 };
        let _ = Method::Get;
        let _ = StatusCode::OK;
        let _ = CacheMode::Default;
        let _ = ResourceType::Document;
    }
}
