//! Cross-platform drag and drop support
//!
//! This module provides a unified API for drag and drop operations across
//! Linux, Windows, and macOS platforms.
//!
//! # Overview
//!
//! The drag and drop system provides:
//! - Cross-platform drag and drop API
//! - Support for files, text, HTML, images, and URLs
//! - Drop target registration and management
//! - Visual drop indicators
//! - Event-based notification system
//! - Thread-safe operations
//!
//! # Example
//!
//! ```rust,no_run
//! use platform_abstraction::drag_drop::{
//!     DragDropManager, DragData, DropTarget, FileDropTarget, Point,
//! };
//! use std::sync::{Arc, Mutex};
//!
//! // Create a drag and drop manager
//! let manager = DragDropManager::new();
//!
//! // Register a file drop target
//! let target = Arc::new(Mutex::new(FileDropTarget::new(0.0, 0.0, 200.0, 200.0)));
//! manager.register_target(target).unwrap();
//!
//! // Start a drag operation
//! let data = DragData::Text("Hello, world!".to_string());
//! manager.start_drag(data, Point::new(10.0, 10.0)).unwrap();
//!
//! // Update drag position as mouse moves
//! manager.update_drag_position(Point::new(50.0, 50.0)).unwrap();
//!
//! // Complete the drop
//! let effect = manager.drop().unwrap();
//! ```
//!
//! # Working with Files
//!
//! ```rust,no_run
//! use platform_abstraction::drag_drop::{DragData, FileDropTarget, DropTarget, Point};
//! use std::path::PathBuf;
//!
//! // Create a file drop target that only accepts text files
//! let mut target = FileDropTarget::new(0.0, 0.0, 100.0, 100.0)
//!     .with_extensions(vec!["txt".to_string(), "md".to_string()]);
//!
//! // Create drag data from files
//! let files = vec![PathBuf::from("/path/to/file.txt")];
//! let data = DragData::Files(files);
//!
//! // Check if target accepts the data
//! if target.accepts(&data) {
//!     target.on_drop(data, Point::new(50.0, 50.0)).unwrap();
//! }
//! ```
//!
//! # Creating Custom Drop Targets
//!
//! ```rust,no_run
//! use platform_abstraction::drag_drop::{
//!     DragData, DropTarget, DropTargetId, DropIndicator, Point, DragDropResult,
//! };
//!
//! struct CustomDropTarget {
//!     id: DropTargetId,
//!     bounds: (f32, f32, f32, f32), // x, y, w, h
//! }
//!
//! impl DropTarget for CustomDropTarget {
//!     fn id(&self) -> DropTargetId {
//!         self.id
//!     }
//!
//!     fn accepts(&self, data: &DragData) -> bool {
//!         // Only accept URLs
//!         data.is_url()
//!     }
//!
//!     fn on_drop(&mut self, data: DragData, position: Point) -> DragDropResult<()> {
//!         if let Some(url) = data.as_url() {
//!             println!("Dropped URL: {} at {:?}", url, position);
//!         }
//!         Ok(())
//!     }
//!
//!     fn on_drag_enter(&mut self, _data: &DragData) {
//!         println!("Drag entered");
//!     }
//!
//!     fn on_drag_leave(&mut self) {
//!         println!("Drag left");
//!     }
//!
//!     fn contains_point(&self, point: Point) -> bool {
//!         point.x >= self.bounds.0 && point.x <= self.bounds.0 + self.bounds.2 &&
//!         point.y >= self.bounds.1 && point.y <= self.bounds.1 + self.bounds.3
//!     }
//! }
//! ```
//!
//! # Creating Drag Sources
//!
//! ```rust,no_run
//! use platform_abstraction::drag_drop::{
//!     DragSource, DragData, DropEffect, TextDragSource, UrlDragSource,
//! };
//!
//! // Text drag source
//! let text_source = TextDragSource::new("Draggable text".to_string());
//! if text_source.can_drag() {
//!     let data = text_source.get_drag_data();
//! }
//!
//! // URL drag source
//! let url_source = UrlDragSource::new("https://example.com".to_string());
//! if url_source.can_drag() {
//!     let data = url_source.get_drag_data();
//! }
//! ```
//!
//! # Event Handling
//!
//! ```rust,no_run
//! use platform_abstraction::drag_drop::{DragDropManager, DragDropEvent, DragData, Point};
//!
//! let manager = DragDropManager::new();
//!
//! // Add an event listener
//! manager.add_listener(|event| {
//!     match event {
//!         DragDropEvent::DragStarted { data, position } => {
//!             println!("Drag started at {:?}", position);
//!         }
//!         DragDropEvent::DragMoved { position, target_id } => {
//!             println!("Drag moved to {:?}", position);
//!         }
//!         DragDropEvent::DragEntered { target_id, .. } => {
//!             println!("Entered target {:?}", target_id);
//!         }
//!         DragDropEvent::DragLeft { target_id } => {
//!             println!("Left target {:?}", target_id);
//!         }
//!         DragDropEvent::Dropped { target_id, data, position, effect } => {
//!             println!("Dropped at {:?} with effect {:?}", position, effect);
//!         }
//!         DragDropEvent::DragCancelled => {
//!             println!("Drag cancelled");
//!         }
//!         DragDropEvent::DragEnded => {
//!             println!("Drag ended");
//!         }
//!     }
//! });
//! ```

mod service;
mod types;

// Re-export public types
pub use service::{
    drag_drop_supported, DragDropError, DragDropEvent, DragDropManager, DragDropResult,
    DragSource, DropTarget, FileDropTarget, TextDragSource, UrlDragSource,
};
pub use types::{
    DragData, DragFormat, DragState, DropEffect, DropIndicator, DropIndicatorStyle, DropTargetId,
    Point,
};
