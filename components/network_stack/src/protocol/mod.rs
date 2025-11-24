//! Protocol handler system for routing URLs to appropriate handlers.
//!
//! This module provides a unified interface for handling different URL protocols:
//! - `http://` and `https://` - delegated to network client
//! - `file://` - local file system access
//! - `extension://` - browser extension resources
//! - `browser://` - internal browser pages (settings, history, etc.)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    ProtocolRouter                           │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
//! │  │ http(s)://  │  │  file://    │  │ extension:// │        │
//! │  │  Handler    │  │  Handler    │  │   Handler    │        │
//! │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
//! │         │                │                │                │
//! │         ▼                ▼                ▼                │
//! │  ┌─────────────────────────────────────────────────────┐  │
//! │  │              ProtocolResponse                        │  │
//! │  └─────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use network_stack::protocol::{ProtocolRouter, HttpProtocolHandler, FileProtocolHandler};
//! use url::Url;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut router = ProtocolRouter::new();
//!     router.register(Box::new(HttpProtocolHandler::new(client)));
//!     router.register(Box::new(FileProtocolHandler::new()));
//!
//!     let url = Url::parse("file:///home/user/document.html").unwrap();
//!     let response = router.handle(&url).await.unwrap();
//! }
//! ```

mod handlers;
mod response;
mod router;
mod types;

pub use handlers::{
    ExtensionProtocolHandler, FileProtocolHandler, HttpProtocolHandler, InternalProtocolHandler,
};
pub use response::ProtocolResponse;
pub use router::ProtocolRouter;
pub use types::{ProtocolError, ProtocolHandler, ProtocolResult};
