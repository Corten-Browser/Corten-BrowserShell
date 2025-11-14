// @implements: REQ-UI-001, REQ-UI-002, REQ-UI-003, REQ-UI-004, REQ-UI-005
//! UI Chrome Component
//!
//! Browser UI widgets including address bar, tab bar, toolbar, menu system, and theming.
//!
//! # Modules
//!
//! - `widgets`: UI widget implementations (address bar, tab bar, toolbar)
//! - `theme`: Theme management (light/dark/auto)
//! - `shortcuts`: Keyboard shortcut handling
//! - `menu`: Application menu system
//!
//! # Example
//!
//! ```rust
//! use ui_chrome::widgets::address_bar::AddressBar;
//! use ui_chrome::theme::{ThemeManager, Theme};
//!
//! let mut address_bar = AddressBar::new();
//! address_bar.set_text("https://example.com".to_string());
//!
//! let mut theme_manager = ThemeManager::new();
//! theme_manager.set_theme(Theme::Dark);
//! ```

pub mod widgets;
pub mod theme;
pub mod shortcuts;
pub mod menu;
