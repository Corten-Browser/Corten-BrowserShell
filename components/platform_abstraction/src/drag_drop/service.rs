//! Drag and drop service trait and cross-platform implementation
//!
//! This module provides the DragDropService trait, DropTarget trait, DragSource trait,
//! and a DragDropManager for coordinating drag and drop operations.

use super::types::{DragData, DragState, DropEffect, DropIndicator, DropTargetId, Point};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use thiserror::Error;

/// Errors that can occur during drag and drop operations
#[derive(Debug, Error)]
pub enum DragDropError {
    /// Drag and drop is not available on this platform
    #[error("drag and drop not available: {0}")]
    NotAvailable(String),

    /// No active drag operation
    #[error("no active drag operation")]
    NoDragActive,

    /// Drag operation already in progress
    #[error("drag operation already in progress")]
    DragAlreadyActive,

    /// Drop target not found
    #[error("drop target not found: {0}")]
    TargetNotFound(u64),

    /// Drop not allowed at this location
    #[error("drop not allowed: {0}")]
    DropNotAllowed(String),

    /// Failed to start drag operation
    #[error("failed to start drag: {0}")]
    StartFailed(String),

    /// Failed to complete drop operation
    #[error("drop failed: {0}")]
    DropFailed(String),

    /// Internal error
    #[error("internal error: {0}")]
    Internal(String),
}

/// Result type for drag and drop operations
pub type DragDropResult<T> = Result<T, DragDropError>;

/// Events generated during drag and drop operations
#[derive(Debug, Clone, PartialEq)]
pub enum DragDropEvent {
    /// Drag operation started
    DragStarted {
        /// The data being dragged
        data: DragData,
        /// Starting position
        position: Point,
    },
    /// Drag position updated
    DragMoved {
        /// Current position
        position: Point,
        /// The drop target currently under the cursor, if any
        target_id: Option<DropTargetId>,
    },
    /// Entered a drop target
    DragEntered {
        /// The target entered
        target_id: DropTargetId,
        /// The data being dragged
        data: DragData,
    },
    /// Left a drop target
    DragLeft {
        /// The target left
        target_id: DropTargetId,
    },
    /// Drop occurred
    Dropped {
        /// The target where the drop occurred
        target_id: DropTargetId,
        /// The dropped data
        data: DragData,
        /// Position of the drop
        position: Point,
        /// The drop effect that was applied
        effect: DropEffect,
    },
    /// Drag operation cancelled
    DragCancelled,
    /// Drag operation ended (successfully or cancelled)
    DragEnded,
}

/// Trait for components that can accept dropped content
pub trait DropTarget: Send + Sync {
    /// Get the unique identifier for this drop target
    fn id(&self) -> DropTargetId;

    /// Check if this target accepts the given drag data
    ///
    /// # Arguments
    ///
    /// * `data` - The data being dragged
    ///
    /// # Returns
    ///
    /// `true` if this target can accept the data, `false` otherwise
    fn accepts(&self, data: &DragData) -> bool;

    /// Get the allowed drop effects for the given data
    ///
    /// # Arguments
    ///
    /// * `data` - The data being dragged
    ///
    /// # Returns
    ///
    /// The drop effect that would apply if dropped here
    fn drop_effect(&self, data: &DragData) -> DropEffect {
        if self.accepts(data) {
            DropEffect::Copy
        } else {
            DropEffect::None
        }
    }

    /// Handle the drop event
    ///
    /// # Arguments
    ///
    /// * `data` - The dropped data
    /// * `position` - The position where the drop occurred
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the drop was handled successfully
    /// * `Err(DragDropError)` if the drop failed
    fn on_drop(&mut self, data: DragData, position: Point) -> DragDropResult<()>;

    /// Called when a drag enters this target
    ///
    /// # Arguments
    ///
    /// * `data` - The data being dragged
    fn on_drag_enter(&mut self, data: &DragData);

    /// Called when a drag leaves this target
    fn on_drag_leave(&mut self);

    /// Called when a drag moves over this target
    ///
    /// # Arguments
    ///
    /// * `position` - The current position over the target
    fn on_drag_over(&mut self, _position: Point) {
        // Default implementation does nothing
    }

    /// Get the drop indicator to display for this target
    ///
    /// # Arguments
    ///
    /// * `position` - The current drag position
    ///
    /// # Returns
    ///
    /// The drop indicator to display, if any
    fn get_drop_indicator(&self, _position: Point) -> Option<DropIndicator> {
        None
    }

    /// Check if a point is within this target's bounds
    ///
    /// # Arguments
    ///
    /// * `point` - The point to test
    ///
    /// # Returns
    ///
    /// `true` if the point is within this target's bounds
    fn contains_point(&self, point: Point) -> bool;
}

/// Trait for components that can be dragged
pub trait DragSource: Send + Sync {
    /// Check if this source can start a drag operation
    fn can_drag(&self) -> bool {
        true
    }

    /// Get the drag data for this source
    ///
    /// # Returns
    ///
    /// The data to drag, or `None` if drag is not allowed
    fn get_drag_data(&self) -> Option<DragData>;

    /// Get the allowed drop effects for this drag source
    fn allowed_effects(&self) -> Vec<DropEffect> {
        vec![DropEffect::Copy, DropEffect::Move, DropEffect::Link]
    }

    /// Called when the drag starts
    fn on_drag_start(&mut self) {}

    /// Called when the drag ends
    ///
    /// # Arguments
    ///
    /// * `effect` - The effect that was applied, or `None` if cancelled
    fn on_drag_end(&mut self, _effect: Option<DropEffect>) {}
}

/// Manager for coordinating drag and drop operations
pub struct DragDropManager {
    /// Current drag state, if any
    current_drag: RwLock<Option<DragState>>,
    /// Registered drop targets
    targets: RwLock<HashMap<DropTargetId, Arc<Mutex<dyn DropTarget>>>>,
    /// The target currently under the cursor
    current_target: RwLock<Option<DropTargetId>>,
    /// Event listeners
    listeners: RwLock<Vec<Box<dyn Fn(DragDropEvent) + Send + Sync>>>,
    /// Whether drag and drop is supported
    supported: bool,
}

impl DragDropManager {
    /// Create a new drag and drop manager
    pub fn new() -> Self {
        Self {
            current_drag: RwLock::new(None),
            targets: RwLock::new(HashMap::new()),
            current_target: RwLock::new(None),
            listeners: RwLock::new(Vec::new()),
            supported: drag_drop_supported(),
        }
    }

    /// Check if drag and drop is supported
    pub fn is_supported(&self) -> bool {
        self.supported
    }

    /// Start a drag operation
    ///
    /// # Arguments
    ///
    /// * `data` - The data to drag
    /// * `position` - The starting position
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the drag started successfully
    /// * `Err(DragDropError)` if a drag is already in progress
    pub fn start_drag(&self, data: DragData, position: Point) -> DragDropResult<()> {
        let mut current = self
            .current_drag
            .write()
            .map_err(|_| DragDropError::Internal("lock error".to_string()))?;

        if current.is_some() {
            return Err(DragDropError::DragAlreadyActive);
        }

        let state = DragState::new(data.clone(), position);
        *current = Some(state);

        self.emit_event(DragDropEvent::DragStarted { data, position });

        Ok(())
    }

    /// Update the drag position
    ///
    /// # Arguments
    ///
    /// * `position` - The new position
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the position was updated
    /// * `Err(DragDropError::NoDragActive)` if no drag is in progress
    pub fn update_drag_position(&self, position: Point) -> DragDropResult<()> {
        let mut current = self
            .current_drag
            .write()
            .map_err(|_| DragDropError::Internal("lock error".to_string()))?;

        let state = current.as_mut().ok_or(DragDropError::NoDragActive)?;
        state.update_position(position);

        // Find target under cursor
        let target_id = self.find_target_at(position, &state.data)?;
        let prev_target = self.get_current_target()?;

        // Handle target transitions
        if target_id != prev_target {
            // Left previous target
            if let Some(prev_id) = prev_target {
                self.notify_target_leave(prev_id)?;
                self.emit_event(DragDropEvent::DragLeft { target_id: prev_id });
            }

            // Entered new target
            if let Some(new_id) = target_id {
                self.notify_target_enter(new_id, &state.data)?;
                self.emit_event(DragDropEvent::DragEntered {
                    target_id: new_id,
                    data: state.data.clone(),
                });
            }

            self.set_current_target(target_id)?;
        } else if let Some(id) = target_id {
            // Still over same target, notify move
            self.notify_target_over(id, position)?;
        }

        state.set_over_valid_target(target_id.is_some());

        self.emit_event(DragDropEvent::DragMoved {
            position,
            target_id,
        });

        Ok(())
    }

    /// Complete the drop at the current position
    ///
    /// # Returns
    ///
    /// * `Ok(DropEffect)` - The effect that was applied
    /// * `Err(DragDropError)` if no drag is in progress or drop failed
    pub fn drop(&self) -> DragDropResult<DropEffect> {
        let mut current = self
            .current_drag
            .write()
            .map_err(|_| DragDropError::Internal("lock error".to_string()))?;

        let state = current.take().ok_or(DragDropError::NoDragActive)?;
        let target_id = self.get_current_target()?;

        // Clear current target
        self.set_current_target(None)?;

        let effect = if let Some(id) = target_id {
            // Get the target and perform the drop
            let targets = self
                .targets
                .read()
                .map_err(|_| DragDropError::Internal("lock error".to_string()))?;

            let target = targets
                .get(&id)
                .ok_or(DragDropError::TargetNotFound(id.0))?
                .clone();

            let mut target_guard = target
                .lock()
                .map_err(|_| DragDropError::Internal("lock error".to_string()))?;

            let effect = target_guard.drop_effect(&state.data);

            if effect.is_allowed() {
                target_guard.on_drop(state.data.clone(), state.current_position)?;

                self.emit_event(DragDropEvent::Dropped {
                    target_id: id,
                    data: state.data,
                    position: state.current_position,
                    effect,
                });

                effect
            } else {
                self.emit_event(DragDropEvent::DragCancelled);
                DropEffect::None
            }
        } else {
            self.emit_event(DragDropEvent::DragCancelled);
            DropEffect::None
        };

        self.emit_event(DragDropEvent::DragEnded);

        Ok(effect)
    }

    /// Cancel the current drag operation
    pub fn cancel_drag(&self) -> DragDropResult<()> {
        let mut current = self
            .current_drag
            .write()
            .map_err(|_| DragDropError::Internal("lock error".to_string()))?;

        if current.take().is_none() {
            return Err(DragDropError::NoDragActive);
        }

        // Leave current target if any
        if let Some(id) = self.get_current_target()? {
            self.notify_target_leave(id)?;
            self.emit_event(DragDropEvent::DragLeft { target_id: id });
        }

        self.set_current_target(None)?;
        self.emit_event(DragDropEvent::DragCancelled);
        self.emit_event(DragDropEvent::DragEnded);

        Ok(())
    }

    /// Check if a drag is currently in progress
    pub fn is_dragging(&self) -> bool {
        self.current_drag
            .read()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    /// Get the current drag state
    pub fn get_drag_state(&self) -> Option<DragState> {
        self.current_drag
            .read()
            .ok()
            .and_then(|guard| guard.clone())
    }

    /// Get the current drop indicator, if any
    pub fn get_drop_indicator(&self) -> Option<DropIndicator> {
        let state = self.get_drag_state()?;
        let target_id = self.get_current_target().ok()??;

        let targets = self.targets.read().ok()?;
        let target = targets.get(&target_id)?;
        let guard = target.lock().ok()?;

        guard.get_drop_indicator(state.current_position)
    }

    /// Register a drop target
    ///
    /// # Arguments
    ///
    /// * `target` - The drop target to register
    pub fn register_target(&self, target: Arc<Mutex<dyn DropTarget>>) -> DragDropResult<DropTargetId> {
        let id = {
            let guard = target
                .lock()
                .map_err(|_| DragDropError::Internal("lock error".to_string()))?;
            guard.id()
        };

        let mut targets = self
            .targets
            .write()
            .map_err(|_| DragDropError::Internal("lock error".to_string()))?;

        targets.insert(id, target);
        Ok(id)
    }

    /// Unregister a drop target
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the target to unregister
    pub fn unregister_target(&self, id: DropTargetId) -> DragDropResult<()> {
        let mut targets = self
            .targets
            .write()
            .map_err(|_| DragDropError::Internal("lock error".to_string()))?;

        targets.remove(&id);
        Ok(())
    }

    /// Add an event listener
    ///
    /// # Arguments
    ///
    /// * `listener` - The listener function
    pub fn add_listener<F>(&self, listener: F)
    where
        F: Fn(DragDropEvent) + Send + Sync + 'static,
    {
        if let Ok(mut listeners) = self.listeners.write() {
            listeners.push(Box::new(listener));
        }
    }

    /// Find the drop target at a given position
    fn find_target_at(
        &self,
        position: Point,
        data: &DragData,
    ) -> DragDropResult<Option<DropTargetId>> {
        let targets = self
            .targets
            .read()
            .map_err(|_| DragDropError::Internal("lock error".to_string()))?;

        for (id, target) in targets.iter() {
            let guard = target
                .lock()
                .map_err(|_| DragDropError::Internal("lock error".to_string()))?;

            if guard.contains_point(position) && guard.accepts(data) {
                return Ok(Some(*id));
            }
        }

        Ok(None)
    }

    /// Get the current target ID
    fn get_current_target(&self) -> DragDropResult<Option<DropTargetId>> {
        self.current_target
            .read()
            .map(|guard| *guard)
            .map_err(|_| DragDropError::Internal("lock error".to_string()))
    }

    /// Set the current target ID
    fn set_current_target(&self, id: Option<DropTargetId>) -> DragDropResult<()> {
        let mut current = self
            .current_target
            .write()
            .map_err(|_| DragDropError::Internal("lock error".to_string()))?;
        *current = id;
        Ok(())
    }

    /// Notify a target that a drag entered
    fn notify_target_enter(&self, id: DropTargetId, data: &DragData) -> DragDropResult<()> {
        let targets = self
            .targets
            .read()
            .map_err(|_| DragDropError::Internal("lock error".to_string()))?;

        if let Some(target) = targets.get(&id) {
            let mut guard = target
                .lock()
                .map_err(|_| DragDropError::Internal("lock error".to_string()))?;
            guard.on_drag_enter(data);
        }

        Ok(())
    }

    /// Notify a target that a drag left
    fn notify_target_leave(&self, id: DropTargetId) -> DragDropResult<()> {
        let targets = self
            .targets
            .read()
            .map_err(|_| DragDropError::Internal("lock error".to_string()))?;

        if let Some(target) = targets.get(&id) {
            let mut guard = target
                .lock()
                .map_err(|_| DragDropError::Internal("lock error".to_string()))?;
            guard.on_drag_leave();
        }

        Ok(())
    }

    /// Notify a target that a drag is over it
    fn notify_target_over(&self, id: DropTargetId, position: Point) -> DragDropResult<()> {
        let targets = self
            .targets
            .read()
            .map_err(|_| DragDropError::Internal("lock error".to_string()))?;

        if let Some(target) = targets.get(&id) {
            let mut guard = target
                .lock()
                .map_err(|_| DragDropError::Internal("lock error".to_string()))?;
            guard.on_drag_over(position);
        }

        Ok(())
    }

    /// Emit an event to all listeners
    fn emit_event(&self, event: DragDropEvent) {
        if let Ok(listeners) = self.listeners.read() {
            for listener in listeners.iter() {
                listener(event.clone());
            }
        }
    }
}

impl Default for DragDropManager {
    fn default() -> Self {
        Self::new()
    }
}

// Make DragDropManager thread-safe
unsafe impl Send for DragDropManager {}
unsafe impl Sync for DragDropManager {}

/// Check if drag and drop is supported on the current platform
pub fn drag_drop_supported() -> bool {
    cfg!(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    ))
}

/// A simple file drop target that accepts files
pub struct FileDropTarget {
    id: DropTargetId,
    bounds: (Point, f32, f32), // position, width, height
    accepted_extensions: Option<Vec<String>>,
    is_hovered: bool,
    dropped_files: Vec<std::path::PathBuf>,
}

impl FileDropTarget {
    /// Create a new file drop target
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id: DropTargetId::new(),
            bounds: (Point::new(x, y), width, height),
            accepted_extensions: None,
            is_hovered: false,
            dropped_files: Vec::new(),
        }
    }

    /// Set accepted file extensions (e.g., ["txt", "html"])
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.accepted_extensions = Some(extensions);
        self
    }

    /// Check if the target is currently hovered
    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    /// Get the files that have been dropped
    pub fn dropped_files(&self) -> &[std::path::PathBuf] {
        &self.dropped_files
    }

    /// Clear the dropped files list
    pub fn clear_dropped_files(&mut self) {
        self.dropped_files.clear();
    }
}

impl DropTarget for FileDropTarget {
    fn id(&self) -> DropTargetId {
        self.id
    }

    fn accepts(&self, data: &DragData) -> bool {
        if let DragData::Files(files) = data {
            if let Some(ref extensions) = self.accepted_extensions {
                // Check if any file has an accepted extension
                files.iter().any(|path| {
                    path.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| extensions.iter().any(|ext| ext.eq_ignore_ascii_case(e)))
                        .unwrap_or(false)
                })
            } else {
                // Accept all files
                !files.is_empty()
            }
        } else {
            false
        }
    }

    fn drop_effect(&self, data: &DragData) -> DropEffect {
        if self.accepts(data) {
            DropEffect::Copy
        } else {
            DropEffect::None
        }
    }

    fn on_drop(&mut self, data: DragData, _position: Point) -> DragDropResult<()> {
        if let DragData::Files(files) = data {
            self.dropped_files.extend(files);
            self.is_hovered = false;
            Ok(())
        } else {
            Err(DragDropError::DropNotAllowed("not files".to_string()))
        }
    }

    fn on_drag_enter(&mut self, _data: &DragData) {
        self.is_hovered = true;
    }

    fn on_drag_leave(&mut self) {
        self.is_hovered = false;
    }

    fn get_drop_indicator(&self, _position: Point) -> Option<DropIndicator> {
        if self.is_hovered {
            Some(DropIndicator::border(
                self.bounds.0.x,
                self.bounds.0.y,
                self.bounds.1,
                self.bounds.2,
            ))
        } else {
            None
        }
    }

    fn contains_point(&self, point: Point) -> bool {
        point.x >= self.bounds.0.x
            && point.x <= self.bounds.0.x + self.bounds.1
            && point.y >= self.bounds.0.y
            && point.y <= self.bounds.0.y + self.bounds.2
    }
}

/// A simple text drag source
pub struct TextDragSource {
    text: String,
    can_drag: bool,
}

impl TextDragSource {
    /// Create a new text drag source
    pub fn new(text: String) -> Self {
        Self {
            text,
            can_drag: true,
        }
    }

    /// Set whether dragging is allowed
    pub fn set_can_drag(&mut self, can_drag: bool) {
        self.can_drag = can_drag;
    }
}

impl DragSource for TextDragSource {
    fn can_drag(&self) -> bool {
        self.can_drag && !self.text.is_empty()
    }

    fn get_drag_data(&self) -> Option<DragData> {
        if self.can_drag() {
            Some(DragData::Text(self.text.clone()))
        } else {
            None
        }
    }

    fn allowed_effects(&self) -> Vec<DropEffect> {
        vec![DropEffect::Copy]
    }
}

/// A URL drag source
pub struct UrlDragSource {
    url: String,
}

impl UrlDragSource {
    /// Create a new URL drag source
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl DragSource for UrlDragSource {
    fn can_drag(&self) -> bool {
        !self.url.is_empty()
    }

    fn get_drag_data(&self) -> Option<DragData> {
        if self.can_drag() {
            Some(DragData::Url(self.url.clone()))
        } else {
            None
        }
    }

    fn allowed_effects(&self) -> Vec<DropEffect> {
        vec![DropEffect::Copy, DropEffect::Link]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_drag_drop_error_display() {
        let err = DragDropError::NotAvailable("test".to_string());
        assert_eq!(err.to_string(), "drag and drop not available: test");

        let err = DragDropError::NoDragActive;
        assert_eq!(err.to_string(), "no active drag operation");

        let err = DragDropError::DragAlreadyActive;
        assert_eq!(err.to_string(), "drag operation already in progress");

        let err = DragDropError::TargetNotFound(123);
        assert_eq!(err.to_string(), "drop target not found: 123");

        let err = DragDropError::DropNotAllowed("reason".to_string());
        assert_eq!(err.to_string(), "drop not allowed: reason");

        let err = DragDropError::StartFailed("error".to_string());
        assert_eq!(err.to_string(), "failed to start drag: error");

        let err = DragDropError::DropFailed("error".to_string());
        assert_eq!(err.to_string(), "drop failed: error");

        let err = DragDropError::Internal("error".to_string());
        assert_eq!(err.to_string(), "internal error: error");
    }

    #[test]
    fn test_drag_drop_supported() {
        let expected = cfg!(any(
            target_os = "linux",
            target_os = "windows",
            target_os = "macos"
        ));
        assert_eq!(drag_drop_supported(), expected);
    }

    #[test]
    fn test_drag_drop_manager_new() {
        let manager = DragDropManager::new();
        assert!(!manager.is_dragging());
        assert!(manager.get_drag_state().is_none());
    }

    #[test]
    fn test_drag_drop_manager_start_drag() {
        let manager = DragDropManager::new();
        let data = DragData::Text("test".to_string());
        let pos = Point::new(10.0, 20.0);

        let result = manager.start_drag(data.clone(), pos);
        assert!(result.is_ok());
        assert!(manager.is_dragging());

        let state = manager.get_drag_state().unwrap();
        assert_eq!(state.data, data);
        assert_eq!(state.start_position, pos);
    }

    #[test]
    fn test_drag_drop_manager_double_start() {
        let manager = DragDropManager::new();
        let data = DragData::Text("test".to_string());
        let pos = Point::new(10.0, 20.0);

        manager.start_drag(data.clone(), pos).unwrap();

        let result = manager.start_drag(data, pos);
        assert!(matches!(result, Err(DragDropError::DragAlreadyActive)));
    }

    #[test]
    fn test_drag_drop_manager_update_position() {
        let manager = DragDropManager::new();
        let data = DragData::Text("test".to_string());
        manager.start_drag(data, Point::origin()).unwrap();

        let new_pos = Point::new(100.0, 100.0);
        let result = manager.update_drag_position(new_pos);
        assert!(result.is_ok());

        let state = manager.get_drag_state().unwrap();
        assert_eq!(state.current_position, new_pos);
    }

    #[test]
    fn test_drag_drop_manager_update_no_drag() {
        let manager = DragDropManager::new();
        let result = manager.update_drag_position(Point::origin());
        assert!(matches!(result, Err(DragDropError::NoDragActive)));
    }

    #[test]
    fn test_drag_drop_manager_cancel() {
        let manager = DragDropManager::new();
        let data = DragData::Text("test".to_string());
        manager.start_drag(data, Point::origin()).unwrap();

        assert!(manager.is_dragging());
        manager.cancel_drag().unwrap();
        assert!(!manager.is_dragging());
    }

    #[test]
    fn test_drag_drop_manager_cancel_no_drag() {
        let manager = DragDropManager::new();
        let result = manager.cancel_drag();
        assert!(matches!(result, Err(DragDropError::NoDragActive)));
    }

    #[test]
    fn test_drag_drop_manager_drop_no_target() {
        let manager = DragDropManager::new();
        let data = DragData::Text("test".to_string());
        manager.start_drag(data, Point::origin()).unwrap();

        let result = manager.drop();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DropEffect::None);
        assert!(!manager.is_dragging());
    }

    #[test]
    fn test_file_drop_target_accepts_files() {
        let target = FileDropTarget::new(0.0, 0.0, 100.0, 100.0);
        let files = vec![PathBuf::from("/test/file.txt")];
        let data = DragData::Files(files);

        assert!(target.accepts(&data));
    }

    #[test]
    fn test_file_drop_target_rejects_text() {
        let target = FileDropTarget::new(0.0, 0.0, 100.0, 100.0);
        let data = DragData::Text("test".to_string());

        assert!(!target.accepts(&data));
    }

    #[test]
    fn test_file_drop_target_with_extensions() {
        let target = FileDropTarget::new(0.0, 0.0, 100.0, 100.0)
            .with_extensions(vec!["txt".to_string(), "md".to_string()]);

        // Should accept .txt file
        let txt_files = vec![PathBuf::from("/test/file.txt")];
        assert!(target.accepts(&DragData::Files(txt_files)));

        // Should accept .MD file (case insensitive)
        let md_files = vec![PathBuf::from("/test/file.MD")];
        assert!(target.accepts(&DragData::Files(md_files)));

        // Should reject .html file
        let html_files = vec![PathBuf::from("/test/file.html")];
        assert!(!target.accepts(&DragData::Files(html_files)));
    }

    #[test]
    fn test_file_drop_target_contains_point() {
        let target = FileDropTarget::new(10.0, 10.0, 100.0, 100.0);

        assert!(target.contains_point(Point::new(50.0, 50.0)));
        assert!(target.contains_point(Point::new(10.0, 10.0)));
        assert!(target.contains_point(Point::new(110.0, 110.0)));
        assert!(!target.contains_point(Point::new(5.0, 5.0)));
        assert!(!target.contains_point(Point::new(150.0, 150.0)));
    }

    #[test]
    fn test_file_drop_target_hover_state() {
        let mut target = FileDropTarget::new(0.0, 0.0, 100.0, 100.0);
        let data = DragData::Files(vec![PathBuf::from("/test")]);

        assert!(!target.is_hovered());

        target.on_drag_enter(&data);
        assert!(target.is_hovered());

        target.on_drag_leave();
        assert!(!target.is_hovered());
    }

    #[test]
    fn test_file_drop_target_drop() {
        let mut target = FileDropTarget::new(0.0, 0.0, 100.0, 100.0);
        let files = vec![PathBuf::from("/test/a.txt"), PathBuf::from("/test/b.txt")];
        let data = DragData::Files(files.clone());

        target.on_drag_enter(&data);
        let result = target.on_drop(data, Point::new(50.0, 50.0));

        assert!(result.is_ok());
        assert_eq!(target.dropped_files().len(), 2);
        assert!(!target.is_hovered());
    }

    #[test]
    fn test_file_drop_target_clear() {
        let mut target = FileDropTarget::new(0.0, 0.0, 100.0, 100.0);
        let files = vec![PathBuf::from("/test/file.txt")];
        let data = DragData::Files(files);

        target.on_drop(data, Point::origin()).unwrap();
        assert_eq!(target.dropped_files().len(), 1);

        target.clear_dropped_files();
        assert!(target.dropped_files().is_empty());
    }

    #[test]
    fn test_text_drag_source_can_drag() {
        let source = TextDragSource::new("hello".to_string());
        assert!(source.can_drag());

        let empty_source = TextDragSource::new(String::new());
        assert!(!empty_source.can_drag());
    }

    #[test]
    fn test_text_drag_source_get_data() {
        let source = TextDragSource::new("hello".to_string());
        let data = source.get_drag_data();

        assert!(data.is_some());
        assert_eq!(data.unwrap().as_text(), Some("hello"));
    }

    #[test]
    fn test_text_drag_source_disabled() {
        let mut source = TextDragSource::new("hello".to_string());
        source.set_can_drag(false);

        assert!(!source.can_drag());
        assert!(source.get_drag_data().is_none());
    }

    #[test]
    fn test_url_drag_source_can_drag() {
        let source = UrlDragSource::new("https://example.com".to_string());
        assert!(source.can_drag());

        let empty_source = UrlDragSource::new(String::new());
        assert!(!empty_source.can_drag());
    }

    #[test]
    fn test_url_drag_source_get_data() {
        let source = UrlDragSource::new("https://example.com".to_string());
        let data = source.get_drag_data();

        assert!(data.is_some());
        assert_eq!(data.unwrap().as_url(), Some("https://example.com"));
    }

    #[test]
    fn test_url_drag_source_effects() {
        let source = UrlDragSource::new("https://example.com".to_string());
        let effects = source.allowed_effects();

        assert!(effects.contains(&DropEffect::Copy));
        assert!(effects.contains(&DropEffect::Link));
    }

    #[test]
    fn test_register_unregister_target() {
        let manager = DragDropManager::new();
        let target = Arc::new(Mutex::new(FileDropTarget::new(0.0, 0.0, 100.0, 100.0)));

        let id = manager.register_target(target.clone()).unwrap();
        assert!(manager.unregister_target(id).is_ok());
    }

    #[test]
    fn test_event_listener() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let manager = DragDropManager::new();
        let event_received = Arc::new(AtomicBool::new(false));
        let event_received_clone = event_received.clone();

        manager.add_listener(move |event| {
            if matches!(event, DragDropEvent::DragStarted { .. }) {
                event_received_clone.store(true, Ordering::SeqCst);
            }
        });

        let data = DragData::Text("test".to_string());
        manager.start_drag(data, Point::origin()).unwrap();

        assert!(event_received.load(Ordering::SeqCst));
    }
}
