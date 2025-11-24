//! Privacy interceptor for the network stack.
//!
//! Provides request interception for privacy headers including:
//! - Do Not Track (DNT) header
//! - Global Privacy Control (GPC) header

use crate::error::NetworkResult;
use crate::interceptor::{InterceptorOutcome, RequestInterceptor};
use crate::request::NetworkRequest;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for the privacy interceptor.
#[derive(Debug, Clone)]
pub struct PrivacyInterceptorConfig {
    /// Whether to send the DNT header.
    pub dnt_enabled: bool,
    /// Whether to send the GPC header.
    pub gpc_enabled: bool,
}

impl Default for PrivacyInterceptorConfig {
    fn default() -> Self {
        Self {
            dnt_enabled: true,
            gpc_enabled: true,
        }
    }
}

/// Interceptor that adds privacy headers (DNT, GPC) to requests.
#[derive(Debug)]
pub struct PrivacyInterceptor {
    config: Arc<RwLock<PrivacyInterceptorConfig>>,
}

impl PrivacyInterceptor {
    /// Create a new privacy interceptor with default configuration.
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(PrivacyInterceptorConfig::default())),
        }
    }

    /// Create a new privacy interceptor with custom configuration.
    pub fn with_config(config: PrivacyInterceptorConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Create a new privacy interceptor with only DNT enabled.
    pub fn dnt_only() -> Self {
        Self::with_config(PrivacyInterceptorConfig {
            dnt_enabled: true,
            gpc_enabled: false,
        })
    }

    /// Create a new privacy interceptor with only GPC enabled.
    pub fn gpc_only() -> Self {
        Self::with_config(PrivacyInterceptorConfig {
            dnt_enabled: false,
            gpc_enabled: true,
        })
    }

    /// Enable or disable DNT.
    pub async fn set_dnt_enabled(&self, enabled: bool) {
        self.config.write().await.dnt_enabled = enabled;
    }

    /// Check if DNT is enabled.
    pub async fn is_dnt_enabled(&self) -> bool {
        self.config.read().await.dnt_enabled
    }

    /// Enable or disable GPC.
    pub async fn set_gpc_enabled(&self, enabled: bool) {
        self.config.write().await.gpc_enabled = enabled;
    }

    /// Check if GPC is enabled.
    pub async fn is_gpc_enabled(&self) -> bool {
        self.config.read().await.gpc_enabled
    }

    /// Update the full configuration.
    pub async fn update_config(&self, config: PrivacyInterceptorConfig) {
        let mut current = self.config.write().await;
        *current = config;
    }

    /// Get the current configuration.
    pub async fn get_config(&self) -> PrivacyInterceptorConfig {
        self.config.read().await.clone()
    }
}

impl Default for PrivacyInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PrivacyInterceptor {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
        }
    }
}

#[async_trait]
impl RequestInterceptor for PrivacyInterceptor {
    async fn intercept_request(
        &self,
        mut request: NetworkRequest,
    ) -> NetworkResult<InterceptorOutcome<NetworkRequest>> {
        let config = self.config.read().await;

        // Add DNT header if enabled
        if config.dnt_enabled {
            request.headers.insert("DNT".to_string(), "1".to_string());
        }

        // Add GPC header if enabled
        if config.gpc_enabled {
            request
                .headers
                .insert("Sec-GPC".to_string(), "1".to_string());
        }

        Ok(InterceptorOutcome::Continue(request))
    }

    fn name(&self) -> &str {
        "PrivacyInterceptor"
    }

    fn priority(&self) -> i32 {
        95 // Run early, after user-agent but before auth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[tokio::test]
    async fn test_default_config() {
        let interceptor = PrivacyInterceptor::new();
        let config = interceptor.get_config().await;

        assert!(config.dnt_enabled);
        assert!(config.gpc_enabled);
    }

    #[tokio::test]
    async fn test_adds_dnt_header() {
        let interceptor = PrivacyInterceptor::new();
        let url = Url::parse("https://example.com").unwrap();
        let request = NetworkRequest::get(url);

        let result = interceptor.intercept_request(request).await.unwrap();

        if let InterceptorOutcome::Continue(req) = result {
            assert_eq!(req.headers.get("DNT"), Some(&"1".to_string()));
        } else {
            panic!("Expected Continue outcome");
        }
    }

    #[tokio::test]
    async fn test_adds_gpc_header() {
        let interceptor = PrivacyInterceptor::new();
        let url = Url::parse("https://example.com").unwrap();
        let request = NetworkRequest::get(url);

        let result = interceptor.intercept_request(request).await.unwrap();

        if let InterceptorOutcome::Continue(req) = result {
            assert_eq!(req.headers.get("Sec-GPC"), Some(&"1".to_string()));
        } else {
            panic!("Expected Continue outcome");
        }
    }

    #[tokio::test]
    async fn test_dnt_only() {
        let interceptor = PrivacyInterceptor::dnt_only();
        let url = Url::parse("https://example.com").unwrap();
        let request = NetworkRequest::get(url);

        let result = interceptor.intercept_request(request).await.unwrap();

        if let InterceptorOutcome::Continue(req) = result {
            assert_eq!(req.headers.get("DNT"), Some(&"1".to_string()));
            assert!(req.headers.get("Sec-GPC").is_none());
        } else {
            panic!("Expected Continue outcome");
        }
    }

    #[tokio::test]
    async fn test_gpc_only() {
        let interceptor = PrivacyInterceptor::gpc_only();
        let url = Url::parse("https://example.com").unwrap();
        let request = NetworkRequest::get(url);

        let result = interceptor.intercept_request(request).await.unwrap();

        if let InterceptorOutcome::Continue(req) = result {
            assert!(req.headers.get("DNT").is_none());
            assert_eq!(req.headers.get("Sec-GPC"), Some(&"1".to_string()));
        } else {
            panic!("Expected Continue outcome");
        }
    }

    #[tokio::test]
    async fn test_disable_dnt() {
        let interceptor = PrivacyInterceptor::new();
        interceptor.set_dnt_enabled(false).await;

        assert!(!interceptor.is_dnt_enabled().await);

        let url = Url::parse("https://example.com").unwrap();
        let request = NetworkRequest::get(url);

        let result = interceptor.intercept_request(request).await.unwrap();

        if let InterceptorOutcome::Continue(req) = result {
            assert!(req.headers.get("DNT").is_none());
            assert_eq!(req.headers.get("Sec-GPC"), Some(&"1".to_string()));
        } else {
            panic!("Expected Continue outcome");
        }
    }

    #[tokio::test]
    async fn test_disable_gpc() {
        let interceptor = PrivacyInterceptor::new();
        interceptor.set_gpc_enabled(false).await;

        assert!(!interceptor.is_gpc_enabled().await);

        let url = Url::parse("https://example.com").unwrap();
        let request = NetworkRequest::get(url);

        let result = interceptor.intercept_request(request).await.unwrap();

        if let InterceptorOutcome::Continue(req) = result {
            assert_eq!(req.headers.get("DNT"), Some(&"1".to_string()));
            assert!(req.headers.get("Sec-GPC").is_none());
        } else {
            panic!("Expected Continue outcome");
        }
    }

    #[tokio::test]
    async fn test_disable_both() {
        let config = PrivacyInterceptorConfig {
            dnt_enabled: false,
            gpc_enabled: false,
        };
        let interceptor = PrivacyInterceptor::with_config(config);

        let url = Url::parse("https://example.com").unwrap();
        let request = NetworkRequest::get(url);

        let result = interceptor.intercept_request(request).await.unwrap();

        if let InterceptorOutcome::Continue(req) = result {
            assert!(req.headers.get("DNT").is_none());
            assert!(req.headers.get("Sec-GPC").is_none());
        } else {
            panic!("Expected Continue outcome");
        }
    }

    #[tokio::test]
    async fn test_update_config() {
        let interceptor = PrivacyInterceptor::new();

        let new_config = PrivacyInterceptorConfig {
            dnt_enabled: false,
            gpc_enabled: false,
        };
        interceptor.update_config(new_config).await;

        let config = interceptor.get_config().await;
        assert!(!config.dnt_enabled);
        assert!(!config.gpc_enabled);
    }

    #[tokio::test]
    async fn test_interceptor_priority() {
        let interceptor = PrivacyInterceptor::new();
        assert_eq!(interceptor.priority(), 95);
    }

    #[tokio::test]
    async fn test_interceptor_name() {
        let interceptor = PrivacyInterceptor::new();
        assert_eq!(interceptor.name(), "PrivacyInterceptor");
    }

    #[tokio::test]
    async fn test_clone_shares_state() {
        let interceptor = PrivacyInterceptor::new();
        let cloned = interceptor.clone();

        // Change config through original
        interceptor.set_dnt_enabled(false).await;

        // Clone should see the change
        assert!(!cloned.is_dnt_enabled().await);
    }
}
