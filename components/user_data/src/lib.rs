// @implements: REQ-007, REQ-008, REQ-009, REQ-010
//! User Data Management Library
//!
//! Provides settings persistence, bookmarks management, downloads tracking,
//! and history storage for the Corten browser.
//!
//! # Modules
//!
//! - `settings`: Key-value settings storage with SQLite backend
//! - `bookmarks`: Hierarchical bookmarks with folder support
//! - `downloads`: Download state tracking with resume support
//! - `history`: Browsing history with search capability
//!
//! # Example
//!
//! ```rust,no_run
//! use user_data::settings::SettingsManager;
//! use rusqlite::Connection;
//!
//! let conn = Connection::open("user_data.db").unwrap();
//! let mut settings = SettingsManager::new(conn).unwrap();
//! settings.set("theme", "dark").unwrap();
//! ```

pub mod settings;
pub mod bookmarks;
pub mod downloads;
pub mod history;

pub use settings::SettingsManager;
pub use bookmarks::BookmarksManager;
pub use downloads::DownloadsTracker;
pub use history::HistoryManager;
