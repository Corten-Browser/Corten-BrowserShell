//! Resource loader with type-specific handling.
//!
//! This module provides a high-level resource loading API that handles
//! different resource types appropriately (images, scripts, stylesheets, etc.).

use crate::client::NetworkClient;
use crate::error::{NetworkError, NetworkResult};
use crate::request::{NetworkRequest, ResourceType};
use crate::response::NetworkResponse;
use std::sync::Arc;
use tokio::sync::Semaphore;
use url::Url;

/// Resource load result with metadata
#[derive(Debug, Clone)]
pub struct ResourceLoadResult {
    /// The loaded resource response
    pub response: NetworkResponse,
    /// The resource type
    pub resource_type: ResourceType,
    /// Redirect chain if any
    pub redirect_chain: Vec<Url>,
    /// Whether this was loaded from cache
    pub from_cache: bool,
}

/// Resource loader with priority handling and concurrent load limits
pub struct ResourceLoader<C: NetworkClient> {
    client: Arc<C>,
    /// Maximum concurrent loads
    max_concurrent: Arc<Semaphore>,
    /// Priority queues for different resource types
    high_priority_types: Vec<ResourceType>,
}

impl<C: NetworkClient + 'static> ResourceLoader<C> {
    /// Create a new resource loader
    pub fn new(client: Arc<C>, max_concurrent_loads: usize) -> Self {
        Self {
            client,
            max_concurrent: Arc::new(Semaphore::new(max_concurrent_loads)),
            high_priority_types: vec![
                ResourceType::Document,
                ResourceType::Stylesheet,
                ResourceType::Script,
            ],
        }
    }

    /// Load a resource with automatic type-specific handling
    pub async fn load(
        &self,
        url: Url,
        resource_type: ResourceType,
    ) -> NetworkResult<ResourceLoadResult> {
        // Acquire semaphore permit
        let _permit = self.max_concurrent.acquire().await
            .map_err(|e| NetworkError::Internal(format!("Failed to acquire permit: {}", e)))?;

        tracing::debug!(
            url = %url,
            resource_type = ?resource_type,
            "Loading resource"
        );

        // Build request with appropriate settings for resource type
        let request = self.build_request(url, resource_type)?;

        // Execute request
        let response = self.client.fetch(request).await?;

        // Validate response for resource type
        self.validate_response(&response, resource_type)?;

        let from_cache = response.cache_status.is_hit();

        Ok(ResourceLoadResult {
            response,
            resource_type,
            redirect_chain: Vec::new(), // TODO: Track redirects
            from_cache,
        })
    }

    /// Load multiple resources in parallel
    pub async fn load_batch(
        &self,
        resources: Vec<(Url, ResourceType)>,
    ) -> Vec<NetworkResult<ResourceLoadResult>> {
        let mut handles = Vec::new();

        for (url, resource_type) in resources {
            let loader = self.clone();
            let handle = tokio::spawn(async move {
                loader.load(url, resource_type).await
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await
                .map_err(|e| NetworkError::Internal(format!("Task failed: {}", e)))
                .and_then(|r| r);
            results.push(result);
        }

        results
    }

    /// Build a request with appropriate settings for the resource type
    fn build_request(
        &self,
        url: Url,
        resource_type: ResourceType,
    ) -> NetworkResult<NetworkRequest> {
        let mut request = NetworkRequest::get(url).resource_type(resource_type);

        // Set appropriate headers based on resource type
        match resource_type {
            ResourceType::Image => {
                request = request.header("Accept", "image/webp,image/apng,image/*,*/*;q=0.8");
            }
            ResourceType::Stylesheet => {
                request = request.header("Accept", "text/css,*/*;q=0.1");
            }
            ResourceType::Script => {
                request = request.header("Accept", "*/*");
            }
            ResourceType::Font => {
                request = request.header("Accept", "font/woff2,font/woff,*/*;q=0.1");
            }
            ResourceType::Document => {
                request = request.header(
                    "Accept",
                    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                );
            }
            ResourceType::Media => {
                request = request.header("Accept", "video/*,audio/*");
            }
            ResourceType::Other | ResourceType::Xhr | ResourceType::WebSocket => {
                request = request.header("Accept", "*/*");
            }
        }

        Ok(request)
    }

    /// Validate that the response is appropriate for the resource type
    fn validate_response(
        &self,
        response: &NetworkResponse,
        resource_type: ResourceType,
    ) -> NetworkResult<()> {
        // Check status code
        if !response.status.is_success() {
            return Err(NetworkError::HttpError {
                status_code: response.status.as_u16(),
                url: response.url.to_string(),
            });
        }

        // Validate content type for certain resource types
        if let Some(content_type) = response.headers.get("content-type") {
            let valid = match resource_type {
                ResourceType::Image => {
                    content_type.starts_with("image/")
                }
                ResourceType::Stylesheet => {
                    content_type.starts_with("text/css") || content_type.contains("stylesheet")
                }
                ResourceType::Script => {
                    content_type.contains("javascript") ||
                    content_type.contains("ecmascript") ||
                    content_type.starts_with("text/javascript") ||
                    content_type.starts_with("application/javascript")
                }
                ResourceType::Font => {
                    content_type.starts_with("font/") ||
                    content_type.contains("woff") ||
                    content_type.contains("truetype") ||
                    content_type.contains("opentype")
                }
                ResourceType::Media => {
                    content_type.starts_with("video/") || content_type.starts_with("audio/")
                }
                // Don't validate for other types
                _ => true,
            };

            if !valid {
                tracing::warn!(
                    resource_type = ?resource_type,
                    content_type = %content_type,
                    "Content type mismatch for resource"
                );
            }
        }

        Ok(())
    }

    /// Check if a resource type has high priority
    pub fn is_high_priority(&self, resource_type: ResourceType) -> bool {
        self.high_priority_types.contains(&resource_type)
    }

    /// Load a document (HTML page)
    pub async fn load_document(&self, url: Url) -> NetworkResult<ResourceLoadResult> {
        self.load(url, ResourceType::Document).await
    }

    /// Load a stylesheet (CSS)
    pub async fn load_stylesheet(&self, url: Url) -> NetworkResult<ResourceLoadResult> {
        self.load(url, ResourceType::Stylesheet).await
    }

    /// Load a script (JavaScript)
    pub async fn load_script(&self, url: Url) -> NetworkResult<ResourceLoadResult> {
        self.load(url, ResourceType::Script).await
    }

    /// Load an image
    pub async fn load_image(&self, url: Url) -> NetworkResult<ResourceLoadResult> {
        self.load(url, ResourceType::Image).await
    }

    /// Load a font
    pub async fn load_font(&self, url: Url) -> NetworkResult<ResourceLoadResult> {
        self.load(url, ResourceType::Font).await
    }

    /// Load media (audio/video)
    pub async fn load_media(&self, url: Url) -> NetworkResult<ResourceLoadResult> {
        self.load(url, ResourceType::Media).await
    }
}

// Manual Clone implementation since Semaphore doesn't implement Clone
impl<C: NetworkClient + 'static> Clone for ResourceLoader<C> {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            max_concurrent: Arc::clone(&self.max_concurrent),
            high_priority_types: self.high_priority_types.clone(),
        }
    }
}

/// Builder for ResourceLoader with custom configuration
pub struct ResourceLoaderBuilder<C: NetworkClient + 'static> {
    client: Arc<C>,
    max_concurrent: usize,
    high_priority_types: Vec<ResourceType>,
}

impl<C: NetworkClient + 'static> ResourceLoaderBuilder<C> {
    /// Create a new builder
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            max_concurrent: 6, // Browser default
            high_priority_types: vec![
                ResourceType::Document,
                ResourceType::Stylesheet,
                ResourceType::Script,
            ],
        }
    }

    /// Set maximum concurrent loads
    pub fn max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Set high priority resource types
    pub fn high_priority_types(mut self, types: Vec<ResourceType>) -> Self {
        self.high_priority_types = types;
        self
    }

    /// Build the resource loader
    pub fn build(self) -> ResourceLoader<C> {
        ResourceLoader {
            client: self.client,
            max_concurrent: Arc::new(Semaphore::new(self.max_concurrent)),
            high_priority_types: self.high_priority_types,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::HttpClient;
    use crate::response::StatusCode;

    #[tokio::test]
    async fn test_resource_loader_creation() {
        let client = HttpClient::new().unwrap();
        let loader = ResourceLoader::new(Arc::new(client), 6);

        assert!(loader.is_high_priority(ResourceType::Document));
        assert!(loader.is_high_priority(ResourceType::Script));
        assert!(!loader.is_high_priority(ResourceType::Image));
    }

    #[tokio::test]
    async fn test_build_request_headers() {
        let client = HttpClient::new().unwrap();
        let loader = ResourceLoader::new(Arc::new(client), 6);

        let url = Url::parse("https://example.com/test.js").unwrap();
        let request = loader.build_request(url.clone(), ResourceType::Script).unwrap();

        assert_eq!(request.resource_type, ResourceType::Script);
        assert!(request.headers.contains_key("Accept"));
    }

    #[test]
    fn test_validate_response() {
        let client = HttpClient::new().unwrap();
        let loader = ResourceLoader::new(Arc::new(client), 6);

        let mut response = NetworkResponse::new(
            StatusCode::OK,
            Url::parse("https://example.com/test.css").unwrap(),
        );
        response.headers.insert("content-type".to_string(), "text/css".to_string());

        assert!(loader.validate_response(&response, ResourceType::Stylesheet).is_ok());
    }

    #[test]
    fn test_validate_response_error() {
        let client = HttpClient::new().unwrap();
        let loader = ResourceLoader::new(Arc::new(client), 6);

        let response = NetworkResponse::new(
            StatusCode::NOT_FOUND,
            Url::parse("https://example.com/missing.css").unwrap(),
        );

        assert!(loader.validate_response(&response, ResourceType::Stylesheet).is_err());
    }

    #[tokio::test]
    async fn test_resource_loader_builder() {
        let client = HttpClient::new().unwrap();
        let loader = ResourceLoaderBuilder::new(Arc::new(client))
            .max_concurrent(10)
            .high_priority_types(vec![ResourceType::Document])
            .build();

        assert!(loader.is_high_priority(ResourceType::Document));
        assert!(!loader.is_high_priority(ResourceType::Script));
    }
}
