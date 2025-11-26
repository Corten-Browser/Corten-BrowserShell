//! WebRequest API for extension network interception.
//!
//! This module provides Chrome-compatible webRequest API that allows
//! extensions to observe and modify network requests.

use crate::types::{ExtensionId, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

/// WebRequest event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WebRequestEvent {
    /// Request is about to be made
    OnBeforeRequest,
    /// Request headers are about to be sent
    OnBeforeSendHeaders,
    /// Request was sent
    OnSendHeaders,
    /// Response headers have been received
    OnHeadersReceived,
    /// Authentication is required
    OnAuthRequired,
    /// Response started
    OnResponseStarted,
    /// Request completed successfully
    OnCompleted,
    /// Request encountered an error
    OnErrorOccurred,
}

/// Request details provided to extension listeners
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDetails {
    /// Unique request ID
    pub request_id: String,
    /// Request URL
    pub url: Url,
    /// Request method (GET, POST, etc.)
    pub method: String,
    /// Frame ID (0 for main frame)
    pub frame_id: u64,
    /// Parent frame ID (-1 if no parent)
    pub parent_frame_id: i64,
    /// Tab ID (-1 if not in a tab)
    pub tab_id: i64,
    /// Resource type
    pub resource_type: ResourceType,
    /// Request timestamp (milliseconds since epoch)
    pub timestamp: u64,
    /// Request headers
    pub request_headers: Option<HashMap<String, String>>,
    /// Response headers
    pub response_headers: Option<HashMap<String, String>>,
    /// HTTP status code
    pub status_code: Option<u16>,
    /// Status line
    pub status_line: Option<String>,
}

/// Resource type for webRequest filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    MainFrame,
    SubFrame,
    Stylesheet,
    Script,
    Image,
    Font,
    Object,
    XmlHttpRequest,
    Ping,
    CspReport,
    Media,
    WebSocket,
    Other,
}

/// Request modification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestAction {
    /// Allow the request to continue unchanged
    Continue,
    /// Cancel the request
    Cancel,
    /// Redirect to a different URL
    Redirect { url: Url },
    /// Modify request headers
    ModifyHeaders { headers: HashMap<String, String> },
    /// Provide authentication credentials
    Auth { username: String, password: String },
}

/// WebRequest listener callback type
pub type WebRequestListener = Arc<
    dyn Fn(RequestDetails) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<RequestAction>> + Send>>
        + Send
        + Sync,
>;

/// WebRequest filter for matching requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestFilter {
    /// URL patterns to match
    pub urls: Vec<String>,
    /// Resource types to match
    pub types: Option<Vec<ResourceType>>,
    /// Tab ID to match (-1 for all tabs)
    pub tab_id: Option<i64>,
    /// Window ID to match
    pub window_id: Option<i64>,
}

impl RequestFilter {
    /// Create a new request filter
    pub fn new(urls: Vec<String>) -> Self {
        Self {
            urls,
            types: None,
            tab_id: None,
            window_id: None,
        }
    }

    /// Check if this filter matches the request details
    pub fn matches(&self, details: &RequestDetails) -> bool {
        // Check URL patterns
        let url_match = self.urls.iter().any(|pattern| {
            if pattern == "<all_urls>" {
                true
            } else {
                // Simple wildcard matching
                let regex_pattern = pattern
                    .replace('.', r"\.")
                    .replace('*', ".*")
                    .replace('?', ".");
                regex::Regex::new(&format!("^{}$", regex_pattern))
                    .map(|re| re.is_match(details.url.as_str()))
                    .unwrap_or(false)
            }
        });

        if !url_match {
            return false;
        }

        // Check resource types
        if let Some(ref types) = self.types {
            if !types.contains(&details.resource_type) {
                return false;
            }
        }

        // Check tab ID
        if let Some(tab_id) = self.tab_id {
            if tab_id != -1 && tab_id != details.tab_id {
                return false;
            }
        }

        true
    }
}

/// Registered listener for webRequest events
#[derive(Clone)]
struct RegisteredListener {
    extension_id: ExtensionId,
    filter: RequestFilter,
    callback: WebRequestListener,
    extra_info: Vec<String>,
}

/// WebRequest API implementation
pub struct WebRequestApi {
    /// Listeners organized by event type
    listeners: Arc<RwLock<HashMap<WebRequestEvent, Vec<RegisteredListener>>>>,
    /// Active requests being processed
    active_requests: Arc<RwLock<HashMap<String, RequestDetails>>>,
}

impl WebRequestApi {
    /// Create a new WebRequest API
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
            active_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a listener for a webRequest event
    pub async fn add_listener(
        &self,
        extension_id: ExtensionId,
        event: WebRequestEvent,
        filter: RequestFilter,
        callback: WebRequestListener,
        extra_info: Vec<String>,
    ) -> Result<()> {
        let mut listeners = self.listeners.write().await;

        let listener = RegisteredListener {
            extension_id,
            filter,
            callback,
            extra_info,
        };

        listeners
            .entry(event)
            .or_insert_with(Vec::new)
            .push(listener);

        tracing::debug!(
            extension_id = %extension_id,
            event = ?event,
            "Registered webRequest listener"
        );

        Ok(())
    }

    /// Remove all listeners for an extension
    pub async fn remove_extension_listeners(&self, extension_id: ExtensionId) -> Result<()> {
        let mut listeners = self.listeners.write().await;

        for event_listeners in listeners.values_mut() {
            event_listeners.retain(|l| l.extension_id != extension_id);
        }

        tracing::debug!(
            extension_id = %extension_id,
            "Removed all webRequest listeners"
        );

        Ok(())
    }

    /// Fire a webRequest event and collect responses
    pub async fn fire_event(
        &self,
        event: WebRequestEvent,
        details: RequestDetails,
    ) -> Result<Vec<RequestAction>> {
        // Store active request
        {
            let mut active = self.active_requests.write().await;
            active.insert(details.request_id.clone(), details.clone());
        }

        let listeners = self.listeners.read().await;
        let event_listeners = listeners.get(&event);

        let mut actions = Vec::new();

        // Call all matching listeners (if any)
        if let Some(listeners_list) = event_listeners {
            for listener in listeners_list {
                if listener.filter.matches(&details) {
                    tracing::debug!(
                        extension_id = %listener.extension_id,
                        event = ?event,
                        url = %details.url,
                        "Calling webRequest listener"
                    );

                    match (listener.callback)(details.clone()).await {
                        Ok(action) => {
                            actions.push(action);
                        }
                        Err(e) => {
                            tracing::error!(
                                extension_id = %listener.extension_id,
                                error = %e,
                                "WebRequest listener error"
                            );
                        }
                    }
                }
            }
        }

        // Cleanup for terminal events
        if matches!(event, WebRequestEvent::OnCompleted | WebRequestEvent::OnErrorOccurred) {
            let mut active = self.active_requests.write().await;
            active.remove(&details.request_id);
        }

        Ok(actions)
    }

    /// Get details of an active request
    pub async fn get_request_details(&self, request_id: &str) -> Option<RequestDetails> {
        let active = self.active_requests.read().await;
        active.get(request_id).cloned()
    }

    /// Get all active request IDs
    pub async fn get_active_requests(&self) -> Vec<String> {
        let active = self.active_requests.read().await;
        active.keys().cloned().collect()
    }

    /// Resolve conflicting actions from multiple listeners
    pub fn resolve_actions(&self, actions: Vec<RequestAction>) -> RequestAction {
        // Priority: Cancel > Redirect > ModifyHeaders > Auth > Continue
        for action in &actions {
            if matches!(action, RequestAction::Cancel) {
                return action.clone();
            }
        }

        for action in &actions {
            if matches!(action, RequestAction::Redirect { .. }) {
                return action.clone();
            }
        }

        for action in &actions {
            if matches!(action, RequestAction::ModifyHeaders { .. }) {
                return action.clone();
            }
        }

        for action in &actions {
            if matches!(action, RequestAction::Auth { .. }) {
                return action.clone();
            }
        }

        RequestAction::Continue
    }
}

impl Default for WebRequestApi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_details(url: &str) -> RequestDetails {
        RequestDetails {
            request_id: "test-123".to_string(),
            url: Url::parse(url).unwrap(),
            method: "GET".to_string(),
            frame_id: 0,
            parent_frame_id: -1,
            tab_id: 1,
            resource_type: ResourceType::MainFrame,
            timestamp: 0,
            request_headers: None,
            response_headers: None,
            status_code: None,
            status_line: None,
        }
    }

    #[test]
    fn test_request_filter_url_match() {
        let filter = RequestFilter::new(vec!["https://example.com/*".to_string()]);

        let details1 = create_test_details("https://example.com/page");
        assert!(filter.matches(&details1));

        let details2 = create_test_details("https://other.com/page");
        assert!(!filter.matches(&details2));
    }

    #[test]
    fn test_request_filter_all_urls() {
        let filter = RequestFilter::new(vec!["<all_urls>".to_string()]);

        let details1 = create_test_details("https://example.com/page");
        assert!(filter.matches(&details1));

        let details2 = create_test_details("http://test.org/");
        assert!(filter.matches(&details2));
    }

    #[test]
    fn test_request_filter_resource_type() {
        let mut filter = RequestFilter::new(vec!["<all_urls>".to_string()]);
        filter.types = Some(vec![ResourceType::Script, ResourceType::Image]);

        let mut details = create_test_details("https://example.com/script.js");
        details.resource_type = ResourceType::Script;
        assert!(filter.matches(&details));

        details.resource_type = ResourceType::MainFrame;
        assert!(!filter.matches(&details));
    }

    #[tokio::test]
    async fn test_add_listener() {
        let api = WebRequestApi::new();
        let ext_id = ExtensionId::new();
        let filter = RequestFilter::new(vec!["<all_urls>".to_string()]);

        let callback = Arc::new(|_details: RequestDetails| {
            Box::pin(async { Ok(RequestAction::Continue) })
                as std::pin::Pin<Box<dyn std::future::Future<Output = Result<RequestAction>> + Send>>
        });

        let result = api.add_listener(
            ext_id,
            WebRequestEvent::OnBeforeRequest,
            filter,
            callback,
            vec![],
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fire_event() {
        let api = WebRequestApi::new();
        let ext_id = ExtensionId::new();
        let filter = RequestFilter::new(vec!["https://example.com/*".to_string()]);

        let callback = Arc::new(|_details: RequestDetails| {
            Box::pin(async { Ok(RequestAction::Cancel) })
                as std::pin::Pin<Box<dyn std::future::Future<Output = Result<RequestAction>> + Send>>
        });

        api.add_listener(
            ext_id,
            WebRequestEvent::OnBeforeRequest,
            filter,
            callback,
            vec![],
        ).await.unwrap();

        let details = create_test_details("https://example.com/page");
        let actions = api.fire_event(WebRequestEvent::OnBeforeRequest, details).await.unwrap();

        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], RequestAction::Cancel));
    }

    #[tokio::test]
    async fn test_remove_extension_listeners() {
        let api = WebRequestApi::new();
        let ext_id = ExtensionId::new();
        let filter = RequestFilter::new(vec!["<all_urls>".to_string()]);

        let callback = Arc::new(|_details: RequestDetails| {
            Box::pin(async { Ok(RequestAction::Continue) })
                as std::pin::Pin<Box<dyn std::future::Future<Output = Result<RequestAction>> + Send>>
        });

        api.add_listener(ext_id, WebRequestEvent::OnBeforeRequest, filter, callback, vec![])
            .await
            .unwrap();

        api.remove_extension_listeners(ext_id).await.unwrap();

        // Fire event - should have no listeners
        let details = create_test_details("https://example.com/");
        let actions = api.fire_event(WebRequestEvent::OnBeforeRequest, details).await.unwrap();
        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn test_resolve_actions() {
        let api = WebRequestApi::new();

        // Cancel has highest priority
        let actions = vec![
            RequestAction::Continue,
            RequestAction::Cancel,
            RequestAction::Redirect { url: Url::parse("https://example.com").unwrap() },
        ];
        assert!(matches!(api.resolve_actions(actions), RequestAction::Cancel));

        // Redirect has second priority
        let actions = vec![
            RequestAction::Continue,
            RequestAction::Redirect { url: Url::parse("https://example.com").unwrap() },
        ];
        assert!(matches!(api.resolve_actions(actions), RequestAction::Redirect { .. }));

        // Continue is default
        let actions = vec![RequestAction::Continue, RequestAction::Continue];
        assert!(matches!(api.resolve_actions(actions), RequestAction::Continue));
    }

    #[tokio::test]
    async fn test_active_requests_tracking() {
        let api = WebRequestApi::new();
        let ext_id = ExtensionId::new();
        let filter = RequestFilter::new(vec!["<all_urls>".to_string()]);

        let callback = Arc::new(|_details: RequestDetails| {
            Box::pin(async { Ok(RequestAction::Continue) })
                as std::pin::Pin<Box<dyn std::future::Future<Output = Result<RequestAction>> + Send>>
        });

        api.add_listener(ext_id, WebRequestEvent::OnBeforeRequest, filter, callback, vec![])
            .await
            .unwrap();

        let details = create_test_details("https://example.com/");
        let request_id = details.request_id.clone();

        // Fire OnBeforeRequest - should add to active requests
        api.fire_event(WebRequestEvent::OnBeforeRequest, details.clone()).await.unwrap();

        assert!(api.get_request_details(&request_id).await.is_some());

        // Fire OnCompleted - should remove from active requests
        api.fire_event(WebRequestEvent::OnCompleted, details).await.unwrap();

        assert!(api.get_request_details(&request_id).await.is_none());
    }
}
