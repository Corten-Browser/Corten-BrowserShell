//! Extension Messaging API
//!
//! Provides messaging between content scripts and background scripts,
//! as well as between extensions.

use crate::types::{ExtensionError, ExtensionId, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Message sent between extension components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionMessage {
    /// Message ID for response correlation
    pub id: u64,
    /// Target of the message
    pub target: MessageTarget,
    /// Message payload (JSON-serializable)
    pub payload: serde_json::Value,
    /// Whether this expects a response
    pub expects_response: bool,
}

impl ExtensionMessage {
    /// Create a new message
    pub fn new(target: MessageTarget, payload: serde_json::Value) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self {
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            target,
            payload,
            expects_response: false,
        }
    }

    /// Create a message that expects a response
    pub fn with_response(mut self) -> Self {
        self.expects_response = true;
        self
    }

    /// Create a response to this message
    pub fn create_response(&self, payload: serde_json::Value) -> ExtensionMessage {
        ExtensionMessage {
            id: self.id,
            target: MessageTarget::Response,
            payload,
            expects_response: false,
        }
    }
}

/// Target for a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageTarget {
    /// Message to background script
    Background,
    /// Message to content script in a specific tab
    ContentScript { tab_id: u64 },
    /// Message to all content scripts
    AllContentScripts,
    /// Message to another extension
    Extension { extension_id: ExtensionId },
    /// Message to popup
    Popup,
    /// Message to options page
    Options,
    /// Response to a previous message
    Response,
    /// Native application
    Native { application: String },
}

/// Sender for messages from a specific source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSender {
    /// ID of the sending extension
    pub extension_id: ExtensionId,
    /// Tab ID if sent from a content script
    pub tab_id: Option<u64>,
    /// Frame ID if sent from a frame
    pub frame_id: Option<u64>,
    /// URL of the sending page/script
    pub url: Option<String>,
}

impl MessageSender {
    /// Create a new message sender from extension background
    pub fn from_background(extension_id: ExtensionId) -> Self {
        Self {
            extension_id,
            tab_id: None,
            frame_id: None,
            url: None,
        }
    }

    /// Create a new message sender from content script
    pub fn from_content_script(extension_id: ExtensionId, tab_id: u64, url: String) -> Self {
        Self {
            extension_id,
            tab_id: Some(tab_id),
            frame_id: Some(0),
            url: Some(url),
        }
    }
}

/// A channel for sending and receiving messages
pub struct MessageChannel {
    /// Sender half of the channel
    sender: mpsc::Sender<(ExtensionMessage, MessageSender)>,
    /// Receiver half of the channel
    receiver: mpsc::Receiver<(ExtensionMessage, MessageSender)>,
}

impl MessageChannel {
    /// Create a new message channel
    pub fn new(buffer_size: usize) -> Self {
        let (sender, receiver) = mpsc::channel(buffer_size);
        Self { sender, receiver }
    }

    /// Get a clone of the sender
    pub fn sender(&self) -> mpsc::Sender<(ExtensionMessage, MessageSender)> {
        self.sender.clone()
    }

    /// Receive a message
    pub async fn recv(&mut self) -> Option<(ExtensionMessage, MessageSender)> {
        self.receiver.recv().await
    }

    /// Try to receive a message without blocking
    pub fn try_recv(&mut self) -> Option<(ExtensionMessage, MessageSender)> {
        self.receiver.try_recv().ok()
    }
}

/// Messaging API
///
/// Manages message channels for all extensions
pub struct MessagingApi {
    /// Message channels for extensions (extension_id -> channel sender)
    channels: HashMap<ExtensionId, mpsc::Sender<(ExtensionMessage, MessageSender)>>,
    /// Content script channels (extension_id, tab_id) -> channel sender
    content_script_channels:
        HashMap<(ExtensionId, u64), mpsc::Sender<(ExtensionMessage, MessageSender)>>,
    /// Pending responses (message_id -> response sender)
    pending_responses:
        HashMap<u64, tokio::sync::oneshot::Sender<std::result::Result<serde_json::Value, String>>>,
    /// Message handlers (extension_id -> handlers)
    message_handlers: HashMap<ExtensionId, Vec<Box<dyn MessageHandler + Send + Sync>>>,
}

impl MessagingApi {
    /// Create a new MessagingApi
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            content_script_channels: HashMap::new(),
            pending_responses: HashMap::new(),
            message_handlers: HashMap::new(),
        }
    }

    /// Register a message channel for an extension
    pub fn register_channel(
        &mut self,
        extension_id: ExtensionId,
        sender: mpsc::Sender<(ExtensionMessage, MessageSender)>,
    ) {
        self.channels.insert(extension_id, sender);
    }

    /// Register a content script channel
    pub fn register_content_script_channel(
        &mut self,
        extension_id: ExtensionId,
        tab_id: u64,
        sender: mpsc::Sender<(ExtensionMessage, MessageSender)>,
    ) {
        self.content_script_channels
            .insert((extension_id, tab_id), sender);
    }

    /// Remove all channels for an extension
    pub fn remove_extension(&mut self, extension_id: ExtensionId) {
        self.channels.remove(&extension_id);
        self.content_script_channels
            .retain(|(ext_id, _), _| *ext_id != extension_id);
        self.message_handlers.remove(&extension_id);
    }

    /// Remove content script channel when tab closes
    pub fn remove_content_script(&mut self, extension_id: ExtensionId, tab_id: u64) {
        self.content_script_channels
            .remove(&(extension_id, tab_id));
    }

    /// Send a message from an extension
    pub async fn send(&self, from: ExtensionId, message: ExtensionMessage) -> Result<()> {
        match &message.target {
            MessageTarget::Background => {
                // Find the extension's background channel
                if let Some(sender) = self.channels.get(&from) {
                    let msg_sender = MessageSender::from_background(from);
                    sender
                        .send((message, msg_sender))
                        .await
                        .map_err(|_| ExtensionError::MessagingError("Channel closed".to_string()))?;
                }
            }

            MessageTarget::ContentScript { tab_id } => {
                // Send to specific content script
                if let Some(sender) = self.content_script_channels.get(&(from, *tab_id)) {
                    let msg_sender = MessageSender::from_background(from);
                    sender
                        .send((message, msg_sender))
                        .await
                        .map_err(|_| ExtensionError::MessagingError("Channel closed".to_string()))?;
                }
            }

            MessageTarget::AllContentScripts => {
                // Send to all content scripts for this extension
                let msg_sender = MessageSender::from_background(from);
                for ((ext_id, _), sender) in &self.content_script_channels {
                    if *ext_id == from {
                        let _ = sender.send((message.clone(), msg_sender.clone())).await;
                    }
                }
            }

            MessageTarget::Extension { extension_id } => {
                // Send to another extension
                if let Some(sender) = self.channels.get(extension_id) {
                    let msg_sender = MessageSender::from_background(from);
                    sender
                        .send((message, msg_sender))
                        .await
                        .map_err(|_| ExtensionError::MessagingError("Channel closed".to_string()))?;
                }
            }

            MessageTarget::Response => {
                // This should be handled by the response system
                return Err(ExtensionError::MessagingError(
                    "Cannot send Response directly".to_string(),
                ));
            }

            MessageTarget::Popup | MessageTarget::Options => {
                // Send to popup/options page (same as background for now)
                if let Some(sender) = self.channels.get(&from) {
                    let msg_sender = MessageSender::from_background(from);
                    sender
                        .send((message, msg_sender))
                        .await
                        .map_err(|_| ExtensionError::MessagingError("Channel closed".to_string()))?;
                }
            }

            MessageTarget::Native { application: _ } => {
                // Native messaging not implemented
                return Err(ExtensionError::MessagingError(
                    "Native messaging not supported".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Send a message and wait for response
    pub async fn send_and_wait(
        &mut self,
        from: ExtensionId,
        mut message: ExtensionMessage,
        timeout: std::time::Duration,
    ) -> Result<serde_json::Value> {
        message.expects_response = true;
        let message_id = message.id;

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.pending_responses.insert(message_id, tx);

        self.send(from, message).await?;

        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(Ok(value))) => Ok(value),
            Ok(Ok(Err(e))) => Err(ExtensionError::MessagingError(e)),
            Ok(Err(_)) => Err(ExtensionError::MessagingError("Response cancelled".to_string())),
            Err(_) => {
                self.pending_responses.remove(&message_id);
                Err(ExtensionError::MessagingError("Response timeout".to_string()))
            }
        }
    }

    /// Handle a response to a pending message
    pub fn handle_response(
        &mut self,
        message_id: u64,
        result: std::result::Result<serde_json::Value, String>,
    ) {
        if let Some(sender) = self.pending_responses.remove(&message_id) {
            let _ = sender.send(result);
        }
    }

    /// Check if an extension has a registered channel
    pub fn has_channel(&self, extension_id: ExtensionId) -> bool {
        self.channels.contains_key(&extension_id)
    }

    /// Get all registered extension IDs
    pub fn list_extensions(&self) -> Vec<ExtensionId> {
        self.channels.keys().cloned().collect()
    }
}

impl Default for MessagingApi {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for handling messages
#[allow(dead_code)]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message
    fn handle(
        &self,
        message: &ExtensionMessage,
        sender: &MessageSender,
    ) -> Option<serde_json::Value>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_message_creation() {
        let msg = ExtensionMessage::new(
            MessageTarget::Background,
            serde_json::json!({"action": "test"}),
        );

        assert!(msg.id > 0);
        assert!(!msg.expects_response);

        let msg_with_response = msg.with_response();
        assert!(msg_with_response.expects_response);
    }

    #[test]
    fn test_message_response() {
        let msg = ExtensionMessage::new(
            MessageTarget::Background,
            serde_json::json!({"action": "test"}),
        );

        let response = msg.create_response(serde_json::json!({"result": "ok"}));
        assert_eq!(response.id, msg.id);
        assert!(matches!(response.target, MessageTarget::Response));
    }

    #[test]
    fn test_message_sender() {
        let ext_id = ExtensionId::from_string("test-ext");

        let bg_sender = MessageSender::from_background(ext_id);
        assert!(bg_sender.tab_id.is_none());

        let cs_sender =
            MessageSender::from_content_script(ext_id, 1, "https://example.com".to_string());
        assert_eq!(cs_sender.tab_id, Some(1));
        assert_eq!(cs_sender.url, Some("https://example.com".to_string()));
    }

    #[tokio::test]
    async fn test_message_channel() {
        let mut channel = MessageChannel::new(10);
        let sender = channel.sender();

        let ext_id = ExtensionId::from_string("test-ext");
        let msg = ExtensionMessage::new(
            MessageTarget::Background,
            serde_json::json!({"test": true}),
        );
        let msg_sender = MessageSender::from_background(ext_id);

        sender.send((msg.clone(), msg_sender)).await.unwrap();

        let (received, _) = channel.recv().await.unwrap();
        assert_eq!(received.id, msg.id);
    }

    #[test]
    fn test_messaging_api() {
        let mut api = MessagingApi::new();
        let ext_id = ExtensionId::from_string("test-ext");

        assert!(!api.has_channel(ext_id));

        let (tx, _rx) = mpsc::channel(10);
        api.register_channel(ext_id, tx);

        assert!(api.has_channel(ext_id));

        api.remove_extension(ext_id);
        assert!(!api.has_channel(ext_id));
    }

    #[test]
    fn test_content_script_channel() {
        let mut api = MessagingApi::new();
        let ext_id = ExtensionId::from_string("test-ext");

        let (tx, _rx) = mpsc::channel(10);
        api.register_content_script_channel(ext_id, 1, tx.clone());
        api.register_content_script_channel(ext_id, 2, tx);

        api.remove_content_script(ext_id, 1);

        // Channel for tab 2 should still exist
        api.remove_extension(ext_id);
    }
}
