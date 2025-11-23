//! Shared types for the CortenBrowser Browser Shell
//!
//! This crate provides common data structures and type definitions used across
//! all browser shell components, including ID types, configuration structs,
//! keyboard shortcuts, error types, and the core [`BrowserComponent`] trait.
//!
//! # Core Trait
//!
//! The [`BrowserComponent`] trait defines a standard interface that all browser
//! shell components must implement. It provides:
//!
//! - Lifecycle management (`initialize`, `shutdown`)
//! - Message handling (`handle_message`)
//! - Health monitoring (`health_check`)
//! - Metrics collection (`get_metrics`)
//!
//! # Storage Layer
//!
//! The [`storage`] module provides a SQLite-based key-value store with:
//! - Typed key-value storage with JSON serialization
//! - Migration system for schema updates
//! - Expiration support for cache-like data
//! - Thread-safe concurrent access
//!
//! Enable the `storage` feature to use the SQLite backend:
//!
//! ```toml
//! [dependencies]
//! shared_types = { path = "../shared_types", features = ["storage"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use shared_types::{BrowserComponent, ComponentState, ComponentHealth};
//!
//! // All browser components implement BrowserComponent
//! async fn check_component<C: BrowserComponent>(component: &C) {
//!     let state = component.state();
//!     let health = component.health_check().await;
//!     let metrics = component.get_metrics();
//! }
//!
//! // Using storage
//! use shared_types::storage::{Storage, InMemoryStorage};
//!
//! let storage = InMemoryStorage::new("settings");
//! storage.set("theme", &"dark").await?;
//! let theme: Option<String> = storage.get("theme").await?;
//! ```

mod component;
mod errors;
mod ids;
mod keyboard_shortcut;
pub mod storage;
mod window_config;

// Re-export all public types
pub use component::*;
pub use errors::*;
pub use ids::*;
pub use keyboard_shortcut::*;
pub use window_config::*;

// Re-export core storage types for convenience
pub use storage::{InMemoryStorage, Migration, Storage, StorageMetadata, StorageResult};

#[cfg(feature = "storage")]
pub use storage::SqliteStorage;
