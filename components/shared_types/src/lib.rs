//! Shared types for the CortenBrowser Browser Shell
//!
//! This crate provides common data structures and type definitions used across
//! all browser shell components, including ID types, configuration structs,
//! keyboard shortcuts, and error types.

mod ids;
mod window_config;
mod keyboard_shortcut;
mod errors;

// Re-export all public types
pub use ids::*;
pub use window_config::*;
pub use keyboard_shortcut::*;
pub use errors::*;
