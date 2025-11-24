//! Cross-platform system clipboard support
//!
//! This module provides a unified API for reading from and writing to the system
//! clipboard across Linux, Windows, and macOS platforms.
//!
//! # Overview
//!
//! The clipboard system provides:
//! - Cross-platform clipboard API using the arboard crate
//! - Support for plain text content
//! - Support for HTML formatted content
//! - Support for image content (RGBA format)
//! - Thread-safe operations
//! - Error handling for clipboard unavailability
//!
//! # Example
//!
//! ```rust,no_run
//! use platform_abstraction::clipboard::{
//!     ClipboardService, SystemClipboardService, ClipboardContent,
//! };
//!
//! // Create a clipboard service
//! let service = SystemClipboardService::new().expect("clipboard available");
//!
//! // Write text to clipboard
//! service.write_text("Hello, clipboard!").unwrap();
//!
//! // Read text from clipboard
//! let text = service.read_text().unwrap();
//! println!("Clipboard contains: {}", text);
//!
//! // Check what content is available
//! if service.has_text() {
//!     println!("Clipboard has text");
//! }
//! if service.has_image() {
//!     println!("Clipboard has an image");
//! }
//! ```
//!
//! # Working with HTML
//!
//! ```rust,no_run
//! use platform_abstraction::clipboard::{ClipboardService, SystemClipboardService};
//!
//! let service = SystemClipboardService::new().expect("clipboard available");
//!
//! // Write HTML content (also sets plain text fallback)
//! service.write_html("<p>Hello <b>World</b></p>").unwrap();
//!
//! // Read HTML content
//! if service.has_html() {
//!     let html = service.read_html().unwrap();
//!     println!("HTML: {}", html);
//! }
//! ```
//!
//! # Working with Images
//!
//! ```rust,no_run
//! use platform_abstraction::clipboard::{
//!     ClipboardService, SystemClipboardService, ImageData,
//! };
//!
//! let service = SystemClipboardService::new().expect("clipboard available");
//!
//! // Create a 2x2 red image (RGBA format)
//! let red_pixel = [255u8, 0, 0, 255];
//! let bytes: Vec<u8> = red_pixel.iter().cloned().cycle().take(16).collect();
//! let image = ImageData::new(bytes, 2, 2);
//!
//! // Write image to clipboard
//! service.write_image(&image).unwrap();
//!
//! // Read image from clipboard
//! let img = service.read_image().unwrap();
//! println!("Image size: {}x{}", img.width, img.height);
//! ```
//!
//! # Auto-detecting Content Type
//!
//! ```rust,no_run
//! use platform_abstraction::clipboard::{
//!     ClipboardService, SystemClipboardService, ClipboardContent,
//! };
//!
//! let service = SystemClipboardService::new().expect("clipboard available");
//!
//! // Read whatever is in the clipboard
//! match service.read_content().unwrap() {
//!     ClipboardContent::Text(text) => println!("Text: {}", text),
//!     ClipboardContent::Html(html) => println!("HTML: {}", html),
//!     ClipboardContent::Image(img) => println!("Image: {}x{}", img.width, img.height),
//!     ClipboardContent::Empty => println!("Clipboard is empty"),
//! }
//! ```

mod service;
mod types;

// Re-export public types
pub use service::{clipboard_supported, ClipboardError, ClipboardResult, ClipboardService, SystemClipboardService};
pub use types::{ClipboardContent, ClipboardFormat, ImageData};
