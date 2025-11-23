//! Protocol response types.

use crate::response::StatusCode;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

/// MIME type constants for common content types.
pub mod mime {
    /// HTML content type.
    pub const HTML: &str = "text/html; charset=utf-8";
    /// Plain text content type.
    pub const TEXT: &str = "text/plain; charset=utf-8";
    /// JSON content type.
    pub const JSON: &str = "application/json";
    /// JavaScript content type.
    pub const JAVASCRIPT: &str = "application/javascript";
    /// CSS content type.
    pub const CSS: &str = "text/css";
    /// SVG content type.
    pub const SVG: &str = "image/svg+xml";
    /// PNG content type.
    pub const PNG: &str = "image/png";
    /// JPEG content type.
    pub const JPEG: &str = "image/jpeg";
    /// GIF content type.
    pub const GIF: &str = "image/gif";
    /// WebP content type.
    pub const WEBP: &str = "image/webp";
    /// ICO content type.
    pub const ICO: &str = "image/x-icon";
    /// PDF content type.
    pub const PDF: &str = "application/pdf";
    /// XML content type.
    pub const XML: &str = "application/xml";
    /// Binary/octet-stream content type.
    pub const BINARY: &str = "application/octet-stream";
    /// WASM content type.
    pub const WASM: &str = "application/wasm";
    /// Font WOFF content type.
    pub const WOFF: &str = "font/woff";
    /// Font WOFF2 content type.
    pub const WOFF2: &str = "font/woff2";
}

/// A response from a protocol handler.
///
/// This is a unified response type that all protocol handlers return,
/// regardless of the underlying protocol (HTTP, file, extension, etc.).
#[derive(Debug, Clone)]
pub struct ProtocolResponse {
    /// HTTP-style status code (200 for success, 404 for not found, etc.).
    pub status: StatusCode,

    /// Response headers as key-value pairs.
    pub headers: HashMap<String, String>,

    /// Response body.
    pub body: Vec<u8>,

    /// The URL that was requested (may differ from final URL after redirects).
    pub url: Url,

    /// MIME type of the response content.
    pub content_type: String,

    /// Time taken to generate the response.
    pub elapsed: Duration,

    /// Whether this response came from cache.
    pub from_cache: bool,

    /// The protocol scheme that handled this request.
    pub handled_by: String,
}

impl ProtocolResponse {
    /// Create a new protocol response.
    pub fn new(url: Url, status: StatusCode, handled_by: impl Into<String>) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
            url,
            content_type: mime::BINARY.to_string(),
            elapsed: Duration::ZERO,
            from_cache: false,
            handled_by: handled_by.into(),
        }
    }

    /// Create a successful response with HTML content.
    pub fn html(url: Url, body: impl Into<String>, handled_by: impl Into<String>) -> Self {
        let body_bytes = body.into().into_bytes();
        Self {
            status: StatusCode::OK,
            headers: HashMap::new(),
            body: body_bytes,
            url,
            content_type: mime::HTML.to_string(),
            elapsed: Duration::ZERO,
            from_cache: false,
            handled_by: handled_by.into(),
        }
    }

    /// Create a successful response with text content.
    pub fn text(url: Url, body: impl Into<String>, handled_by: impl Into<String>) -> Self {
        let body_bytes = body.into().into_bytes();
        Self {
            status: StatusCode::OK,
            headers: HashMap::new(),
            body: body_bytes,
            url,
            content_type: mime::TEXT.to_string(),
            elapsed: Duration::ZERO,
            from_cache: false,
            handled_by: handled_by.into(),
        }
    }

    /// Create a successful response with JSON content.
    pub fn json(url: Url, body: impl Into<String>, handled_by: impl Into<String>) -> Self {
        let body_bytes = body.into().into_bytes();
        Self {
            status: StatusCode::OK,
            headers: HashMap::new(),
            body: body_bytes,
            url,
            content_type: mime::JSON.to_string(),
            elapsed: Duration::ZERO,
            from_cache: false,
            handled_by: handled_by.into(),
        }
    }

    /// Create a successful response with binary content.
    pub fn binary(
        url: Url,
        body: Vec<u8>,
        content_type: impl Into<String>,
        handled_by: impl Into<String>,
    ) -> Self {
        Self {
            status: StatusCode::OK,
            headers: HashMap::new(),
            body,
            url,
            content_type: content_type.into(),
            elapsed: Duration::ZERO,
            from_cache: false,
            handled_by: handled_by.into(),
        }
    }

    /// Create a not found (404) response.
    pub fn not_found(url: Url, handled_by: impl Into<String>) -> Self {
        Self::new(url, StatusCode::NOT_FOUND, handled_by)
    }

    /// Create a forbidden (403) response.
    pub fn forbidden(url: Url, handled_by: impl Into<String>) -> Self {
        Self::new(url, StatusCode::FORBIDDEN, handled_by)
    }

    /// Set response headers.
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    /// Add a single header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the response body.
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    /// Set the content type.
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = content_type.into();
        self
    }

    /// Set the elapsed time.
    pub fn with_elapsed(mut self, elapsed: Duration) -> Self {
        self.elapsed = elapsed;
        self
    }

    /// Mark as coming from cache.
    pub fn with_from_cache(mut self, from_cache: bool) -> Self {
        self.from_cache = from_cache;
        self
    }

    /// Check if the response indicates success (2xx status).
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Check if the response indicates an error.
    pub fn is_error(&self) -> bool {
        self.status.is_error()
    }

    /// Check if the response indicates not found.
    pub fn is_not_found(&self) -> bool {
        self.status.as_u16() == 404
    }

    /// Get the body as a UTF-8 string.
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    /// Get the content length.
    pub fn content_length(&self) -> usize {
        self.body.len()
    }

    /// Get a header value (case-insensitive).
    pub fn header(&self, key: &str) -> Option<&String> {
        let key_lower = key.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == key_lower)
            .map(|(_, v)| v)
    }
}

/// Guess MIME type from file extension.
pub fn guess_mime_type(path: &str) -> &'static str {
    let extension = path.rsplit('.').next().unwrap_or("").to_lowercase();

    match extension.as_str() {
        "html" | "htm" => mime::HTML,
        "txt" => mime::TEXT,
        "json" => mime::JSON,
        "js" | "mjs" => mime::JAVASCRIPT,
        "css" => mime::CSS,
        "svg" => mime::SVG,
        "png" => mime::PNG,
        "jpg" | "jpeg" => mime::JPEG,
        "gif" => mime::GIF,
        "webp" => mime::WEBP,
        "ico" => mime::ICO,
        "pdf" => mime::PDF,
        "xml" => mime::XML,
        "wasm" => mime::WASM,
        "woff" => mime::WOFF,
        "woff2" => mime::WOFF2,
        _ => mime::BINARY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_url() -> Url {
        Url::parse("test://example").unwrap()
    }

    #[test]
    fn test_protocol_response_new() {
        let response = ProtocolResponse::new(test_url(), StatusCode::OK, "test");
        assert!(response.is_success());
        assert!(!response.is_error());
        assert!(response.body.is_empty());
        assert_eq!(response.handled_by, "test");
    }

    #[test]
    fn test_protocol_response_html() {
        let response = ProtocolResponse::html(test_url(), "<h1>Hello</h1>", "test");
        assert!(response.is_success());
        assert_eq!(response.content_type, mime::HTML);
        assert_eq!(response.text().unwrap(), "<h1>Hello</h1>");
    }

    #[test]
    fn test_protocol_response_text() {
        let response = ProtocolResponse::text(test_url(), "Hello, World!", "test");
        assert!(response.is_success());
        assert_eq!(response.content_type, mime::TEXT);
        assert_eq!(response.text().unwrap(), "Hello, World!");
    }

    #[test]
    fn test_protocol_response_json() {
        let response = ProtocolResponse::json(test_url(), r#"{"key": "value"}"#, "test");
        assert!(response.is_success());
        assert_eq!(response.content_type, mime::JSON);
    }

    #[test]
    fn test_protocol_response_binary() {
        let data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes
        let response = ProtocolResponse::binary(test_url(), data.clone(), mime::PNG, "test");
        assert!(response.is_success());
        assert_eq!(response.content_type, mime::PNG);
        assert_eq!(response.body, data);
    }

    #[test]
    fn test_protocol_response_not_found() {
        let response = ProtocolResponse::not_found(test_url(), "test");
        assert!(response.is_not_found());
        assert!(response.is_error());
    }

    #[test]
    fn test_protocol_response_forbidden() {
        let response = ProtocolResponse::forbidden(test_url(), "test");
        assert!(response.is_error());
        assert!(!response.is_success());
    }

    #[test]
    fn test_protocol_response_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom".to_string(), "value".to_string());

        let response = ProtocolResponse::new(test_url(), StatusCode::OK, "test")
            .with_headers(headers)
            .with_header("X-Another", "another");

        assert_eq!(response.header("x-custom"), Some(&"value".to_string()));
        assert_eq!(response.header("X-Another"), Some(&"another".to_string()));
    }

    #[test]
    fn test_protocol_response_content_length() {
        let response = ProtocolResponse::text(test_url(), "Hello", "test");
        assert_eq!(response.content_length(), 5);
    }

    #[test]
    fn test_guess_mime_type() {
        assert_eq!(guess_mime_type("index.html"), mime::HTML);
        assert_eq!(guess_mime_type("page.htm"), mime::HTML);
        assert_eq!(guess_mime_type("style.css"), mime::CSS);
        assert_eq!(guess_mime_type("script.js"), mime::JAVASCRIPT);
        assert_eq!(guess_mime_type("module.mjs"), mime::JAVASCRIPT);
        assert_eq!(guess_mime_type("data.json"), mime::JSON);
        assert_eq!(guess_mime_type("image.png"), mime::PNG);
        assert_eq!(guess_mime_type("photo.jpg"), mime::JPEG);
        assert_eq!(guess_mime_type("photo.jpeg"), mime::JPEG);
        assert_eq!(guess_mime_type("animation.gif"), mime::GIF);
        assert_eq!(guess_mime_type("modern.webp"), mime::WEBP);
        assert_eq!(guess_mime_type("icon.svg"), mime::SVG);
        assert_eq!(guess_mime_type("favicon.ico"), mime::ICO);
        assert_eq!(guess_mime_type("document.pdf"), mime::PDF);
        assert_eq!(guess_mime_type("data.xml"), mime::XML);
        assert_eq!(guess_mime_type("app.wasm"), mime::WASM);
        assert_eq!(guess_mime_type("font.woff"), mime::WOFF);
        assert_eq!(guess_mime_type("font.woff2"), mime::WOFF2);
        assert_eq!(guess_mime_type("unknown.xyz"), mime::BINARY);
        assert_eq!(guess_mime_type("no_extension"), mime::BINARY);
    }

    #[test]
    fn test_guess_mime_type_case_insensitive() {
        assert_eq!(guess_mime_type("file.HTML"), mime::HTML);
        assert_eq!(guess_mime_type("file.Json"), mime::JSON);
        assert_eq!(guess_mime_type("file.PNG"), mime::PNG);
    }
}
