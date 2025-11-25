//! Tab Drag and Drop UI
//!
//! Provides visual feedback for tab drag-and-drop operations within the tab bar.
//! This module handles:
//! - Drag initiation and tracking
//! - Visual feedback (ghost tab, drop indicators)
//! - Tab reordering based on drag position
//! - Tab overflow handling with horizontal scroll

use egui::{Pos2, Rect, Ui, Vec2};
use shared_types::TabId;

/// State for tab drag operation
#[derive(Debug, Clone)]
pub struct TabDragState {
    /// The tab currently being dragged, if any
    pub dragging_tab: Option<TabId>,

    /// Start position of the drag
    pub drag_start_pos: Option<Pos2>,

    /// Current drag position
    pub current_drag_pos: Option<Pos2>,

    /// Index where the tab would be dropped if released now
    pub drop_target_index: Option<usize>,

    /// Original index of the dragged tab
    pub original_index: Option<usize>,
}

impl Default for TabDragState {
    fn default() -> Self {
        Self {
            dragging_tab: None,
            drag_start_pos: None,
            current_drag_pos: None,
            drop_target_index: None,
            original_index: None,
        }
    }
}

impl TabDragState {
    /// Create a new tab drag state
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a drag is currently active
    pub fn is_dragging(&self) -> bool {
        self.dragging_tab.is_some()
    }

    /// Start dragging a tab
    pub fn start_drag(&mut self, tab_id: TabId, pos: Pos2, index: usize) {
        self.dragging_tab = Some(tab_id);
        self.drag_start_pos = Some(pos);
        self.current_drag_pos = Some(pos);
        self.original_index = Some(index);
        self.drop_target_index = Some(index);
    }

    /// Update drag position
    pub fn update_drag(&mut self, pos: Pos2) {
        if self.is_dragging() {
            self.current_drag_pos = Some(pos);
        }
    }

    /// End the drag operation and return the target index if moved
    pub fn end_drag(&mut self) -> Option<(TabId, usize, usize)> {
        let result = if let (Some(tab_id), Some(original), Some(target)) =
            (self.dragging_tab, self.original_index, self.drop_target_index) {
            if original != target {
                Some((tab_id, original, target))
            } else {
                None
            }
        } else {
            None
        };

        // Reset state
        self.dragging_tab = None;
        self.drag_start_pos = None;
        self.current_drag_pos = None;
        self.drop_target_index = None;
        self.original_index = None;

        result
    }

    /// Cancel the drag operation
    pub fn cancel_drag(&mut self) {
        self.dragging_tab = None;
        self.drag_start_pos = None;
        self.current_drag_pos = None;
        self.drop_target_index = None;
        self.original_index = None;
    }

    /// Calculate drop target index based on drag position and tab rects
    pub fn calculate_drop_target(&mut self, tab_rects: &[(TabId, Rect)]) {
        if let Some(drag_pos) = self.current_drag_pos {
            // Find the tab rect that contains the drag position
            let mut target_index = None;

            for (index, (_tab_id, rect)) in tab_rects.iter().enumerate() {
                // Check if drag position is within this tab's horizontal bounds
                let rect_center_x = rect.center().x;

                if drag_pos.x < rect_center_x {
                    target_index = Some(index);
                    break;
                }
            }

            // If no match found, drop at the end
            if target_index.is_none() {
                target_index = Some(tab_rects.len());
            }

            self.drop_target_index = target_index;
        }
    }
}

/// Visual configuration for tab drag rendering
#[derive(Debug, Clone)]
pub struct TabDragVisuals {
    /// Opacity for the ghost tab being dragged (0.0 - 1.0)
    pub ghost_opacity: f32,

    /// Color for drop indicator line
    pub drop_indicator_color: egui::Color32,

    /// Width of drop indicator line
    pub drop_indicator_width: f32,

    /// Minimum drag distance before starting drag (prevents accidental drags)
    pub min_drag_distance: f32,
}

impl Default for TabDragVisuals {
    fn default() -> Self {
        Self {
            ghost_opacity: 0.5,
            drop_indicator_color: egui::Color32::from_rgb(0, 120, 215), // Blue
            drop_indicator_width: 2.0,
            min_drag_distance: 5.0,
        }
    }
}

/// Render a ghost tab at the drag position
pub fn render_ghost_tab(
    ui: &mut Ui,
    tab_title: &str,
    drag_pos: Pos2,
    visuals: &TabDragVisuals,
) {
    let painter = ui.painter();

    // Measure tab size
    let font_id = egui::TextStyle::Body.resolve(ui.style());
    let text_color = ui.style().visuals.text_color();
    let galley = painter.layout_no_wrap(
        tab_title.to_string(),
        font_id,
        text_color,
    );

    let tab_width = galley.size().x + 40.0; // Extra padding for tab shape
    let tab_height = 30.0;

    // Calculate ghost tab rect centered on drag position
    let ghost_rect = Rect::from_center_size(
        drag_pos,
        Vec2::new(tab_width, tab_height),
    );

    // Draw ghost tab background with opacity
    let mut bg_color = ui.style().visuals.widgets.inactive.bg_fill;
    bg_color = bg_color.linear_multiply(visuals.ghost_opacity);

    painter.rect_filled(
        ghost_rect,
        5.0, // Rounded corners
        bg_color,
    );

    // Draw tab border with opacity
    let mut border_color = ui.style().visuals.widgets.inactive.bg_stroke.color;
    border_color = border_color.linear_multiply(visuals.ghost_opacity);

    painter.rect_stroke(
        ghost_rect,
        5.0,
        egui::Stroke::new(1.0, border_color),
    );

    // Draw tab title with opacity
    let mut text_color = ui.style().visuals.text_color();
    text_color = text_color.linear_multiply(visuals.ghost_opacity);

    painter.galley(
        Pos2::new(
            ghost_rect.center().x - galley.size().x / 2.0,
            ghost_rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        text_color,
    );
}

/// Render a drop indicator line between tabs
pub fn render_drop_indicator(
    ui: &mut Ui,
    tab_rects: &[(TabId, Rect)],
    drop_index: usize,
    visuals: &TabDragVisuals,
) {
    let painter = ui.painter();

    // Calculate indicator position
    let indicator_x = if drop_index == 0 {
        // Before first tab
        if let Some((_, rect)) = tab_rects.first() {
            rect.left()
        } else {
            return;
        }
    } else if drop_index >= tab_rects.len() {
        // After last tab
        if let Some((_, rect)) = tab_rects.last() {
            rect.right()
        } else {
            return;
        }
    } else {
        // Between tabs
        if let Some((_, rect)) = tab_rects.get(drop_index - 1) {
            rect.right()
        } else {
            return;
        }
    };

    // Get tab bar bounds from first tab (assumes all tabs at same Y position)
    if let Some((_, first_rect)) = tab_rects.first() {
        let top = first_rect.top();
        let bottom = first_rect.bottom();

        // Draw vertical line
        painter.line_segment(
            [
                Pos2::new(indicator_x, top),
                Pos2::new(indicator_x, bottom),
            ],
            egui::Stroke::new(visuals.drop_indicator_width, visuals.drop_indicator_color),
        );

        // Draw small arrow markers at top and bottom
        let arrow_size = 5.0;

        // Top arrow (pointing down)
        painter.add(egui::Shape::convex_polygon(
            vec![
                Pos2::new(indicator_x, top),
                Pos2::new(indicator_x - arrow_size, top - arrow_size),
                Pos2::new(indicator_x + arrow_size, top - arrow_size),
            ],
            visuals.drop_indicator_color,
            egui::Stroke::NONE,
        ));

        // Bottom arrow (pointing up)
        painter.add(egui::Shape::convex_polygon(
            vec![
                Pos2::new(indicator_x, bottom),
                Pos2::new(indicator_x - arrow_size, bottom + arrow_size),
                Pos2::new(indicator_x + arrow_size, bottom + arrow_size),
            ],
            visuals.drop_indicator_color,
            egui::Stroke::NONE,
        ));
    }
}

/// Handle tab overflow by adding scroll functionality
pub struct TabOverflowHandler {
    /// Current scroll offset
    pub scroll_offset: f32,

    /// Maximum scroll offset
    pub max_scroll: f32,

    /// Width available for tabs
    pub available_width: f32,

    /// Total width needed for all tabs
    pub total_tab_width: f32,
}

impl Default for TabOverflowHandler {
    fn default() -> Self {
        Self {
            scroll_offset: 0.0,
            max_scroll: 0.0,
            available_width: 0.0,
            total_tab_width: 0.0,
        }
    }
}

impl TabOverflowHandler {
    /// Create a new overflow handler
    pub fn new() -> Self {
        Self::default()
    }

    /// Update available width and total tab width
    pub fn update(&mut self, available_width: f32, total_tab_width: f32) {
        self.available_width = available_width;
        self.total_tab_width = total_tab_width;

        // Calculate max scroll (can't be negative)
        self.max_scroll = (total_tab_width - available_width).max(0.0);

        // Clamp current scroll offset
        self.scroll_offset = self.scroll_offset.clamp(0.0, self.max_scroll);
    }

    /// Check if tabs overflow (need scrolling)
    pub fn is_overflowing(&self) -> bool {
        self.total_tab_width > self.available_width
    }

    /// Scroll left by a delta
    pub fn scroll_left(&mut self, delta: f32) {
        self.scroll_offset = (self.scroll_offset - delta).max(0.0);
    }

    /// Scroll right by a delta
    pub fn scroll_right(&mut self, delta: f32) {
        self.scroll_offset = (self.scroll_offset + delta).min(self.max_scroll);
    }

    /// Handle scroll wheel input
    pub fn handle_scroll(&mut self, scroll_delta: Vec2) {
        // Horizontal scroll
        if scroll_delta.x != 0.0 {
            self.scroll_offset = (self.scroll_offset - scroll_delta.x * 10.0)
                .clamp(0.0, self.max_scroll);
        }

        // Vertical scroll can also scroll horizontally when shift is held
        // This is handled by egui's scroll_delta which already converts shift+vertical to horizontal
    }

    /// Get the current scroll offset
    pub fn offset(&self) -> f32 {
        self.scroll_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_drag_state_default() {
        let state = TabDragState::default();
        assert!(!state.is_dragging());
        assert!(state.dragging_tab.is_none());
    }

    #[test]
    fn test_start_drag() {
        let mut state = TabDragState::new();
        let tab_id = TabId::new();
        let pos = Pos2::new(100.0, 50.0);

        state.start_drag(tab_id, pos, 2);

        assert!(state.is_dragging());
        assert_eq!(state.dragging_tab, Some(tab_id));
        assert_eq!(state.drag_start_pos, Some(pos));
        assert_eq!(state.original_index, Some(2));
    }

    #[test]
    fn test_end_drag_with_movement() {
        let mut state = TabDragState::new();
        let tab_id = TabId::new();
        let pos = Pos2::new(100.0, 50.0);

        state.start_drag(tab_id, pos, 2);
        state.drop_target_index = Some(4);

        let result = state.end_drag();

        assert!(result.is_some());
        let (id, original, target) = result.unwrap();
        assert_eq!(id, tab_id);
        assert_eq!(original, 2);
        assert_eq!(target, 4);
        assert!(!state.is_dragging());
    }

    #[test]
    fn test_end_drag_without_movement() {
        let mut state = TabDragState::new();
        let tab_id = TabId::new();
        let pos = Pos2::new(100.0, 50.0);

        state.start_drag(tab_id, pos, 2);
        // drop_target_index remains 2 (no movement)

        let result = state.end_drag();

        assert!(result.is_none());
        assert!(!state.is_dragging());
    }

    #[test]
    fn test_cancel_drag() {
        let mut state = TabDragState::new();
        let tab_id = TabId::new();
        let pos = Pos2::new(100.0, 50.0);

        state.start_drag(tab_id, pos, 2);
        state.cancel_drag();

        assert!(!state.is_dragging());
        assert!(state.dragging_tab.is_none());
    }

    #[test]
    fn test_overflow_handler_no_overflow() {
        let mut handler = TabOverflowHandler::new();
        handler.update(500.0, 400.0);

        assert!(!handler.is_overflowing());
        assert_eq!(handler.max_scroll, 0.0);
    }

    #[test]
    fn test_overflow_handler_with_overflow() {
        let mut handler = TabOverflowHandler::new();
        handler.update(500.0, 800.0);

        assert!(handler.is_overflowing());
        assert_eq!(handler.max_scroll, 300.0);
    }

    #[test]
    fn test_overflow_handler_scroll() {
        let mut handler = TabOverflowHandler::new();
        handler.update(500.0, 800.0);

        handler.scroll_right(100.0);
        assert_eq!(handler.offset(), 100.0);

        handler.scroll_right(250.0);
        assert_eq!(handler.offset(), 300.0); // Clamped to max_scroll

        handler.scroll_left(50.0);
        assert_eq!(handler.offset(), 250.0);

        handler.scroll_left(300.0);
        assert_eq!(handler.offset(), 0.0); // Clamped to 0
    }

    #[test]
    fn test_tab_drag_visuals_default() {
        let visuals = TabDragVisuals::default();
        assert_eq!(visuals.ghost_opacity, 0.5);
        assert!(visuals.min_drag_distance > 0.0);
    }
}
