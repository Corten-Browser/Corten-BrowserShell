// @implements: REQ-004
//! Clipboard operations
//!
//! Cross-platform clipboard access for text content.

use shared_types::error::ComponentError;

/// Clipboard manager
pub struct Clipboard {
    // Stub - will be filled in GREEN phase
}

impl Clipboard {
    /// Create a new clipboard manager
    pub fn new() -> Self {
        Self {}
    }

    /// Read text from clipboard
    /// Stub - will implement in GREEN phase
    pub async fn read_text(&self) -> Result<String, ComponentError> {
        unimplemented!("Clipboard::read_text not yet implemented")
    }

    /// Write text to clipboard
    /// Stub - will implement in GREEN phase
    pub async fn write_text(&mut self, _text: &str) -> Result<(), ComponentError> {
        unimplemented!("Clipboard::write_text not yet implemented")
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}
