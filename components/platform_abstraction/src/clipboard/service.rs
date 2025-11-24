//! Clipboard service trait and cross-platform implementation
//!
//! This module provides the ClipboardService trait and a cross-platform
//! implementation using the arboard crate.

use super::types::{ClipboardContent, ClipboardFormat, ImageData};
use std::sync::Mutex;
use thiserror::Error;

/// Errors that can occur during clipboard operations
#[derive(Debug, Error)]
pub enum ClipboardError {
    /// Clipboard is not available on this platform
    #[error("clipboard not available: {0}")]
    NotAvailable(String),

    /// Failed to read from clipboard
    #[error("failed to read clipboard: {0}")]
    ReadFailed(String),

    /// Failed to write to clipboard
    #[error("failed to write to clipboard: {0}")]
    WriteFailed(String),

    /// The requested format is not available in the clipboard
    #[error("format not available: {0:?}")]
    FormatNotAvailable(ClipboardFormat),

    /// Image format conversion failed
    #[error("image conversion failed: {0}")]
    ImageConversionFailed(String),

    /// Clipboard is currently locked by another operation
    #[error("clipboard is locked")]
    Locked,

    /// Operation timed out
    #[error("clipboard operation timed out")]
    Timeout,
}

/// Result type for clipboard operations
pub type ClipboardResult<T> = Result<T, ClipboardError>;

/// Trait for clipboard services
///
/// This trait defines the interface for reading from and writing to the system clipboard.
/// Implementations should handle platform-specific details.
///
/// # Thread Safety
///
/// Implementations must be thread-safe (`Send + Sync`). The system clipboard is a shared
/// resource, so implementations should handle concurrent access appropriately.
pub trait ClipboardService: Send + Sync {
    /// Read plain text from the clipboard
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The text content from the clipboard
    /// * `Err(ClipboardError::FormatNotAvailable)` - If clipboard doesn't contain text
    /// * `Err(ClipboardError)` - For other errors
    fn read_text(&self) -> ClipboardResult<String>;

    /// Write plain text to the clipboard
    ///
    /// # Arguments
    ///
    /// * `text` - The text to write to the clipboard
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Text was successfully written
    /// * `Err(ClipboardError)` - If writing failed
    fn write_text(&self, text: &str) -> ClipboardResult<()>;

    /// Read HTML content from the clipboard
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The HTML content from the clipboard
    /// * `Err(ClipboardError::FormatNotAvailable)` - If clipboard doesn't contain HTML
    /// * `Err(ClipboardError)` - For other errors
    fn read_html(&self) -> ClipboardResult<String>;

    /// Write HTML content to the clipboard
    ///
    /// This also sets the plain text version with the HTML stripped.
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML to write to the clipboard
    ///
    /// # Returns
    ///
    /// * `Ok(())` - HTML was successfully written
    /// * `Err(ClipboardError)` - If writing failed
    fn write_html(&self, html: &str) -> ClipboardResult<()>;

    /// Read image data from the clipboard
    ///
    /// # Returns
    ///
    /// * `Ok(ImageData)` - The image data from the clipboard (RGBA format)
    /// * `Err(ClipboardError::FormatNotAvailable)` - If clipboard doesn't contain an image
    /// * `Err(ClipboardError)` - For other errors
    fn read_image(&self) -> ClipboardResult<ImageData>;

    /// Write image data to the clipboard
    ///
    /// # Arguments
    ///
    /// * `image` - The image data to write (RGBA format)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Image was successfully written
    /// * `Err(ClipboardError)` - If writing failed
    fn write_image(&self, image: &ImageData) -> ClipboardResult<()>;

    /// Clear all content from the clipboard
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Clipboard was successfully cleared
    /// * `Err(ClipboardError)` - If clearing failed
    fn clear(&self) -> ClipboardResult<()>;

    /// Check if the clipboard contains text
    ///
    /// This is a quick check without reading the full content.
    fn has_text(&self) -> bool;

    /// Check if the clipboard contains an image
    ///
    /// This is a quick check without reading the full content.
    fn has_image(&self) -> bool;

    /// Check if the clipboard contains HTML
    ///
    /// This is a quick check without reading the full content.
    fn has_html(&self) -> bool {
        // Default implementation tries to read HTML
        self.read_html().is_ok()
    }

    /// Get the current clipboard content (auto-detecting format)
    ///
    /// This tries to read content in order of preference: Image, HTML, Text.
    fn read_content(&self) -> ClipboardResult<ClipboardContent> {
        // Try image first
        if let Ok(image) = self.read_image() {
            return Ok(ClipboardContent::Image(image));
        }

        // Try HTML
        if let Ok(html) = self.read_html() {
            return Ok(ClipboardContent::Html(html));
        }

        // Try text
        if let Ok(text) = self.read_text() {
            return Ok(ClipboardContent::Text(text));
        }

        Ok(ClipboardContent::Empty)
    }

    /// Check if clipboard operations are supported on this platform
    fn is_supported(&self) -> bool {
        true
    }
}

/// Cross-platform clipboard service implementation
///
/// This service uses the arboard crate for cross-platform clipboard support
/// on Linux (X11/Wayland), Windows, and macOS.
pub struct SystemClipboardService {
    /// Inner clipboard handle protected by mutex for thread safety
    clipboard: Mutex<Option<arboard::Clipboard>>,
}

impl SystemClipboardService {
    /// Create a new clipboard service
    ///
    /// # Returns
    ///
    /// * `Ok(SystemClipboardService)` - Successfully created service
    /// * `Err(ClipboardError::NotAvailable)` - If clipboard is not available
    pub fn new() -> ClipboardResult<Self> {
        let clipboard = arboard::Clipboard::new()
            .map_err(|e| ClipboardError::NotAvailable(e.to_string()))?;

        Ok(Self {
            clipboard: Mutex::new(Some(clipboard)),
        })
    }

    /// Create a new clipboard service, returning None if not available
    pub fn try_new() -> Option<Self> {
        Self::new().ok()
    }

    /// Execute a closure with the clipboard handle
    fn with_clipboard<F, T>(&self, f: F) -> ClipboardResult<T>
    where
        F: FnOnce(&mut arboard::Clipboard) -> ClipboardResult<T>,
    {
        let mut guard = self
            .clipboard
            .lock()
            .map_err(|_| ClipboardError::Locked)?;

        let clipboard = guard
            .as_mut()
            .ok_or_else(|| ClipboardError::NotAvailable("clipboard not initialized".into()))?;

        f(clipboard)
    }
}

impl Default for SystemClipboardService {
    fn default() -> Self {
        Self::new().expect("Failed to create clipboard service")
    }
}

impl ClipboardService for SystemClipboardService {
    fn read_text(&self) -> ClipboardResult<String> {
        self.with_clipboard(|clipboard| {
            clipboard
                .get_text()
                .map_err(|e| match e {
                    arboard::Error::ContentNotAvailable => {
                        ClipboardError::FormatNotAvailable(ClipboardFormat::Text)
                    }
                    _ => ClipboardError::ReadFailed(e.to_string()),
                })
        })
    }

    fn write_text(&self, text: &str) -> ClipboardResult<()> {
        self.with_clipboard(|clipboard| {
            clipboard
                .set_text(text.to_string())
                .map_err(|e| ClipboardError::WriteFailed(e.to_string()))
        })
    }

    fn read_html(&self) -> ClipboardResult<String> {
        // Note: arboard 3.x doesn't support reading HTML directly.
        // We return the format not available error.
        // Future versions may add HTML reading support.
        Err(ClipboardError::FormatNotAvailable(ClipboardFormat::Html))
    }

    fn write_html(&self, html: &str) -> ClipboardResult<()> {
        self.with_clipboard(|clipboard| {
            // Strip HTML tags for plain text fallback
            let plain_text = strip_html_tags(html);

            clipboard
                .set_html(html.to_string(), Some(plain_text))
                .map_err(|e| ClipboardError::WriteFailed(e.to_string()))
        })
    }

    fn read_image(&self) -> ClipboardResult<ImageData> {
        self.with_clipboard(|clipboard| {
            let img = clipboard.get_image().map_err(|e| match e {
                arboard::Error::ContentNotAvailable => {
                    ClipboardError::FormatNotAvailable(ClipboardFormat::Image)
                }
                _ => ClipboardError::ReadFailed(e.to_string()),
            })?;

            Ok(ImageData {
                bytes: img.bytes.into_owned(),
                width: img.width,
                height: img.height,
            })
        })
    }

    fn write_image(&self, image: &ImageData) -> ClipboardResult<()> {
        self.with_clipboard(|clipboard| {
            let img = arboard::ImageData {
                bytes: std::borrow::Cow::Borrowed(&image.bytes),
                width: image.width,
                height: image.height,
            };

            clipboard
                .set_image(img)
                .map_err(|e| ClipboardError::WriteFailed(e.to_string()))
        })
    }

    fn clear(&self) -> ClipboardResult<()> {
        self.with_clipboard(|clipboard| {
            clipboard
                .clear()
                .map_err(|e| ClipboardError::WriteFailed(e.to_string()))
        })
    }

    fn has_text(&self) -> bool {
        self.read_text().is_ok()
    }

    fn has_image(&self) -> bool {
        self.read_image().is_ok()
    }

    fn has_html(&self) -> bool {
        // Note: arboard 3.x doesn't support reading HTML directly
        false
    }

    fn is_supported(&self) -> bool {
        cfg!(any(
            target_os = "linux",
            target_os = "windows",
            target_os = "macos"
        ))
    }
}

/// Strip HTML tags for plain text conversion
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    // Decode common HTML entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

/// Check if clipboard is supported on the current platform
pub fn clipboard_supported() -> bool {
    cfg!(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html_tags_basic() {
        assert_eq!(strip_html_tags("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html_tags("<b>Bold</b> text"), "Bold text");
        assert_eq!(
            strip_html_tags("<div><span>Nested</span></div>"),
            "Nested"
        );
    }

    #[test]
    fn test_strip_html_tags_entities() {
        assert_eq!(strip_html_tags("&amp;"), "&");
        assert_eq!(strip_html_tags("&lt;tag&gt;"), "<tag>");
        assert_eq!(strip_html_tags("&quot;quoted&quot;"), "\"quoted\"");
        assert_eq!(strip_html_tags("non&nbsp;breaking"), "non breaking");
    }

    #[test]
    fn test_strip_html_tags_empty() {
        assert_eq!(strip_html_tags(""), "");
        assert_eq!(strip_html_tags("<br/>"), "");
    }

    #[test]
    fn test_strip_html_tags_no_tags() {
        assert_eq!(strip_html_tags("Plain text"), "Plain text");
    }

    #[test]
    fn test_clipboard_error_display() {
        let err = ClipboardError::NotAvailable("test".to_string());
        assert_eq!(err.to_string(), "clipboard not available: test");

        let err = ClipboardError::ReadFailed("read error".to_string());
        assert_eq!(err.to_string(), "failed to read clipboard: read error");

        let err = ClipboardError::WriteFailed("write error".to_string());
        assert_eq!(err.to_string(), "failed to write to clipboard: write error");

        let err = ClipboardError::FormatNotAvailable(ClipboardFormat::Text);
        assert_eq!(err.to_string(), "format not available: Text");

        let err = ClipboardError::ImageConversionFailed("conversion".to_string());
        assert_eq!(err.to_string(), "image conversion failed: conversion");

        let err = ClipboardError::Locked;
        assert_eq!(err.to_string(), "clipboard is locked");

        let err = ClipboardError::Timeout;
        assert_eq!(err.to_string(), "clipboard operation timed out");
    }

    #[test]
    fn test_clipboard_supported() {
        let expected = cfg!(any(
            target_os = "linux",
            target_os = "windows",
            target_os = "macos"
        ));
        assert_eq!(clipboard_supported(), expected);
    }

    // Integration tests - these interact with the real clipboard
    // Ignored by default to avoid affecting the user's clipboard
    #[test]
    #[ignore]
    fn test_clipboard_text_roundtrip() {
        let service = SystemClipboardService::new().expect("clipboard should be available");
        let test_text = "Test clipboard text";

        service.write_text(test_text).expect("write should succeed");
        let read_text = service.read_text().expect("read should succeed");

        assert_eq!(read_text, test_text);
    }

    #[test]
    #[ignore]
    fn test_clipboard_html_write() {
        // Note: arboard 3.x supports writing HTML but not reading it back
        let service = SystemClipboardService::new().expect("clipboard should be available");
        let test_html = "<p>Test <b>HTML</b> content</p>";

        // Writing HTML should succeed and set plain text fallback
        service.write_html(test_html).expect("write should succeed");

        // We can only verify the plain text fallback is set
        let text = service.read_text().expect("text fallback should be readable");
        assert!(text.contains("HTML") || text.contains("Test"));
    }

    #[test]
    #[ignore]
    fn test_clipboard_image_roundtrip() {
        let service = SystemClipboardService::new().expect("clipboard should be available");

        // Create a 2x2 red image (RGBA)
        let red_pixel = [255u8, 0, 0, 255];
        let bytes: Vec<u8> = red_pixel.iter().cloned().cycle().take(16).collect();
        let test_image = ImageData::new(bytes.clone(), 2, 2);

        service
            .write_image(&test_image)
            .expect("write should succeed");
        let read_image = service.read_image().expect("read should succeed");

        assert_eq!(read_image.width, 2);
        assert_eq!(read_image.height, 2);
        assert_eq!(read_image.bytes.len(), 16);
    }

    #[test]
    #[ignore]
    fn test_clipboard_clear() {
        let service = SystemClipboardService::new().expect("clipboard should be available");

        service
            .write_text("Some text")
            .expect("write should succeed");
        service.clear().expect("clear should succeed");

        // After clear, text should not be available
        assert!(!service.has_text());
    }

    #[test]
    #[ignore]
    fn test_clipboard_has_text() {
        let service = SystemClipboardService::new().expect("clipboard should be available");

        service
            .write_text("Test text")
            .expect("write should succeed");
        assert!(service.has_text());
    }

    #[test]
    #[ignore]
    fn test_clipboard_read_content_text() {
        let service = SystemClipboardService::new().expect("clipboard should be available");

        service
            .write_text("Plain text")
            .expect("write should succeed");
        let content = service.read_content().expect("read should succeed");

        assert!(content.is_text());
        assert_eq!(content.as_text(), Some("Plain text"));
    }

    #[test]
    fn test_system_clipboard_is_supported() {
        // This test doesn't require clipboard access
        let expected = cfg!(any(
            target_os = "linux",
            target_os = "windows",
            target_os = "macos"
        ));

        // We can't create the service in all environments,
        // but we can test the static function
        assert_eq!(clipboard_supported(), expected);
    }
}
