//! Protocol router for routing URLs to appropriate handlers.

use std::collections::HashMap;
use std::sync::RwLock;
use url::Url;

use super::response::ProtocolResponse;
use super::types::{ProtocolError, ProtocolHandler, ProtocolResult};

/// Routes URLs to their appropriate protocol handlers.
///
/// The router maintains a registry of protocol handlers and routes incoming
/// URL requests to the appropriate handler based on the URL scheme.
///
/// # Example
///
/// ```rust,ignore
/// use network_stack::protocol::{ProtocolRouter, FileProtocolHandler, HttpProtocolHandler};
/// use std::sync::Arc;
///
/// let mut router = ProtocolRouter::new();
///
/// // Register handlers
/// router.register(Box::new(FileProtocolHandler::new()));
/// router.register(Box::new(HttpProtocolHandler::new(client)));
///
/// // Route a URL
/// let url = Url::parse("file:///home/user/doc.html").unwrap();
/// let response = router.handle(&url).await.unwrap();
/// ```
pub struct ProtocolRouter {
    /// Map from scheme to handler.
    handlers: RwLock<HashMap<String, Box<dyn ProtocolHandler>>>,

    /// Order in which handlers were registered (for iteration).
    registration_order: RwLock<Vec<String>>,
}

impl ProtocolRouter {
    /// Create a new empty protocol router.
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
            registration_order: RwLock::new(Vec::new()),
        }
    }

    /// Register a protocol handler.
    ///
    /// The handler will be registered for all schemes it supports.
    /// If a handler already exists for a scheme, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler to register.
    pub fn register(&self, handler: Box<dyn ProtocolHandler>) {
        let schemes = handler.schemes();
        let mut handlers = self.handlers.write().unwrap();
        let mut order = self.registration_order.write().unwrap();

        for scheme in schemes {
            let scheme_lower = scheme.to_lowercase();

            // Remove from order if already present
            order.retain(|s| s != &scheme_lower);

            // Add to handlers and order
            handlers.insert(scheme_lower.clone(), handler.clone_handler());
            order.push(scheme_lower);
        }
    }

    /// Unregister a handler for a specific scheme.
    ///
    /// # Arguments
    ///
    /// * `scheme` - The scheme to unregister.
    ///
    /// # Returns
    ///
    /// `true` if a handler was removed, `false` if no handler was registered.
    pub fn unregister(&self, scheme: &str) -> bool {
        let scheme_lower = scheme.to_lowercase();
        let mut handlers = self.handlers.write().unwrap();
        let mut order = self.registration_order.write().unwrap();

        if handlers.remove(&scheme_lower).is_some() {
            order.retain(|s| s != &scheme_lower);
            true
        } else {
            false
        }
    }

    /// Get the handler for a specific scheme.
    ///
    /// # Arguments
    ///
    /// * `scheme` - The scheme to look up.
    ///
    /// # Returns
    ///
    /// A reference to the handler, or `None` if no handler is registered.
    pub fn get_handler(&self, scheme: &str) -> Option<&dyn ProtocolHandler> {
        let scheme_lower = scheme.to_lowercase();
        let handlers = self.handlers.read().unwrap();
        // We can't return a reference to data behind the lock
        // This is a limitation - we'll need to use the route/handle methods instead
        drop(handlers);
        None
    }

    /// Route a URL to find the appropriate handler.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to route.
    ///
    /// # Returns
    ///
    /// The scheme of the handler that will handle this URL, or `None`.
    pub fn route(&self, url: &Url) -> Option<String> {
        let scheme = url.scheme().to_lowercase();
        let handlers = self.handlers.read().unwrap();

        if handlers.contains_key(&scheme) {
            Some(scheme)
        } else {
            None
        }
    }

    /// Check if a handler is registered for the given scheme.
    ///
    /// # Arguments
    ///
    /// * `scheme` - The scheme to check.
    pub fn has_handler(&self, scheme: &str) -> bool {
        let scheme_lower = scheme.to_lowercase();
        let handlers = self.handlers.read().unwrap();
        handlers.contains_key(&scheme_lower)
    }

    /// Check if this router can handle the given URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check.
    pub fn can_handle(&self, url: &Url) -> bool {
        self.has_handler(url.scheme())
    }

    /// Handle a URL request by routing to the appropriate handler.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to handle.
    ///
    /// # Returns
    ///
    /// A `ProtocolResponse` from the handler, or a `ProtocolError` if no handler
    /// is registered or the handler fails.
    pub async fn handle(&self, url: &Url) -> ProtocolResult<ProtocolResponse> {
        let scheme = url.scheme().to_lowercase();

        // Get handler reference under read lock, then drop lock before await
        let handler_result = {
            let handlers = self.handlers.read().unwrap();
            handlers.get(&scheme).map(|h| h.clone_handler())
        };

        match handler_result {
            Some(handler) => handler.handle(url).await,
            None => Err(ProtocolError::no_handler(url)),
        }
    }

    /// Get a list of all registered schemes.
    pub fn registered_schemes(&self) -> Vec<String> {
        let order = self.registration_order.read().unwrap();
        order.clone()
    }

    /// Get handler information for debugging/logging.
    pub fn handler_info(&self) -> Vec<(String, String)> {
        let handlers = self.handlers.read().unwrap();
        let order = self.registration_order.read().unwrap();

        order
            .iter()
            .filter_map(|scheme| {
                handlers
                    .get(scheme)
                    .map(|h| (scheme.clone(), h.name().to_string()))
            })
            .collect()
    }

    /// Clear all registered handlers.
    pub fn clear(&self) {
        let mut handlers = self.handlers.write().unwrap();
        let mut order = self.registration_order.write().unwrap();

        handlers.clear();
        order.clear();
    }
}

impl Default for ProtocolRouter {
    fn default() -> Self {
        Self::new()
    }
}

// We need a way to clone handlers for async handling
// This is a workaround for the async lifetime issue
trait CloneHandler: ProtocolHandler {
    fn clone_handler(&self) -> Box<dyn ProtocolHandler>;
}

impl<T: ProtocolHandler + Clone + 'static> CloneHandler for T {
    fn clone_handler(&self) -> Box<dyn ProtocolHandler> {
        Box::new(self.clone())
    }
}

// Implement for Box<dyn ProtocolHandler>
impl dyn ProtocolHandler {
    fn clone_handler(&self) -> Box<dyn ProtocolHandler> {
        // This is a placeholder - actual implementation requires Clone
        // In practice, handlers should implement Clone
        panic!("Protocol handlers must implement Clone")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    #[derive(Clone)]
    struct MockHandler {
        scheme: String,
        name: String,
    }

    impl MockHandler {
        fn new(scheme: &str) -> Self {
            Self {
                scheme: scheme.to_string(),
                name: format!("{} handler", scheme),
            }
        }
    }

    #[async_trait]
    impl ProtocolHandler for MockHandler {
        fn scheme(&self) -> &str {
            &self.scheme
        }

        fn name(&self) -> &str {
            &self.name
        }

        async fn handle(&self, url: &Url) -> ProtocolResult<ProtocolResponse> {
            Ok(ProtocolResponse::text(
                url.clone(),
                format!("Handled by {}", self.scheme),
                &self.scheme,
            ))
        }
    }

    #[derive(Clone)]
    struct MultiSchemeHandler;

    #[async_trait]
    impl ProtocolHandler for MultiSchemeHandler {
        fn scheme(&self) -> &str {
            "http"
        }

        fn schemes(&self) -> Vec<&str> {
            vec!["http", "https"]
        }

        fn name(&self) -> &str {
            "HTTP/HTTPS"
        }

        async fn handle(&self, url: &Url) -> ProtocolResult<ProtocolResponse> {
            Ok(ProtocolResponse::text(
                url.clone(),
                "HTTP response",
                url.scheme(),
            ))
        }
    }

    #[test]
    fn test_router_new() {
        let router = ProtocolRouter::new();
        assert!(router.registered_schemes().is_empty());
    }

    #[test]
    fn test_router_register_single_scheme() {
        let router = ProtocolRouter::new();
        router.register(Box::new(MockHandler::new("test")));

        assert!(router.has_handler("test"));
        assert!(router.has_handler("TEST")); // Case insensitive
        assert!(!router.has_handler("other"));
        assert_eq!(router.registered_schemes(), vec!["test"]);
    }

    #[test]
    fn test_router_register_multi_scheme() {
        let router = ProtocolRouter::new();
        router.register(Box::new(MultiSchemeHandler));

        assert!(router.has_handler("http"));
        assert!(router.has_handler("https"));
        assert!(router.has_handler("HTTP")); // Case insensitive
        assert!(router.has_handler("HTTPS")); // Case insensitive
    }

    #[test]
    fn test_router_unregister() {
        let router = ProtocolRouter::new();
        router.register(Box::new(MockHandler::new("test")));

        assert!(router.has_handler("test"));
        assert!(router.unregister("test"));
        assert!(!router.has_handler("test"));
        assert!(!router.unregister("test")); // Already removed
    }

    #[test]
    fn test_router_route() {
        let router = ProtocolRouter::new();
        router.register(Box::new(MockHandler::new("file")));

        let file_url = Url::parse("file:///path/to/file").unwrap();
        let http_url = Url::parse("http://example.com").unwrap();

        assert_eq!(router.route(&file_url), Some("file".to_string()));
        assert_eq!(router.route(&http_url), None);
    }

    #[test]
    fn test_router_can_handle() {
        let router = ProtocolRouter::new();
        router.register(Box::new(MockHandler::new("file")));

        let file_url = Url::parse("file:///path/to/file").unwrap();
        let http_url = Url::parse("http://example.com").unwrap();

        assert!(router.can_handle(&file_url));
        assert!(!router.can_handle(&http_url));
    }

    #[test]
    fn test_router_handler_info() {
        let router = ProtocolRouter::new();
        router.register(Box::new(MockHandler::new("file")));
        router.register(Box::new(MockHandler::new("custom")));

        let info = router.handler_info();
        assert_eq!(info.len(), 2);

        let schemes: Vec<&str> = info.iter().map(|(s, _)| s.as_str()).collect();
        assert!(schemes.contains(&"file"));
        assert!(schemes.contains(&"custom"));
    }

    #[test]
    fn test_router_clear() {
        let router = ProtocolRouter::new();
        router.register(Box::new(MockHandler::new("file")));
        router.register(Box::new(MockHandler::new("custom")));

        assert_eq!(router.registered_schemes().len(), 2);

        router.clear();

        assert!(router.registered_schemes().is_empty());
        assert!(!router.has_handler("file"));
        assert!(!router.has_handler("custom"));
    }

    #[test]
    fn test_router_replace_handler() {
        let router = ProtocolRouter::new();
        router.register(Box::new(MockHandler::new("test")));

        let info = router.handler_info();
        assert_eq!(info[0].1, "test handler");

        // Register a new handler for the same scheme
        let mut new_handler = MockHandler::new("test");
        new_handler.name = "New test handler".to_string();
        router.register(Box::new(new_handler));

        let info = router.handler_info();
        assert_eq!(info.len(), 1);
        assert_eq!(info[0].1, "New test handler");
    }

    #[tokio::test]
    async fn test_router_handle_success() {
        let router = ProtocolRouter::new();
        router.register(Box::new(MockHandler::new("test")));

        let url = Url::parse("test://example").unwrap();
        let response = router.handle(&url).await.unwrap();

        assert!(response.is_success());
        assert_eq!(response.text().unwrap(), "Handled by test");
    }

    #[tokio::test]
    async fn test_router_handle_no_handler() {
        let router = ProtocolRouter::new();

        let url = Url::parse("unknown://example").unwrap();
        let result = router.handle(&url).await;

        assert!(matches!(result, Err(ProtocolError::NoHandler { .. })));
    }
}
