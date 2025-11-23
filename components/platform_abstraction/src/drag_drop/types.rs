//! Drag and drop types and data structures
//!
//! This module defines the core types used for cross-platform drag and drop operations.

use super::super::clipboard::ImageData;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A 2D point representing a position
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Point {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

impl Point {
    /// Create a new point
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create a point at the origin (0, 0)
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Calculate the distance to another point
    pub fn distance_to(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<(i32, i32)> for Point {
    fn from((x, y): (i32, i32)) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
        }
    }
}

/// Data that can be dragged and dropped
#[derive(Debug, Clone, PartialEq)]
pub enum DragData {
    /// List of file paths
    Files(Vec<PathBuf>),
    /// Plain text content
    Text(String),
    /// HTML formatted content
    Html(String),
    /// Image data (RGBA format)
    Image(ImageData),
    /// URL string
    Url(String),
}

impl DragData {
    /// Returns true if the drag data is empty
    pub fn is_empty(&self) -> bool {
        match self {
            DragData::Files(files) => files.is_empty(),
            DragData::Text(text) => text.is_empty(),
            DragData::Html(html) => html.is_empty(),
            DragData::Image(img) => img.is_empty(),
            DragData::Url(url) => url.is_empty(),
        }
    }

    /// Returns true if the drag data contains files
    pub fn is_files(&self) -> bool {
        matches!(self, DragData::Files(_))
    }

    /// Returns true if the drag data contains text
    pub fn is_text(&self) -> bool {
        matches!(self, DragData::Text(_))
    }

    /// Returns true if the drag data contains HTML
    pub fn is_html(&self) -> bool {
        matches!(self, DragData::Html(_))
    }

    /// Returns true if the drag data contains an image
    pub fn is_image(&self) -> bool {
        matches!(self, DragData::Image(_))
    }

    /// Returns true if the drag data contains a URL
    pub fn is_url(&self) -> bool {
        matches!(self, DragData::Url(_))
    }

    /// Get the files if available
    pub fn as_files(&self) -> Option<&Vec<PathBuf>> {
        match self {
            DragData::Files(files) => Some(files),
            _ => None,
        }
    }

    /// Get the text content if available
    pub fn as_text(&self) -> Option<&str> {
        match self {
            DragData::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Get the HTML content if available
    pub fn as_html(&self) -> Option<&str> {
        match self {
            DragData::Html(html) => Some(html),
            _ => None,
        }
    }

    /// Get the image data if available
    pub fn as_image(&self) -> Option<&ImageData> {
        match self {
            DragData::Image(img) => Some(img),
            _ => None,
        }
    }

    /// Get the URL if available
    pub fn as_url(&self) -> Option<&str> {
        match self {
            DragData::Url(url) => Some(url),
            _ => None,
        }
    }

    /// Get the format of this drag data
    pub fn format(&self) -> DragFormat {
        match self {
            DragData::Files(_) => DragFormat::Files,
            DragData::Text(_) => DragFormat::Text,
            DragData::Html(_) => DragFormat::Html,
            DragData::Image(_) => DragFormat::Image,
            DragData::Url(_) => DragFormat::Url,
        }
    }
}

impl From<Vec<PathBuf>> for DragData {
    fn from(files: Vec<PathBuf>) -> Self {
        DragData::Files(files)
    }
}

impl From<String> for DragData {
    fn from(text: String) -> Self {
        DragData::Text(text)
    }
}

impl From<&str> for DragData {
    fn from(text: &str) -> Self {
        DragData::Text(text.to_string())
    }
}

impl From<ImageData> for DragData {
    fn from(image: ImageData) -> Self {
        DragData::Image(image)
    }
}

/// Format of drag data for querying availability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DragFormat {
    /// File paths format
    Files,
    /// Plain text format
    Text,
    /// HTML format
    Html,
    /// Image format
    Image,
    /// URL format
    Url,
}

/// Visual effect to show during a drop operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DropEffect {
    /// No drop effect (drop not allowed)
    #[default]
    None,
    /// Copy the data
    Copy,
    /// Move the data
    Move,
    /// Create a link to the data
    Link,
}

impl DropEffect {
    /// Returns true if this effect allows dropping
    pub fn is_allowed(&self) -> bool {
        !matches!(self, DropEffect::None)
    }
}

/// State of an ongoing drag operation
#[derive(Debug, Clone, PartialEq)]
pub struct DragState {
    /// The data being dragged
    pub data: DragData,
    /// Starting position of the drag
    pub start_position: Point,
    /// Current position of the drag
    pub current_position: Point,
    /// The allowed drop effects
    pub allowed_effects: Vec<DropEffect>,
    /// Whether the drag is currently over a valid drop target
    pub over_valid_target: bool,
}

impl DragState {
    /// Create a new drag state
    pub fn new(data: DragData, start_position: Point) -> Self {
        Self {
            data,
            start_position,
            current_position: start_position,
            allowed_effects: vec![DropEffect::Copy, DropEffect::Move, DropEffect::Link],
            over_valid_target: false,
        }
    }

    /// Update the current position
    pub fn update_position(&mut self, position: Point) {
        self.current_position = position;
    }

    /// Calculate the drag distance from the start
    pub fn drag_distance(&self) -> f32 {
        self.start_position.distance_to(&self.current_position)
    }

    /// Set whether we're over a valid target
    pub fn set_over_valid_target(&mut self, valid: bool) {
        self.over_valid_target = valid;
    }
}

/// Visual indicator style for drop targets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DropIndicatorStyle {
    /// No indicator
    #[default]
    None,
    /// Highlight border around the target
    Border,
    /// Overlay on the target area
    Overlay,
    /// Insert indicator (for lists/trees)
    Insert,
}

/// Configuration for drop indicators
#[derive(Debug, Clone, PartialEq)]
pub struct DropIndicator {
    /// The style of the indicator
    pub style: DropIndicatorStyle,
    /// The position for insert indicators
    pub position: Point,
    /// Width of the indicator area
    pub width: f32,
    /// Height of the indicator area
    pub height: f32,
    /// Whether the indicator is currently active
    pub active: bool,
}

impl Default for DropIndicator {
    fn default() -> Self {
        Self {
            style: DropIndicatorStyle::None,
            position: Point::origin(),
            width: 0.0,
            height: 0.0,
            active: false,
        }
    }
}

impl DropIndicator {
    /// Create a new border indicator
    pub fn border(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            style: DropIndicatorStyle::Border,
            position: Point::new(x, y),
            width,
            height,
            active: true,
        }
    }

    /// Create a new overlay indicator
    pub fn overlay(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            style: DropIndicatorStyle::Overlay,
            position: Point::new(x, y),
            width,
            height,
            active: true,
        }
    }

    /// Create a new insert indicator
    pub fn insert(x: f32, y: f32, width: f32) -> Self {
        Self {
            style: DropIndicatorStyle::Insert,
            position: Point::new(x, y),
            width,
            height: 2.0, // Standard insert indicator height
            active: true,
        }
    }

    /// Deactivate the indicator
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Check if a point is within the indicator bounds
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.position.x
            && point.x <= self.position.x + self.width
            && point.y >= self.position.y
            && point.y <= self.position.y + self.height
    }
}

/// Unique identifier for a drop target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DropTargetId(pub u64);

impl DropTargetId {
    /// Create a new drop target ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for DropTargetId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_new() {
        let point = Point::new(10.0, 20.0);
        assert_eq!(point.x, 10.0);
        assert_eq!(point.y, 20.0);
    }

    #[test]
    fn test_point_origin() {
        let origin = Point::origin();
        assert_eq!(origin.x, 0.0);
        assert_eq!(origin.y, 0.0);
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_point_from_tuple_f32() {
        let point: Point = (5.0, 10.0).into();
        assert_eq!(point.x, 5.0);
        assert_eq!(point.y, 10.0);
    }

    #[test]
    fn test_point_from_tuple_i32() {
        let point: Point = (5i32, 10i32).into();
        assert_eq!(point.x, 5.0);
        assert_eq!(point.y, 10.0);
    }

    #[test]
    fn test_drag_data_files() {
        let files = vec![PathBuf::from("/test/file.txt")];
        let data = DragData::Files(files.clone());
        assert!(data.is_files());
        assert!(!data.is_empty());
        assert_eq!(data.as_files(), Some(&files));
        assert_eq!(data.format(), DragFormat::Files);
    }

    #[test]
    fn test_drag_data_text() {
        let data = DragData::Text("hello".to_string());
        assert!(data.is_text());
        assert!(!data.is_empty());
        assert_eq!(data.as_text(), Some("hello"));
        assert_eq!(data.format(), DragFormat::Text);
    }

    #[test]
    fn test_drag_data_html() {
        let data = DragData::Html("<p>hello</p>".to_string());
        assert!(data.is_html());
        assert!(!data.is_empty());
        assert_eq!(data.as_html(), Some("<p>hello</p>"));
        assert_eq!(data.format(), DragFormat::Html);
    }

    #[test]
    fn test_drag_data_image() {
        let image = ImageData::new(vec![0u8; 4], 1, 1);
        let data = DragData::Image(image.clone());
        assert!(data.is_image());
        assert!(!data.is_empty());
        assert_eq!(data.as_image(), Some(&image));
        assert_eq!(data.format(), DragFormat::Image);
    }

    #[test]
    fn test_drag_data_url() {
        let data = DragData::Url("https://example.com".to_string());
        assert!(data.is_url());
        assert!(!data.is_empty());
        assert_eq!(data.as_url(), Some("https://example.com"));
        assert_eq!(data.format(), DragFormat::Url);
    }

    #[test]
    fn test_drag_data_empty() {
        assert!(DragData::Files(vec![]).is_empty());
        assert!(DragData::Text(String::new()).is_empty());
        assert!(DragData::Html(String::new()).is_empty());
        assert!(DragData::Url(String::new()).is_empty());
    }

    #[test]
    fn test_drag_data_from_vec_pathbuf() {
        let files = vec![PathBuf::from("/test")];
        let data: DragData = files.into();
        assert!(data.is_files());
    }

    #[test]
    fn test_drag_data_from_string() {
        let data: DragData = "hello".to_string().into();
        assert!(data.is_text());
    }

    #[test]
    fn test_drag_data_from_str() {
        let data: DragData = "hello".into();
        assert!(data.is_text());
    }

    #[test]
    fn test_drop_effect_is_allowed() {
        assert!(!DropEffect::None.is_allowed());
        assert!(DropEffect::Copy.is_allowed());
        assert!(DropEffect::Move.is_allowed());
        assert!(DropEffect::Link.is_allowed());
    }

    #[test]
    fn test_drag_state_new() {
        let data = DragData::Text("test".to_string());
        let pos = Point::new(10.0, 20.0);
        let state = DragState::new(data.clone(), pos);

        assert_eq!(state.data, data);
        assert_eq!(state.start_position, pos);
        assert_eq!(state.current_position, pos);
        assert!(!state.over_valid_target);
    }

    #[test]
    fn test_drag_state_update_position() {
        let data = DragData::Text("test".to_string());
        let mut state = DragState::new(data, Point::origin());
        state.update_position(Point::new(100.0, 100.0));
        assert_eq!(state.current_position.x, 100.0);
        assert_eq!(state.current_position.y, 100.0);
    }

    #[test]
    fn test_drag_state_distance() {
        let data = DragData::Text("test".to_string());
        let mut state = DragState::new(data, Point::origin());
        state.update_position(Point::new(3.0, 4.0));
        assert!((state.drag_distance() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_drop_indicator_border() {
        let indicator = DropIndicator::border(10.0, 20.0, 100.0, 50.0);
        assert_eq!(indicator.style, DropIndicatorStyle::Border);
        assert!(indicator.active);
        assert_eq!(indicator.position.x, 10.0);
        assert_eq!(indicator.width, 100.0);
    }

    #[test]
    fn test_drop_indicator_overlay() {
        let indicator = DropIndicator::overlay(0.0, 0.0, 50.0, 50.0);
        assert_eq!(indicator.style, DropIndicatorStyle::Overlay);
        assert!(indicator.active);
    }

    #[test]
    fn test_drop_indicator_insert() {
        let indicator = DropIndicator::insert(0.0, 10.0, 100.0);
        assert_eq!(indicator.style, DropIndicatorStyle::Insert);
        assert_eq!(indicator.height, 2.0);
    }

    #[test]
    fn test_drop_indicator_contains() {
        let indicator = DropIndicator::border(10.0, 10.0, 100.0, 100.0);
        assert!(indicator.contains(Point::new(50.0, 50.0)));
        assert!(indicator.contains(Point::new(10.0, 10.0)));
        assert!(indicator.contains(Point::new(110.0, 110.0)));
        assert!(!indicator.contains(Point::new(5.0, 5.0)));
        assert!(!indicator.contains(Point::new(150.0, 150.0)));
    }

    #[test]
    fn test_drop_indicator_deactivate() {
        let mut indicator = DropIndicator::border(0.0, 0.0, 10.0, 10.0);
        assert!(indicator.active);
        indicator.deactivate();
        assert!(!indicator.active);
    }

    #[test]
    fn test_drop_target_id_unique() {
        let id1 = DropTargetId::new();
        let id2 = DropTargetId::new();
        assert_ne!(id1, id2);
    }
}
