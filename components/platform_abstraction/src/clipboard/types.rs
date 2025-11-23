//! Clipboard types and data structures
//!
//! This module defines the core types used for cross-platform clipboard operations.

use serde::{Deserialize, Serialize};

/// Image data that can be stored in the clipboard
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageData {
    /// Raw pixel data in RGBA format (4 bytes per pixel)
    pub bytes: Vec<u8>,
    /// Width of the image in pixels
    pub width: usize,
    /// Height of the image in pixels
    pub height: usize,
}

impl ImageData {
    /// Create new image data from raw RGBA pixels
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw pixel data in RGBA format (4 bytes per pixel)
    /// * `width` - Width of the image in pixels
    /// * `height` - Height of the image in pixels
    ///
    /// # Panics
    ///
    /// Panics if `bytes.len() != width * height * 4`
    pub fn new(bytes: Vec<u8>, width: usize, height: usize) -> Self {
        assert_eq!(
            bytes.len(),
            width * height * 4,
            "Image data size mismatch: expected {} bytes, got {}",
            width * height * 4,
            bytes.len()
        );
        Self {
            bytes,
            width,
            height,
        }
    }

    /// Create new image data from raw RGBA pixels with validation
    ///
    /// Returns `None` if the byte count doesn't match width * height * 4
    pub fn try_new(bytes: Vec<u8>, width: usize, height: usize) -> Option<Self> {
        if bytes.len() == width * height * 4 {
            Some(Self {
                bytes,
                width,
                height,
            })
        } else {
            None
        }
    }

    /// Returns true if the image is empty (zero dimensions)
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0 || self.bytes.is_empty()
    }

    /// Returns the total number of pixels
    pub fn pixel_count(&self) -> usize {
        self.width * self.height
    }
}

/// Content types that can be stored in the clipboard
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardContent {
    /// Plain text content
    Text(String),
    /// HTML formatted content
    Html(String),
    /// Image data (RGBA format)
    Image(ImageData),
    /// Clipboard is empty or contains unsupported format
    Empty,
}

impl ClipboardContent {
    /// Returns true if the clipboard content is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, ClipboardContent::Empty)
    }

    /// Returns true if the clipboard contains text
    pub fn is_text(&self) -> bool {
        matches!(self, ClipboardContent::Text(_))
    }

    /// Returns true if the clipboard contains HTML
    pub fn is_html(&self) -> bool {
        matches!(self, ClipboardContent::Html(_))
    }

    /// Returns true if the clipboard contains an image
    pub fn is_image(&self) -> bool {
        matches!(self, ClipboardContent::Image(_))
    }

    /// Get the text content if available
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ClipboardContent::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Get the HTML content if available
    pub fn as_html(&self) -> Option<&str> {
        match self {
            ClipboardContent::Html(s) => Some(s),
            _ => None,
        }
    }

    /// Get the image data if available
    pub fn as_image(&self) -> Option<&ImageData> {
        match self {
            ClipboardContent::Image(img) => Some(img),
            _ => None,
        }
    }
}

impl Default for ClipboardContent {
    fn default() -> Self {
        ClipboardContent::Empty
    }
}

impl From<String> for ClipboardContent {
    fn from(text: String) -> Self {
        ClipboardContent::Text(text)
    }
}

impl From<&str> for ClipboardContent {
    fn from(text: &str) -> Self {
        ClipboardContent::Text(text.to_string())
    }
}

impl From<ImageData> for ClipboardContent {
    fn from(image: ImageData) -> Self {
        ClipboardContent::Image(image)
    }
}

/// Format of clipboard data for querying availability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClipboardFormat {
    /// Plain text format
    Text,
    /// HTML format
    Html,
    /// Image format (PNG/BMP)
    Image,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_data_new() {
        let bytes = vec![0u8; 16]; // 2x2 RGBA image
        let img = ImageData::new(bytes, 2, 2);
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(img.bytes.len(), 16);
    }

    #[test]
    #[should_panic(expected = "Image data size mismatch")]
    fn test_image_data_new_invalid_size() {
        let bytes = vec![0u8; 10]; // Wrong size
        ImageData::new(bytes, 2, 2);
    }

    #[test]
    fn test_image_data_try_new_valid() {
        let bytes = vec![0u8; 16];
        let img = ImageData::try_new(bytes, 2, 2);
        assert!(img.is_some());
    }

    #[test]
    fn test_image_data_try_new_invalid() {
        let bytes = vec![0u8; 10];
        let img = ImageData::try_new(bytes, 2, 2);
        assert!(img.is_none());
    }

    #[test]
    fn test_image_data_is_empty() {
        let empty = ImageData {
            bytes: vec![],
            width: 0,
            height: 0,
        };
        assert!(empty.is_empty());

        let not_empty = ImageData::new(vec![0u8; 4], 1, 1);
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn test_image_data_pixel_count() {
        let img = ImageData::new(vec![0u8; 64], 4, 4);
        assert_eq!(img.pixel_count(), 16);
    }

    #[test]
    fn test_clipboard_content_is_empty() {
        assert!(ClipboardContent::Empty.is_empty());
        assert!(!ClipboardContent::Text("hello".into()).is_empty());
    }

    #[test]
    fn test_clipboard_content_is_text() {
        assert!(ClipboardContent::Text("hello".into()).is_text());
        assert!(!ClipboardContent::Empty.is_text());
        assert!(!ClipboardContent::Html("<p>hi</p>".into()).is_text());
    }

    #[test]
    fn test_clipboard_content_is_html() {
        assert!(ClipboardContent::Html("<p>hi</p>".into()).is_html());
        assert!(!ClipboardContent::Text("hello".into()).is_html());
    }

    #[test]
    fn test_clipboard_content_is_image() {
        let img = ImageData::new(vec![0u8; 4], 1, 1);
        assert!(ClipboardContent::Image(img).is_image());
        assert!(!ClipboardContent::Text("hello".into()).is_image());
    }

    #[test]
    fn test_clipboard_content_as_text() {
        let content = ClipboardContent::Text("hello".into());
        assert_eq!(content.as_text(), Some("hello"));

        let empty = ClipboardContent::Empty;
        assert_eq!(empty.as_text(), None);
    }

    #[test]
    fn test_clipboard_content_as_html() {
        let content = ClipboardContent::Html("<p>hi</p>".into());
        assert_eq!(content.as_html(), Some("<p>hi</p>"));

        let text = ClipboardContent::Text("hello".into());
        assert_eq!(text.as_html(), None);
    }

    #[test]
    fn test_clipboard_content_as_image() {
        let img = ImageData::new(vec![0u8; 4], 1, 1);
        let content = ClipboardContent::Image(img.clone());
        assert_eq!(content.as_image(), Some(&img));

        let text = ClipboardContent::Text("hello".into());
        assert_eq!(text.as_image(), None);
    }

    #[test]
    fn test_clipboard_content_default() {
        let content = ClipboardContent::default();
        assert!(content.is_empty());
    }

    #[test]
    fn test_clipboard_content_from_string() {
        let content: ClipboardContent = "hello".to_string().into();
        assert_eq!(content.as_text(), Some("hello"));
    }

    #[test]
    fn test_clipboard_content_from_str() {
        let content: ClipboardContent = "hello".into();
        assert_eq!(content.as_text(), Some("hello"));
    }

    #[test]
    fn test_clipboard_content_from_image() {
        let img = ImageData::new(vec![0u8; 4], 1, 1);
        let content: ClipboardContent = img.clone().into();
        assert!(content.is_image());
    }

    #[test]
    fn test_clipboard_format_variants() {
        let formats = [
            ClipboardFormat::Text,
            ClipboardFormat::Html,
            ClipboardFormat::Image,
        ];

        // Verify all variants exist and are distinct
        assert_ne!(formats[0], formats[1]);
        assert_ne!(formats[1], formats[2]);
        assert_ne!(formats[0], formats[2]);
    }
}
