//! Shared types for the CortenBrowser Browser Shell
//!
//! This crate provides common data structures and type definitions used across
//! all browser shell components, including ID types, configuration structs,
//! keyboard shortcuts, and error types.

mod errors;
mod ids;
mod keyboard_shortcut;
mod window_config;

// Re-export all public types
pub use errors::*;
pub use ids::*;
pub use keyboard_shortcut::*;
pub use window_config::*;
