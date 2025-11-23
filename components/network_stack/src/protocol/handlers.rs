//! Protocol handler implementations.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::fs;
use url::Url;

use super::response::{guess_mime_type, ProtocolResponse};
use super::types::{ProtocolError, ProtocolHandler, ProtocolResult};
use crate::{NetworkClient, NetworkRequest};

/// Handler for HTTP and HTTPS protocols.
///
/// Delegates requests to the underlying network client.
pub struct HttpProtocolHandler<C: NetworkClient> {
    client: Arc<C>,
}

impl<C: NetworkClient> HttpProtocolHandler<C> {
    /// Create a new HTTP protocol handler.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: NetworkClient + Send + Sync + 'static> ProtocolHandler for HttpProtocolHandler<C> {
    fn scheme(&self) -> &str {
        "http"
    }

    fn schemes(&self) -> Vec<&str> {
        vec!["http", "https"]
    }

    fn name(&self) -> &str {
        "HTTP/HTTPS"
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    async fn handle(&self, url: &Url) -> ProtocolResult<ProtocolResponse> {
        let start = Instant::now();

        let request = NetworkRequest::get(url.clone());
        let network_response = self.client.fetch(request).await?;

        let mut headers = HashMap::new();
        for (key, value) in &network_response.headers {
            headers.insert(key.clone(), value.clone());
        }

        let content_type = network_response
            .content_type
            .clone()
            .unwrap_or_else(|| guess_mime_type(url.path()).to_string());

        Ok(ProtocolResponse {
            status: network_response.status,
            headers,
            body: network_response.body,
            url: network_response.url,
            content_type,
            elapsed: start.elapsed(),
            from_cache: network_response.cache_status != crate::response::CacheStatus::Miss,
            handled_by: "http".to_string(),
        })
    }
}

/// Handler for file:// protocol.
///
/// Provides access to local files with security restrictions.
pub struct FileProtocolHandler {
    /// Optional base directory for sandboxing file access.
    base_dir: Option<PathBuf>,
    /// Whether to allow access outside base directory.
    allow_outside_base: bool,
}

impl FileProtocolHandler {
    /// Create a new file protocol handler without restrictions.
    pub fn new() -> Self {
        Self {
            base_dir: None,
            allow_outside_base: true,
        }
    }

    /// Create a new file protocol handler sandboxed to a base directory.
    pub fn sandboxed(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: Some(base_dir.into()),
            allow_outside_base: false,
        }
    }

    /// Set whether to allow access outside the base directory.
    pub fn with_allow_outside_base(mut self, allow: bool) -> Self {
        self.allow_outside_base = allow;
        self
    }

    /// Validate and resolve a file path from a URL.
    fn resolve_path(&self, url: &Url) -> ProtocolResult<PathBuf> {
        // Get the path from the URL
        let url_path = url.path();

        // On Windows, file URLs have paths like /C:/path/to/file
        // We need to strip the leading slash for absolute paths
        #[cfg(windows)]
        let path_str = if url_path.len() >= 3
            && url_path.chars().nth(0) == Some('/')
            && url_path.chars().nth(2) == Some(':')
        {
            &url_path[1..]
        } else {
            url_path
        };

        #[cfg(not(windows))]
        let path_str = url_path;

        // Decode percent-encoded characters
        let decoded = percent_decode(path_str);
        let path = PathBuf::from(decoded);

        // Security check: prevent path traversal
        if self.contains_traversal(&path) {
            return Err(ProtocolError::security_violation(
                "Path traversal not allowed",
            ));
        }

        // If sandboxed, ensure the path is within base directory
        if let Some(ref base_dir) = self.base_dir {
            if !self.allow_outside_base {
                // Canonicalize paths for comparison
                let canonical_base = base_dir.canonicalize().map_err(|_| {
                    ProtocolError::invalid_url("Cannot resolve base directory")
                })?;

                // For non-existent files, we need to check if the parent exists
                let canonical_path = if path.exists() {
                    path.canonicalize().map_err(|_| {
                        ProtocolError::file_not_found(&path)
                    })?
                } else {
                    // For non-existent files, check parent directory
                    if let Some(parent) = path.parent() {
                        if parent.exists() {
                            let canonical_parent = parent.canonicalize().map_err(|_| {
                                ProtocolError::file_not_found(&path)
                            })?;
                            canonical_parent.join(path.file_name().unwrap_or_default())
                        } else {
                            return Err(ProtocolError::file_not_found(&path));
                        }
                    } else {
                        return Err(ProtocolError::file_not_found(&path));
                    }
                };

                if !canonical_path.starts_with(&canonical_base) {
                    return Err(ProtocolError::security_violation(
                        "Access outside sandbox not allowed",
                    ));
                }
            }
        }

        Ok(path)
    }

    /// Check if a path contains traversal sequences.
    fn contains_traversal(&self, path: &Path) -> bool {
        for component in path.components() {
            if let std::path::Component::ParentDir = component {
                return true;
            }
        }
        false
    }
}

impl Default for FileProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Percent-decode a string.
fn percent_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            result.push('%');
            result.push_str(&hex);
        } else {
            result.push(c);
        }
    }

    result
}

#[async_trait]
impl ProtocolHandler for FileProtocolHandler {
    fn scheme(&self) -> &str {
        "file"
    }

    fn name(&self) -> &str {
        "Local File"
    }

    async fn handle(&self, url: &Url) -> ProtocolResult<ProtocolResponse> {
        let start = Instant::now();
        let path = self.resolve_path(url)?;

        // Check if file exists
        if !path.exists() {
            return Err(ProtocolError::file_not_found(&path));
        }

        // Check read permissions
        let metadata = fs::metadata(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                ProtocolError::access_denied(&path)
            } else {
                ProtocolError::from(e)
            }
        })?;

        // Don't serve directories directly
        if metadata.is_dir() {
            return Err(ProtocolError::invalid_url("Cannot serve directory"));
        }

        // Read file contents
        let body = fs::read(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                ProtocolError::access_denied(&path)
            } else {
                ProtocolError::from(e)
            }
        })?;

        // Determine content type
        let content_type = guess_mime_type(path.to_string_lossy().as_ref());

        Ok(ProtocolResponse::binary(url.clone(), body, content_type, "file")
            .with_elapsed(start.elapsed()))
    }
}

/// Extension resource resolver interface.
///
/// Implement this trait to provide extension resource resolution.
pub trait ExtensionResolver: Send + Sync {
    /// Get the base path for an extension.
    fn get_extension_path(&self, extension_id: &str) -> Option<PathBuf>;

    /// Check if an extension exists.
    fn extension_exists(&self, extension_id: &str) -> bool {
        self.get_extension_path(extension_id).is_some()
    }
}

/// Simple in-memory extension resolver for testing.
#[derive(Default)]
pub struct InMemoryExtensionResolver {
    extensions: HashMap<String, PathBuf>,
}

impl InMemoryExtensionResolver {
    /// Create a new empty resolver.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an extension.
    pub fn register(&mut self, extension_id: impl Into<String>, path: impl Into<PathBuf>) {
        self.extensions.insert(extension_id.into(), path.into());
    }
}

impl ExtensionResolver for InMemoryExtensionResolver {
    fn get_extension_path(&self, extension_id: &str) -> Option<PathBuf> {
        self.extensions.get(extension_id).cloned()
    }
}

/// Handler for extension:// protocol.
///
/// Provides access to browser extension resources.
pub struct ExtensionProtocolHandler<R: ExtensionResolver> {
    resolver: Arc<R>,
}

impl<R: ExtensionResolver> ExtensionProtocolHandler<R> {
    /// Create a new extension protocol handler.
    pub fn new(resolver: Arc<R>) -> Self {
        Self { resolver }
    }
}

#[async_trait]
impl<R: ExtensionResolver + 'static> ProtocolHandler for ExtensionProtocolHandler<R> {
    fn scheme(&self) -> &str {
        "extension"
    }

    fn name(&self) -> &str {
        "Extension Resources"
    }

    async fn handle(&self, url: &Url) -> ProtocolResult<ProtocolResponse> {
        let start = Instant::now();

        // Parse extension:// URL
        // Format: extension://<extension-id>/<path>
        let extension_id = url.host_str().ok_or_else(|| {
            ProtocolError::invalid_url("Extension URL must have extension ID as host")
        })?;

        let resource_path = url.path();
        let resource_path = resource_path.strip_prefix('/').unwrap_or(resource_path);

        // Get extension base path
        let base_path = self
            .resolver
            .get_extension_path(extension_id)
            .ok_or_else(|| ProtocolError::extension_not_found(extension_id))?;

        // Construct full path
        let full_path = base_path.join(resource_path);

        // Security: prevent path traversal
        let canonical_base = base_path.canonicalize().map_err(|_| {
            ProtocolError::extension_not_found(extension_id)
        })?;

        let canonical_path = full_path.canonicalize().map_err(|_| {
            ProtocolError::extension_resource_not_found(extension_id, resource_path)
        })?;

        if !canonical_path.starts_with(&canonical_base) {
            return Err(ProtocolError::security_violation(
                "Extension path traversal not allowed",
            ));
        }

        // Read file
        let body = fs::read(&canonical_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ProtocolError::extension_resource_not_found(extension_id, resource_path)
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                ProtocolError::access_denied(&canonical_path)
            } else {
                ProtocolError::from(e)
            }
        })?;

        let content_type = guess_mime_type(resource_path);

        Ok(ProtocolResponse::binary(url.clone(), body, content_type, "extension")
            .with_elapsed(start.elapsed()))
    }
}

/// Internal browser page content provider.
pub trait InternalPageProvider: Send + Sync {
    /// Get the content for an internal page.
    fn get_page(&self, page_name: &str) -> Option<(String, String)>; // (content_type, body)

    /// List available internal pages.
    fn list_pages(&self) -> Vec<&str>;
}

/// Simple in-memory internal page provider.
#[derive(Default)]
pub struct InMemoryInternalPageProvider {
    pages: HashMap<String, (String, String)>,
}

impl InMemoryInternalPageProvider {
    /// Create a new empty provider.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an internal page.
    pub fn register(
        &mut self,
        page_name: impl Into<String>,
        content_type: impl Into<String>,
        body: impl Into<String>,
    ) {
        self.pages
            .insert(page_name.into(), (content_type.into(), body.into()));
    }

    /// Create a provider with default browser pages.
    pub fn with_defaults() -> Self {
        let mut provider = Self::new();

        // Register default internal pages
        provider.register(
            "settings",
            "text/html; charset=utf-8",
            r#"<!DOCTYPE html>
<html>
<head><title>Browser Settings</title></head>
<body>
<h1>Browser Settings</h1>
<p>Settings page content goes here.</p>
</body>
</html>"#,
        );

        provider.register(
            "history",
            "text/html; charset=utf-8",
            r#"<!DOCTYPE html>
<html>
<head><title>Browsing History</title></head>
<body>
<h1>Browsing History</h1>
<p>History content goes here.</p>
</body>
</html>"#,
        );

        provider.register(
            "bookmarks",
            "text/html; charset=utf-8",
            r#"<!DOCTYPE html>
<html>
<head><title>Bookmarks</title></head>
<body>
<h1>Bookmarks</h1>
<p>Bookmarks content goes here.</p>
</body>
</html>"#,
        );

        provider.register(
            "newtab",
            "text/html; charset=utf-8",
            r#"<!DOCTYPE html>
<html>
<head><title>New Tab</title></head>
<body>
<h1>New Tab</h1>
<p>Welcome to a new tab.</p>
</body>
</html>"#,
        );

        provider.register(
            "about",
            "text/html; charset=utf-8",
            r#"<!DOCTYPE html>
<html>
<head><title>About Browser</title></head>
<body>
<h1>About Corten Browser</h1>
<p>A modern browser shell.</p>
</body>
</html>"#,
        );

        provider
    }
}

impl InternalPageProvider for InMemoryInternalPageProvider {
    fn get_page(&self, page_name: &str) -> Option<(String, String)> {
        self.pages.get(page_name).cloned()
    }

    fn list_pages(&self) -> Vec<&str> {
        self.pages.keys().map(|s| s.as_str()).collect()
    }
}

/// Handler for browser:// internal pages.
///
/// Provides access to internal browser UI pages like settings, history, etc.
pub struct InternalProtocolHandler<P: InternalPageProvider> {
    provider: Arc<P>,
    scheme_name: String,
}

impl<P: InternalPageProvider> InternalProtocolHandler<P> {
    /// Create a new internal protocol handler with the default "browser" scheme.
    pub fn new(provider: Arc<P>) -> Self {
        Self {
            provider,
            scheme_name: "browser".to_string(),
        }
    }

    /// Create a new internal protocol handler with a custom scheme (e.g., "chrome").
    pub fn with_scheme(provider: Arc<P>, scheme: impl Into<String>) -> Self {
        Self {
            provider,
            scheme_name: scheme.into(),
        }
    }
}

#[async_trait]
impl<P: InternalPageProvider + 'static> ProtocolHandler for InternalProtocolHandler<P> {
    fn scheme(&self) -> &str {
        &self.scheme_name
    }

    fn name(&self) -> &str {
        "Internal Pages"
    }

    async fn handle(&self, url: &Url) -> ProtocolResult<ProtocolResponse> {
        let start = Instant::now();

        // Parse browser://page-name or browser://page-name/
        let page_name = url.host_str().ok_or_else(|| {
            ProtocolError::invalid_url("Internal URL must have page name as host")
        })?;

        let (content_type, body) = self
            .provider
            .get_page(page_name)
            .ok_or_else(|| ProtocolError::internal_page_not_found(page_name))?;

        Ok(ProtocolResponse::binary(
            url.clone(),
            body.into_bytes(),
            content_type,
            &self.scheme_name,
        )
        .with_elapsed(start.elapsed()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_decode() {
        assert_eq!(percent_decode("hello%20world"), "hello world");
        assert_eq!(percent_decode("path%2Fto%2Ffile"), "path/to/file");
        assert_eq!(percent_decode("normal"), "normal");
        assert_eq!(percent_decode("%"), "%");
        assert_eq!(percent_decode("%2"), "%2");
    }

    #[test]
    fn test_file_handler_path_traversal_detection() {
        let handler = FileProtocolHandler::new();

        let safe_path = Path::new("/home/user/file.txt");
        assert!(!handler.contains_traversal(safe_path));

        let unsafe_path = Path::new("/home/user/../other/file.txt");
        assert!(handler.contains_traversal(unsafe_path));
    }

    #[test]
    fn test_in_memory_extension_resolver() {
        let mut resolver = InMemoryExtensionResolver::new();
        resolver.register("my-extension", "/path/to/extension");

        assert!(resolver.extension_exists("my-extension"));
        assert!(!resolver.extension_exists("unknown"));
        assert_eq!(
            resolver.get_extension_path("my-extension"),
            Some(PathBuf::from("/path/to/extension"))
        );
    }

    #[test]
    fn test_in_memory_internal_page_provider() {
        let provider = InMemoryInternalPageProvider::with_defaults();

        assert!(provider.get_page("settings").is_some());
        assert!(provider.get_page("history").is_some());
        assert!(provider.get_page("bookmarks").is_some());
        assert!(provider.get_page("newtab").is_some());
        assert!(provider.get_page("about").is_some());
        assert!(provider.get_page("nonexistent").is_none());

        let pages = provider.list_pages();
        assert!(pages.contains(&"settings"));
    }

    #[tokio::test]
    async fn test_internal_handler_returns_page() {
        let provider = Arc::new(InMemoryInternalPageProvider::with_defaults());
        let handler = InternalProtocolHandler::new(provider);

        let url = Url::parse("browser://settings").unwrap();
        let response = handler.handle(&url).await.unwrap();

        assert!(response.is_success());
        assert!(response.text().unwrap().contains("Browser Settings"));
    }

    #[tokio::test]
    async fn test_internal_handler_not_found() {
        let provider = Arc::new(InMemoryInternalPageProvider::new());
        let handler = InternalProtocolHandler::new(provider);

        let url = Url::parse("browser://nonexistent").unwrap();
        let result = handler.handle(&url).await;

        assert!(matches!(
            result,
            Err(ProtocolError::InternalPageNotFound { .. })
        ));
    }

    #[tokio::test]
    async fn test_internal_handler_custom_scheme() {
        let provider = Arc::new(InMemoryInternalPageProvider::with_defaults());
        let handler = InternalProtocolHandler::with_scheme(provider, "chrome");

        assert_eq!(handler.scheme(), "chrome");

        let url = Url::parse("chrome://settings").unwrap();
        let response = handler.handle(&url).await.unwrap();
        assert!(response.is_success());
        assert_eq!(response.handled_by, "chrome");
    }

    #[test]
    fn test_http_handler_schemes() {
        use crate::HttpClient;

        let client = Arc::new(HttpClient::new().unwrap());
        let handler = HttpProtocolHandler::new(client);

        assert_eq!(handler.scheme(), "http");
        assert_eq!(handler.schemes(), vec!["http", "https"]);

        let http_url = Url::parse("http://example.com").unwrap();
        let https_url = Url::parse("https://example.com").unwrap();
        let ftp_url = Url::parse("ftp://example.com").unwrap();

        assert!(handler.can_handle(&http_url));
        assert!(handler.can_handle(&https_url));
        assert!(!handler.can_handle(&ftp_url));
    }
}
