//! Picture-in-Picture (PiP) support for video content
//!
//! This module provides floating video window management for the browser shell,
//! allowing videos to be displayed in always-on-top floating windows while
//! the user continues browsing.
//!
//! # Core Types
//!
//! - [`PipWindow`]: Represents a floating video window
//! - [`PipManager`]: Coordinates multiple PiP sessions
//! - [`PipConfig`]: Configuration for PiP window behavior
//! - [`PipBounds`]: Position and size with aspect ratio preservation
//!
//! # Example
//!
//! ```rust,ignore
//! use window_manager::pip::{PipManager, PipConfig, PipVideoSource};
//!
//! let mut manager = PipManager::new();
//! let source = PipVideoSource::new(tab_id, "video-element-1".to_string());
//! let config = PipConfig::default();
//!
//! let pip_id = manager.create_pip_window(source, config).await?;
//! manager.toggle_play_pause(pip_id)?;
//! ```

use serde::{Deserialize, Serialize};
use shared_types::{TabId, WindowError};
use std::collections::HashMap;
use uuid::Uuid;

/// Default PiP window width in pixels
pub const DEFAULT_PIP_WIDTH: u32 = 320;

/// Default PiP window height in pixels
pub const DEFAULT_PIP_HEIGHT: u32 = 180;

/// Minimum PiP window width in pixels
pub const MIN_PIP_WIDTH: u32 = 200;

/// Minimum PiP window height in pixels
pub const MIN_PIP_HEIGHT: u32 = 112;

/// Maximum PiP window width in pixels
pub const MAX_PIP_WIDTH: u32 = 800;

/// Maximum PiP window height in pixels
pub const MAX_PIP_HEIGHT: u32 = 450;

/// Default opacity for PiP windows (0.0 - 1.0)
pub const DEFAULT_PIP_OPACITY: f32 = 1.0;

// ============================================================================
// ID Types
// ============================================================================

/// Unique identifier for Picture-in-Picture windows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipWindowId(Uuid);

impl PipWindowId {
    /// Create a new random PipWindowId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for PipWindowId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PipWindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pip-{}", self.0)
    }
}

// ============================================================================
// State and Source Types
// ============================================================================

/// Current state of a PiP window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipState {
    /// PiP window is active and playing video
    Active,
    /// PiP window is minimized/collapsed
    Minimized,
    /// PiP window is visible but video is paused
    Paused,
}

impl Default for PipState {
    fn default() -> Self {
        Self::Active
    }
}

/// Source information for a PiP video
///
/// Identifies which tab and video element the PiP content is sourced from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipVideoSource {
    /// ID of the tab containing the video
    pub tab_id: TabId,
    /// DOM element ID or selector for the video element
    pub video_element_id: String,
    /// Optional URL of the video (for display purposes)
    pub video_url: Option<String>,
}

impl PipVideoSource {
    /// Create a new PipVideoSource
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab containing the video
    /// * `video_element_id` - DOM element ID or selector for the video
    pub fn new(tab_id: TabId, video_element_id: String) -> Self {
        Self {
            tab_id,
            video_element_id,
            video_url: None,
        }
    }

    /// Create a PipVideoSource with a video URL
    pub fn with_url(tab_id: TabId, video_element_id: String, url: String) -> Self {
        Self {
            tab_id,
            video_element_id,
            video_url: Some(url),
        }
    }
}

// ============================================================================
// Bounds and Position
// ============================================================================

/// Position on screen in pixels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipPosition {
    /// X coordinate (pixels from left edge)
    pub x: i32,
    /// Y coordinate (pixels from top edge)
    pub y: i32,
}

impl PipPosition {
    /// Create a new position
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Default for PipPosition {
    fn default() -> Self {
        // Default to bottom-right corner area
        Self { x: 100, y: 100 }
    }
}

/// Size dimensions in pixels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipSize {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl PipSize {
    /// Create a new size
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Calculate aspect ratio (width / height)
    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0 {
            return 0.0;
        }
        self.width as f64 / self.height as f64
    }
}

impl Default for PipSize {
    fn default() -> Self {
        Self {
            width: DEFAULT_PIP_WIDTH,
            height: DEFAULT_PIP_HEIGHT,
        }
    }
}

/// Bounds for a PiP window with aspect ratio preservation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PipBounds {
    /// Current position of the window
    pub position: PipPosition,
    /// Current size of the window
    pub size: PipSize,
    /// Original aspect ratio to preserve during resize
    pub aspect_ratio: f64,
    /// Whether to preserve aspect ratio when resizing
    pub preserve_aspect_ratio: bool,
}

impl PipBounds {
    /// Create new bounds with the given parameters
    pub fn new(position: PipPosition, size: PipSize) -> Self {
        let aspect_ratio = size.aspect_ratio();
        Self {
            position,
            size,
            aspect_ratio,
            preserve_aspect_ratio: true,
        }
    }

    /// Create bounds with a specific aspect ratio
    pub fn with_aspect_ratio(mut self, ratio: f64) -> Self {
        self.aspect_ratio = ratio;
        self
    }

    /// Set whether to preserve aspect ratio
    pub fn preserve_ratio(mut self, preserve: bool) -> Self {
        self.preserve_aspect_ratio = preserve;
        self
    }

    /// Resize the bounds while preserving aspect ratio if enabled
    ///
    /// # Arguments
    ///
    /// * `new_width` - Desired new width
    /// * `new_height` - Desired new height (ignored if preserving aspect ratio)
    ///
    /// # Returns
    ///
    /// The actual size after applying constraints
    pub fn resize(&mut self, new_width: u32, new_height: u32) -> PipSize {
        let (width, height) = if self.preserve_aspect_ratio && self.aspect_ratio > 0.0 {
            // Calculate height based on width to preserve aspect ratio
            let calculated_height = (new_width as f64 / self.aspect_ratio).round() as u32;
            (new_width, calculated_height)
        } else {
            (new_width, new_height)
        };

        // Apply size constraints
        let clamped_width = width.clamp(MIN_PIP_WIDTH, MAX_PIP_WIDTH);
        let clamped_height = height.clamp(MIN_PIP_HEIGHT, MAX_PIP_HEIGHT);

        self.size = PipSize::new(clamped_width, clamped_height);
        self.size
    }

    /// Move the window to a new position
    pub fn move_to(&mut self, x: i32, y: i32) {
        self.position = PipPosition::new(x, y);
    }

    /// Get the right edge x-coordinate
    pub fn right(&self) -> i32 {
        self.position.x + self.size.width as i32
    }

    /// Get the bottom edge y-coordinate
    pub fn bottom(&self) -> i32 {
        self.position.y + self.size.height as i32
    }
}

impl Default for PipBounds {
    fn default() -> Self {
        Self::new(PipPosition::default(), PipSize::default())
    }
}

// ============================================================================
// Controls
// ============================================================================

/// Available control actions for a PiP window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipControlAction {
    /// Play the video
    Play,
    /// Pause the video
    Pause,
    /// Toggle play/pause state
    TogglePlayPause,
    /// Mute the audio
    Mute,
    /// Unmute the audio
    Unmute,
    /// Toggle mute state
    ToggleMute,
    /// Close the PiP window and return video to tab
    Close,
    /// Expand back to the original tab
    Expand,
    /// Minimize the PiP window
    Minimize,
    /// Restore from minimized state
    Restore,
}

/// State of PiP window controls
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipControls {
    /// Whether video is currently playing
    pub is_playing: bool,
    /// Whether audio is muted
    pub is_muted: bool,
    /// Current volume level (0.0 - 1.0)
    pub volume: f32,
    /// Whether controls overlay is visible
    pub controls_visible: bool,
    /// Timeout before controls auto-hide (milliseconds)
    pub auto_hide_ms: u32,
}

impl PipControls {
    /// Create new controls with default state
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle play/pause state
    pub fn toggle_play_pause(&mut self) -> bool {
        self.is_playing = !self.is_playing;
        self.is_playing
    }

    /// Toggle mute state
    pub fn toggle_mute(&mut self) -> bool {
        self.is_muted = !self.is_muted;
        self.is_muted
    }

    /// Set volume level (clamped to 0.0 - 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Show control overlay
    pub fn show_controls(&mut self) {
        self.controls_visible = true;
    }

    /// Hide control overlay
    pub fn hide_controls(&mut self) {
        self.controls_visible = false;
    }
}

impl Default for PipControls {
    fn default() -> Self {
        Self {
            is_playing: true,
            is_muted: false,
            volume: 1.0,
            controls_visible: true,
            auto_hide_ms: 3000,
        }
    }
}

// ============================================================================
// Configuration
// ============================================================================

/// Corner positions for PiP window anchoring
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PipCorner {
    /// Top-left corner
    TopLeft,
    /// Top-right corner
    TopRight,
    /// Bottom-left corner
    BottomLeft,
    /// Bottom-right corner (default)
    #[default]
    BottomRight,
}

/// Configuration for PiP window behavior
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipConfig {
    /// Initial size of the PiP window
    pub initial_size: PipSize,
    /// Initial position (None for auto-placement)
    pub initial_position: Option<PipPosition>,
    /// Corner to anchor to when auto-placing
    pub anchor_corner: PipCorner,
    /// Whether window should be always-on-top
    pub always_on_top: bool,
    /// Window opacity (0.0 - 1.0)
    pub opacity: f32,
    /// Whether to show window decorations (title bar, etc.)
    pub show_decorations: bool,
    /// Whether to preserve video aspect ratio
    pub preserve_aspect_ratio: bool,
    /// Whether to start playing immediately
    pub auto_play: bool,
    /// Margin from screen edges when auto-placing (pixels)
    pub edge_margin: u32,
}

impl PipConfig {
    /// Create a new PipConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder method: set initial size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.initial_size = PipSize::new(width, height);
        self
    }

    /// Builder method: set initial position
    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.initial_position = Some(PipPosition::new(x, y));
        self
    }

    /// Builder method: set anchor corner
    pub fn with_anchor(mut self, corner: PipCorner) -> Self {
        self.anchor_corner = corner;
        self
    }

    /// Builder method: set always-on-top
    pub fn with_always_on_top(mut self, on_top: bool) -> Self {
        self.always_on_top = on_top;
        self
    }

    /// Builder method: set opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

impl Default for PipConfig {
    fn default() -> Self {
        Self {
            initial_size: PipSize::default(),
            initial_position: None,
            anchor_corner: PipCorner::default(),
            always_on_top: true,
            opacity: DEFAULT_PIP_OPACITY,
            show_decorations: false,
            preserve_aspect_ratio: true,
            auto_play: true,
            edge_margin: 16,
        }
    }
}

// ============================================================================
// Video Extraction Interface
// ============================================================================

/// Error type for video extraction operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoExtractionError {
    /// Video element was not found in the page
    ElementNotFound(String),
    /// Video is not playable (format, DRM, etc.)
    NotPlayable(String),
    /// Video extraction is not supported for this source
    NotSupported(String),
    /// Tab does not exist or is not accessible
    TabNotAccessible(TabId),
    /// Internal extraction error
    InternalError(String),
}

impl std::fmt::Display for VideoExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ElementNotFound(id) => write!(f, "Video element not found: {}", id),
            Self::NotPlayable(reason) => write!(f, "Video not playable: {}", reason),
            Self::NotSupported(reason) => write!(f, "Video extraction not supported: {}", reason),
            Self::TabNotAccessible(id) => write!(f, "Tab not accessible: {:?}", id),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for VideoExtractionError {}

/// Information about an extracted video
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedVideoInfo {
    /// Original video width
    pub width: u32,
    /// Original video height
    pub height: u32,
    /// Video duration in seconds (if known)
    pub duration_secs: Option<f64>,
    /// Current playback position in seconds
    pub current_time_secs: f64,
    /// Whether video has audio track
    pub has_audio: bool,
    /// Video title (if available)
    pub title: Option<String>,
}

/// Trait for video extraction from webview
///
/// This trait defines the interface for extracting video content from
/// a browser tab to be displayed in a PiP window. Implementations will
/// be provided by the webview integration layer.
pub trait VideoExtractor: Send + Sync {
    /// Check if a video element can be extracted for PiP
    fn can_extract(&self, source: &PipVideoSource) -> Result<bool, VideoExtractionError>;

    /// Extract video information from a source
    fn get_video_info(
        &self,
        source: &PipVideoSource,
    ) -> Result<ExtractedVideoInfo, VideoExtractionError>;

    /// Start video extraction for PiP display
    fn start_extraction(&mut self, source: &PipVideoSource) -> Result<(), VideoExtractionError>;

    /// Stop video extraction and return to normal playback
    fn stop_extraction(&mut self, source: &PipVideoSource) -> Result<(), VideoExtractionError>;
}

/// Stub implementation of VideoExtractor for testing
#[derive(Debug, Default)]
pub struct StubVideoExtractor {
    /// Simulated extraction state
    active_extractions: Vec<PipVideoSource>,
}

impl StubVideoExtractor {
    /// Create a new stub extractor
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a source is currently being extracted
    pub fn is_extracting(&self, source: &PipVideoSource) -> bool {
        self.active_extractions.contains(source)
    }
}

impl VideoExtractor for StubVideoExtractor {
    fn can_extract(&self, _source: &PipVideoSource) -> Result<bool, VideoExtractionError> {
        // Stub: always returns true
        Ok(true)
    }

    fn get_video_info(
        &self,
        _source: &PipVideoSource,
    ) -> Result<ExtractedVideoInfo, VideoExtractionError> {
        // Stub: return default video info
        Ok(ExtractedVideoInfo {
            width: 1920,
            height: 1080,
            duration_secs: Some(300.0),
            current_time_secs: 0.0,
            has_audio: true,
            title: Some("Test Video".to_string()),
        })
    }

    fn start_extraction(&mut self, source: &PipVideoSource) -> Result<(), VideoExtractionError> {
        if !self.active_extractions.contains(source) {
            self.active_extractions.push(source.clone());
        }
        Ok(())
    }

    fn stop_extraction(&mut self, source: &PipVideoSource) -> Result<(), VideoExtractionError> {
        self.active_extractions.retain(|s| s != source);
        Ok(())
    }
}

// ============================================================================
// PiP Window
// ============================================================================

/// Errors specific to PiP operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PipError {
    /// PiP window not found
    NotFound(PipWindowId),
    /// Maximum number of PiP windows reached
    MaxWindowsReached(usize),
    /// Video extraction failed
    ExtractionFailed(String),
    /// Invalid configuration
    InvalidConfig(String),
    /// Window operation failed
    WindowError(String),
}

impl std::fmt::Display for PipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "PiP window not found: {}", id),
            Self::MaxWindowsReached(max) => {
                write!(f, "Maximum PiP windows ({}) already open", max)
            }
            Self::ExtractionFailed(reason) => write!(f, "Video extraction failed: {}", reason),
            Self::InvalidConfig(reason) => write!(f, "Invalid PiP configuration: {}", reason),
            Self::WindowError(reason) => write!(f, "Window operation failed: {}", reason),
        }
    }
}

impl std::error::Error for PipError {}

impl From<WindowError> for PipError {
    fn from(err: WindowError) -> Self {
        PipError::WindowError(err.to_string())
    }
}

/// Represents a Picture-in-Picture floating video window
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipWindow {
    /// Unique identifier for this PiP window
    pub id: PipWindowId,
    /// Current state of the PiP window
    pub state: PipState,
    /// Source of the video content
    pub source: PipVideoSource,
    /// Window bounds (position and size)
    pub bounds: PipBounds,
    /// Control state
    pub controls: PipControls,
    /// Configuration settings
    pub config: PipConfig,
    /// Timestamp when PiP was created (Unix epoch millis)
    pub created_at: u64,
}

impl PipWindow {
    /// Create a new PiP window with the given source and configuration
    pub fn new(source: PipVideoSource, config: PipConfig) -> Self {
        let position = config.initial_position.unwrap_or_default();
        let size = config.initial_size;
        let aspect_ratio = size.aspect_ratio();

        let bounds = PipBounds::new(position, size)
            .with_aspect_ratio(aspect_ratio)
            .preserve_ratio(config.preserve_aspect_ratio);

        let state = if config.auto_play {
            PipState::Active
        } else {
            PipState::Paused
        };

        let mut controls = PipControls::new();
        controls.is_playing = config.auto_play;

        Self {
            id: PipWindowId::new(),
            state,
            source,
            bounds,
            controls,
            config,
            created_at: Self::current_timestamp(),
        }
    }

    /// Get current timestamp in milliseconds
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Check if the PiP window is currently playing
    pub fn is_playing(&self) -> bool {
        self.state == PipState::Active && self.controls.is_playing
    }

    /// Check if the PiP window is minimized
    pub fn is_minimized(&self) -> bool {
        self.state == PipState::Minimized
    }

    /// Toggle play/pause state
    pub fn toggle_play_pause(&mut self) -> PipState {
        let is_playing = self.controls.toggle_play_pause();
        self.state = if is_playing {
            PipState::Active
        } else {
            PipState::Paused
        };
        self.state
    }

    /// Minimize the PiP window
    pub fn minimize(&mut self) {
        self.state = PipState::Minimized;
    }

    /// Restore from minimized state
    pub fn restore(&mut self) {
        self.state = if self.controls.is_playing {
            PipState::Active
        } else {
            PipState::Paused
        };
    }

    /// Resize the window
    pub fn resize(&mut self, width: u32, height: u32) -> PipSize {
        self.bounds.resize(width, height)
    }

    /// Move the window to a new position
    pub fn move_to(&mut self, x: i32, y: i32) {
        self.bounds.move_to(x, y);
    }

    /// Set window opacity
    pub fn set_opacity(&mut self, opacity: f32) {
        self.config.opacity = opacity.clamp(0.0, 1.0);
    }

    /// Handle a control action
    pub fn handle_action(&mut self, action: PipControlAction) -> PipState {
        match action {
            PipControlAction::Play => {
                self.controls.is_playing = true;
                self.state = PipState::Active;
            }
            PipControlAction::Pause => {
                self.controls.is_playing = false;
                self.state = PipState::Paused;
            }
            PipControlAction::TogglePlayPause => {
                return self.toggle_play_pause();
            }
            PipControlAction::Mute => {
                self.controls.is_muted = true;
            }
            PipControlAction::Unmute => {
                self.controls.is_muted = false;
            }
            PipControlAction::ToggleMute => {
                self.controls.toggle_mute();
            }
            PipControlAction::Minimize => {
                self.minimize();
            }
            PipControlAction::Restore => {
                self.restore();
            }
            PipControlAction::Close | PipControlAction::Expand => {
                // These are handled by PipManager
            }
        }
        self.state
    }
}

// ============================================================================
// PiP Manager
// ============================================================================

/// Maximum number of concurrent PiP windows
pub const MAX_PIP_WINDOWS: usize = 4;

/// Manages multiple Picture-in-Picture sessions
pub struct PipManager {
    /// Active PiP windows
    windows: HashMap<PipWindowId, PipWindow>,
    /// Maximum allowed PiP windows
    max_windows: usize,
    /// Video extractor (stub for webview integration)
    extractor: Box<dyn VideoExtractor>,
}

impl std::fmt::Debug for PipManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipManager")
            .field("windows", &self.windows)
            .field("max_windows", &self.max_windows)
            .field("extractor", &"<dyn VideoExtractor>")
            .finish()
    }
}

impl PipManager {
    /// Create a new PipManager with default settings
    pub fn new() -> Self {
        Self::with_extractor(Box::new(StubVideoExtractor::new()))
    }

    /// Create a new PipManager with a custom video extractor
    pub fn with_extractor(extractor: Box<dyn VideoExtractor>) -> Self {
        Self {
            windows: HashMap::new(),
            max_windows: MAX_PIP_WINDOWS,
            extractor,
        }
    }

    /// Set the maximum number of PiP windows
    pub fn set_max_windows(&mut self, max: usize) {
        self.max_windows = max;
    }

    /// Get the number of active PiP windows
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Check if a new PiP window can be created
    pub fn can_create_pip(&self) -> bool {
        self.windows.len() < self.max_windows
    }

    /// Create a new PiP window
    ///
    /// # Arguments
    ///
    /// * `source` - Video source information
    /// * `config` - PiP window configuration
    ///
    /// # Returns
    ///
    /// * `Ok(PipWindowId)` - ID of the newly created PiP window
    /// * `Err(PipError)` - If creation fails
    pub fn create_pip_window(
        &mut self,
        source: PipVideoSource,
        config: PipConfig,
    ) -> Result<PipWindowId, PipError> {
        // Check window limit
        if self.windows.len() >= self.max_windows {
            return Err(PipError::MaxWindowsReached(self.max_windows));
        }

        // Check if video can be extracted
        if !self
            .extractor
            .can_extract(&source)
            .map_err(|e| PipError::ExtractionFailed(e.to_string()))?
        {
            return Err(PipError::ExtractionFailed(
                "Video cannot be extracted".to_string(),
            ));
        }

        // Create PiP window
        let pip_window = PipWindow::new(source, config);
        let pip_id = pip_window.id;

        self.windows.insert(pip_id, pip_window);

        Ok(pip_id)
    }

    /// Close a PiP window
    ///
    /// # Arguments
    ///
    /// * `pip_id` - ID of the PiP window to close
    ///
    /// # Returns
    ///
    /// * `Ok(PipWindow)` - The closed PiP window
    /// * `Err(PipError)` - If window not found
    pub fn close_pip_window(&mut self, pip_id: PipWindowId) -> Result<PipWindow, PipError> {
        self.windows
            .remove(&pip_id)
            .ok_or(PipError::NotFound(pip_id))
    }

    /// Get a reference to a PiP window
    pub fn get_pip_window(&self, pip_id: PipWindowId) -> Option<&PipWindow> {
        self.windows.get(&pip_id)
    }

    /// Get a mutable reference to a PiP window
    pub fn get_pip_window_mut(&mut self, pip_id: PipWindowId) -> Option<&mut PipWindow> {
        self.windows.get_mut(&pip_id)
    }

    /// Get all active PiP window IDs
    pub fn get_all_pip_ids(&self) -> Vec<PipWindowId> {
        self.windows.keys().copied().collect()
    }

    /// Get all PiP windows for a specific tab
    pub fn get_pip_windows_for_tab(&self, tab_id: TabId) -> Vec<&PipWindow> {
        self.windows
            .values()
            .filter(|w| w.source.tab_id == tab_id)
            .collect()
    }

    /// Toggle play/pause for a PiP window
    pub fn toggle_play_pause(&mut self, pip_id: PipWindowId) -> Result<PipState, PipError> {
        let window = self
            .windows
            .get_mut(&pip_id)
            .ok_or(PipError::NotFound(pip_id))?;
        Ok(window.toggle_play_pause())
    }

    /// Handle a control action for a PiP window
    pub fn handle_action(
        &mut self,
        pip_id: PipWindowId,
        action: PipControlAction,
    ) -> Result<PipState, PipError> {
        // Handle close/expand specially
        if matches!(action, PipControlAction::Close | PipControlAction::Expand) {
            self.close_pip_window(pip_id)?;
            return Ok(PipState::Paused); // Window is gone
        }

        let window = self
            .windows
            .get_mut(&pip_id)
            .ok_or(PipError::NotFound(pip_id))?;
        Ok(window.handle_action(action))
    }

    /// Resize a PiP window
    pub fn resize_pip_window(
        &mut self,
        pip_id: PipWindowId,
        width: u32,
        height: u32,
    ) -> Result<PipSize, PipError> {
        let window = self
            .windows
            .get_mut(&pip_id)
            .ok_or(PipError::NotFound(pip_id))?;
        Ok(window.resize(width, height))
    }

    /// Move a PiP window
    pub fn move_pip_window(
        &mut self,
        pip_id: PipWindowId,
        x: i32,
        y: i32,
    ) -> Result<(), PipError> {
        let window = self
            .windows
            .get_mut(&pip_id)
            .ok_or(PipError::NotFound(pip_id))?;
        window.move_to(x, y);
        Ok(())
    }

    /// Close all PiP windows for a specific tab
    pub fn close_all_for_tab(&mut self, tab_id: TabId) -> Vec<PipWindow> {
        let ids_to_remove: Vec<_> = self
            .windows
            .iter()
            .filter(|(_, w)| w.source.tab_id == tab_id)
            .map(|(id, _)| *id)
            .collect();

        ids_to_remove
            .into_iter()
            .filter_map(|id| self.windows.remove(&id))
            .collect()
    }

    /// Close all PiP windows
    pub fn close_all(&mut self) -> Vec<PipWindow> {
        let windows: Vec<_> = self.windows.drain().map(|(_, w)| w).collect();
        windows
    }
}

impl Default for PipManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_source() -> PipVideoSource {
        PipVideoSource::new(TabId::new(), "test-video-1".to_string())
    }

    fn create_test_config() -> PipConfig {
        PipConfig::default()
    }

    // PipWindowId Tests
    #[test]
    fn test_pip_window_id_uniqueness() {
        let id1 = PipWindowId::new();
        let id2 = PipWindowId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_pip_window_id_display() {
        let id = PipWindowId::new();
        let display = format!("{}", id);
        assert!(display.starts_with("pip-"));
    }

    // PipState Tests
    #[test]
    fn test_pip_state_default() {
        let state = PipState::default();
        assert_eq!(state, PipState::Active);
    }

    // PipVideoSource Tests
    #[test]
    fn test_pip_video_source_new() {
        let tab_id = TabId::new();
        let source = PipVideoSource::new(tab_id, "video-1".to_string());
        assert_eq!(source.tab_id, tab_id);
        assert_eq!(source.video_element_id, "video-1");
        assert!(source.video_url.is_none());
    }

    #[test]
    fn test_pip_video_source_with_url() {
        let tab_id = TabId::new();
        let source =
            PipVideoSource::with_url(tab_id, "video-1".to_string(), "https://example.com".into());
        assert_eq!(source.video_url, Some("https://example.com".to_string()));
    }

    // PipSize Tests
    #[test]
    fn test_pip_size_aspect_ratio() {
        let size = PipSize::new(1920, 1080);
        let ratio = size.aspect_ratio();
        assert!((ratio - 16.0 / 9.0).abs() < 0.01);
    }

    #[test]
    fn test_pip_size_aspect_ratio_zero_height() {
        let size = PipSize::new(100, 0);
        assert_eq!(size.aspect_ratio(), 0.0);
    }

    // PipBounds Tests
    #[test]
    fn test_pip_bounds_resize_preserve_aspect_ratio() {
        let mut bounds = PipBounds::new(PipPosition::default(), PipSize::new(320, 180));
        let new_size = bounds.resize(640, 0); // Height ignored when preserving ratio
        assert_eq!(new_size.width, 640);
        // Should preserve 16:9 ratio
        assert_eq!(new_size.height, 360);
    }

    #[test]
    fn test_pip_bounds_resize_no_preserve() {
        let mut bounds =
            PipBounds::new(PipPosition::default(), PipSize::new(320, 180)).preserve_ratio(false);
        let new_size = bounds.resize(400, 300);
        assert_eq!(new_size.width, 400);
        assert_eq!(new_size.height, 300);
    }

    #[test]
    fn test_pip_bounds_resize_clamps_to_limits() {
        let mut bounds = PipBounds::new(PipPosition::default(), PipSize::new(320, 180));
        let new_size = bounds.resize(50, 50); // Below minimum
        assert_eq!(new_size.width, MIN_PIP_WIDTH);
    }

    #[test]
    fn test_pip_bounds_move_to() {
        let mut bounds = PipBounds::default();
        bounds.move_to(500, 300);
        assert_eq!(bounds.position.x, 500);
        assert_eq!(bounds.position.y, 300);
    }

    #[test]
    fn test_pip_bounds_right_bottom() {
        let bounds = PipBounds::new(PipPosition::new(100, 50), PipSize::new(320, 180));
        assert_eq!(bounds.right(), 420);
        assert_eq!(bounds.bottom(), 230);
    }

    // PipControls Tests
    #[test]
    fn test_pip_controls_toggle_play_pause() {
        let mut controls = PipControls::new();
        assert!(controls.is_playing);
        let result = controls.toggle_play_pause();
        assert!(!result);
        assert!(!controls.is_playing);
    }

    #[test]
    fn test_pip_controls_toggle_mute() {
        let mut controls = PipControls::new();
        assert!(!controls.is_muted);
        let result = controls.toggle_mute();
        assert!(result);
        assert!(controls.is_muted);
    }

    #[test]
    fn test_pip_controls_set_volume_clamps() {
        let mut controls = PipControls::new();
        controls.set_volume(1.5);
        assert_eq!(controls.volume, 1.0);
        controls.set_volume(-0.5);
        assert_eq!(controls.volume, 0.0);
    }

    // PipConfig Tests
    #[test]
    fn test_pip_config_builder() {
        let config = PipConfig::new()
            .with_size(400, 225)
            .with_position(100, 100)
            .with_anchor(PipCorner::TopLeft)
            .with_always_on_top(false)
            .with_opacity(0.8);

        assert_eq!(config.initial_size.width, 400);
        assert_eq!(config.initial_size.height, 225);
        assert_eq!(config.initial_position, Some(PipPosition::new(100, 100)));
        assert_eq!(config.anchor_corner, PipCorner::TopLeft);
        assert!(!config.always_on_top);
        assert!((config.opacity - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_pip_config_opacity_clamps() {
        let config = PipConfig::new().with_opacity(1.5);
        assert_eq!(config.opacity, 1.0);
    }

    // PipWindow Tests
    #[test]
    fn test_pip_window_new() {
        let source = create_test_source();
        let config = create_test_config();
        let window = PipWindow::new(source.clone(), config);

        assert_eq!(window.source, source);
        assert_eq!(window.state, PipState::Active);
        assert!(window.controls.is_playing);
    }

    #[test]
    fn test_pip_window_new_no_autoplay() {
        let source = create_test_source();
        let mut config = create_test_config();
        config.auto_play = false;
        let window = PipWindow::new(source, config);

        assert_eq!(window.state, PipState::Paused);
        assert!(!window.controls.is_playing);
    }

    #[test]
    fn test_pip_window_toggle_play_pause() {
        let source = create_test_source();
        let config = create_test_config();
        let mut window = PipWindow::new(source, config);

        assert_eq!(window.state, PipState::Active);
        let new_state = window.toggle_play_pause();
        assert_eq!(new_state, PipState::Paused);
        let new_state = window.toggle_play_pause();
        assert_eq!(new_state, PipState::Active);
    }

    #[test]
    fn test_pip_window_minimize_restore() {
        let source = create_test_source();
        let config = create_test_config();
        let mut window = PipWindow::new(source, config);

        window.minimize();
        assert!(window.is_minimized());
        assert_eq!(window.state, PipState::Minimized);

        window.restore();
        assert!(!window.is_minimized());
        assert_eq!(window.state, PipState::Active);
    }

    #[test]
    fn test_pip_window_handle_action() {
        let source = create_test_source();
        let config = create_test_config();
        let mut window = PipWindow::new(source, config);

        window.handle_action(PipControlAction::Mute);
        assert!(window.controls.is_muted);

        window.handle_action(PipControlAction::ToggleMute);
        assert!(!window.controls.is_muted);

        window.handle_action(PipControlAction::Pause);
        assert_eq!(window.state, PipState::Paused);
    }

    // StubVideoExtractor Tests
    #[test]
    fn test_stub_extractor_can_extract() {
        let extractor = StubVideoExtractor::new();
        let source = create_test_source();
        assert!(extractor.can_extract(&source).unwrap());
    }

    #[test]
    fn test_stub_extractor_get_video_info() {
        let extractor = StubVideoExtractor::new();
        let source = create_test_source();
        let info = extractor.get_video_info(&source).unwrap();
        assert_eq!(info.width, 1920);
        assert_eq!(info.height, 1080);
    }

    #[test]
    fn test_stub_extractor_start_stop() {
        let mut extractor = StubVideoExtractor::new();
        let source = create_test_source();

        assert!(!extractor.is_extracting(&source));
        extractor.start_extraction(&source).unwrap();
        assert!(extractor.is_extracting(&source));
        extractor.stop_extraction(&source).unwrap();
        assert!(!extractor.is_extracting(&source));
    }

    // PipManager Tests
    #[test]
    fn test_pip_manager_new() {
        let manager = PipManager::new();
        assert_eq!(manager.window_count(), 0);
        assert!(manager.can_create_pip());
    }

    #[test]
    fn test_pip_manager_create_close() {
        let mut manager = PipManager::new();
        let source = create_test_source();
        let config = create_test_config();

        let pip_id = manager.create_pip_window(source, config).unwrap();
        assert_eq!(manager.window_count(), 1);

        let closed = manager.close_pip_window(pip_id).unwrap();
        assert_eq!(closed.id, pip_id);
        assert_eq!(manager.window_count(), 0);
    }

    #[test]
    fn test_pip_manager_max_windows() {
        let mut manager = PipManager::new();
        manager.set_max_windows(2);

        for _ in 0..2 {
            let source = create_test_source();
            let config = create_test_config();
            manager.create_pip_window(source, config).unwrap();
        }

        let source = create_test_source();
        let config = create_test_config();
        let result = manager.create_pip_window(source, config);

        assert!(matches!(result, Err(PipError::MaxWindowsReached(2))));
    }

    #[test]
    fn test_pip_manager_get_window() {
        let mut manager = PipManager::new();
        let source = create_test_source();
        let config = create_test_config();

        let pip_id = manager.create_pip_window(source.clone(), config).unwrap();

        let window = manager.get_pip_window(pip_id).unwrap();
        assert_eq!(window.source, source);

        let window_mut = manager.get_pip_window_mut(pip_id).unwrap();
        window_mut.minimize();
        assert!(manager.get_pip_window(pip_id).unwrap().is_minimized());
    }

    #[test]
    fn test_pip_manager_not_found_error() {
        let mut manager = PipManager::new();
        let fake_id = PipWindowId::new();

        assert!(matches!(
            manager.close_pip_window(fake_id),
            Err(PipError::NotFound(_))
        ));
        assert!(matches!(
            manager.toggle_play_pause(fake_id),
            Err(PipError::NotFound(_))
        ));
    }

    #[test]
    fn test_pip_manager_close_all_for_tab() {
        let mut manager = PipManager::new();
        let tab_id = TabId::new();
        let other_tab_id = TabId::new();

        // Create 2 PiP windows for tab_id
        for _ in 0..2 {
            let source = PipVideoSource::new(tab_id, "video".to_string());
            manager
                .create_pip_window(source, PipConfig::default())
                .unwrap();
        }

        // Create 1 PiP window for other_tab_id
        let source = PipVideoSource::new(other_tab_id, "video".to_string());
        manager
            .create_pip_window(source, PipConfig::default())
            .unwrap();

        assert_eq!(manager.window_count(), 3);

        let closed = manager.close_all_for_tab(tab_id);
        assert_eq!(closed.len(), 2);
        assert_eq!(manager.window_count(), 1);
    }

    #[test]
    fn test_pip_manager_close_all() {
        let mut manager = PipManager::new();

        for _ in 0..3 {
            let source = create_test_source();
            manager
                .create_pip_window(source, PipConfig::default())
                .unwrap();
        }

        let closed = manager.close_all();
        assert_eq!(closed.len(), 3);
        assert_eq!(manager.window_count(), 0);
    }

    #[test]
    fn test_pip_manager_handle_close_action() {
        let mut manager = PipManager::new();
        let source = create_test_source();
        let pip_id = manager
            .create_pip_window(source, PipConfig::default())
            .unwrap();

        manager
            .handle_action(pip_id, PipControlAction::Close)
            .unwrap();
        assert_eq!(manager.window_count(), 0);
    }

    #[test]
    fn test_pip_manager_resize_window() {
        let mut manager = PipManager::new();
        let source = create_test_source();
        let pip_id = manager
            .create_pip_window(source, PipConfig::default())
            .unwrap();

        let new_size = manager.resize_pip_window(pip_id, 500, 300).unwrap();
        // Will preserve aspect ratio from default 16:9
        assert_eq!(new_size.width, 500);
    }

    #[test]
    fn test_pip_manager_move_window() {
        let mut manager = PipManager::new();
        let source = create_test_source();
        let pip_id = manager
            .create_pip_window(source, PipConfig::default())
            .unwrap();

        manager.move_pip_window(pip_id, 200, 150).unwrap();

        let window = manager.get_pip_window(pip_id).unwrap();
        assert_eq!(window.bounds.position.x, 200);
        assert_eq!(window.bounds.position.y, 150);
    }

    #[test]
    fn test_pip_manager_get_all_ids() {
        let mut manager = PipManager::new();
        let mut created_ids = Vec::new();

        for _ in 0..3 {
            let source = create_test_source();
            let id = manager
                .create_pip_window(source, PipConfig::default())
                .unwrap();
            created_ids.push(id);
        }

        let all_ids = manager.get_all_pip_ids();
        assert_eq!(all_ids.len(), 3);
        for id in created_ids {
            assert!(all_ids.contains(&id));
        }
    }

    #[test]
    fn test_pip_manager_get_windows_for_tab() {
        let mut manager = PipManager::new();
        let tab_id = TabId::new();

        // Create 2 windows for the same tab
        for _ in 0..2 {
            let source = PipVideoSource::new(tab_id, "video".to_string());
            manager
                .create_pip_window(source, PipConfig::default())
                .unwrap();
        }

        // Create 1 window for different tab
        let other_source = PipVideoSource::new(TabId::new(), "video".to_string());
        manager
            .create_pip_window(other_source, PipConfig::default())
            .unwrap();

        let tab_windows = manager.get_pip_windows_for_tab(tab_id);
        assert_eq!(tab_windows.len(), 2);
        for window in tab_windows {
            assert_eq!(window.source.tab_id, tab_id);
        }
    }
}
