// @implements: REQ-005
//! System notifications
//!
//! Cross-platform system notification support.

use shared_types::error::ComponentError;

/// Notification structure
#[derive(Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

/// System notifier
pub struct Notifier {
    // Stub - will be filled in GREEN phase
}

impl Notifier {
    /// Create a new notifier
    pub fn new() -> Self {
        Self {}
    }

    /// Send a notification
    /// Stub - will implement in GREEN phase
    pub async fn send(&self, _notification: &Notification) -> Result<(), ComponentError> {
        unimplemented!("Notifier::send not yet implemented")
    }
}

impl Default for Notifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Notification {
    /// Create a new notification
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            icon: None,
        }
    }

    /// Set notification icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}
