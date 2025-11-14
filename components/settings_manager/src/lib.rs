//! Settings Manager Component
//!
//! Manages user settings and preferences for the CortenBrowser Browser Shell.
//! Provides persistence, synchronization, and default settings management.

mod defaults;
mod setting_value;
mod settings_manager;

pub use setting_value::SettingValue;
pub use settings_manager::SettingsManager;
