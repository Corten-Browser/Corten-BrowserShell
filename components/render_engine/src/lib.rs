//! Render Engine Integration Layer
//!
//! Provides render engine abstraction for web content display in the CortenBrowser Browser Shell.
//!
//! # Features
//!
//! - **RenderEngine trait**: Abstraction for render engine implementations
//! - **Viewport management**: Dimensions, scroll position, zoom level, and device pixel ratio
//! - **Frame scheduling**: Support for 60fps rendering with animation frame callbacks
//! - **Dirty region tracking**: Efficient partial repaints through invalidation regions
//! - **Compositor integration**: Layer-based compositing for smooth scrolling and animations
//! - **DOM integration stubs**: Placeholder types for future HTML/CSS/DOM engine integration
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      RenderEngine                            │
//! │  ┌─────────────────┐    ┌────────────────┐                  │
//! │  │ Viewport        │───►│ Frame          │                  │
//! │  │ Management      │    │ Generation     │                  │
//! │  └─────────────────┘    └───────┬────────┘                  │
//! │                                 │                            │
//! │  ┌─────────────────┐    ┌───────▼────────┐                  │
//! │  │ Dirty Region    │◄───│ Compositor     │                  │
//! │  │ Tracking        │    │ Integration    │                  │
//! │  └─────────────────┘    └────────────────┘                  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Quick Start
//!
//! ## Basic Rendering
//!
//! ```rust,ignore
//! use render_engine::{RenderEngine, Viewport, MockRenderEngine};
//!
//! let mut engine = MockRenderEngine::new(800, 600);
//!
//! // Set up viewport
//! let viewport = Viewport::new(800, 600);
//!
//! // Render a frame
//! let frame = engine.render_frame(&viewport).unwrap();
//! println!("Frame size: {}x{}", frame.width, frame.height);
//! ```
//!
//! ## Animation Frame Callbacks
//!
//! ```rust,ignore
//! use render_engine::{RenderEngine, MockRenderEngine};
//!
//! let mut engine = MockRenderEngine::new(800, 600);
//!
//! // Request animation frame (for 60fps rendering)
//! engine.request_animation_frame(Box::new(|timestamp| {
//!     println!("Animation frame at: {}ms", timestamp);
//! }));
//!
//! // Process pending callbacks
//! engine.process_animation_frames();
//! ```
//!
//! ## Dirty Region Tracking
//!
//! ```rust,ignore
//! use render_engine::{RenderEngine, Rect, MockRenderEngine};
//!
//! let mut engine = MockRenderEngine::new(800, 600);
//!
//! // Invalidate specific region
//! engine.invalidate(Some(Rect { x: 100, y: 100, width: 200, height: 200 }));
//!
//! // Or invalidate entire viewport
//! engine.invalidate(None);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Render engine errors
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Invalid viewport dimensions: {0}x{1}")]
    InvalidViewport(u32, u32),
    #[error("Render failed: {0}")]
    RenderFailed(String),
    #[error("Engine not initialized")]
    NotInitialized,
    #[error("Invalid zoom level: {0}")]
    InvalidZoom(f32),
    #[error("Frame buffer allocation failed")]
    AllocationFailed,
    #[error("Compositor error: {0}")]
    CompositorError(String),
}

/// Result type for render operations
pub type RenderResult<T> = Result<T, RenderError>;

/// Rectangle structure for regions and bounds
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    /// X coordinate of top-left corner
    pub x: i32,
    /// Y coordinate of top-left corner
    pub y: i32,
    /// Width of the rectangle
    pub width: u32,
    /// Height of the rectangle
    pub height: u32,
}

impl Rect {
    /// Create a new rectangle
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// Create a rectangle at origin with given dimensions
    pub fn from_size(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    /// Check if this rectangle contains a point
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && x < self.x + self.width as i32
            && y >= self.y
            && y < self.y + self.height as i32
    }

    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width as i32
            && self.x + self.width as i32 > other.x
            && self.y < other.y + other.height as i32
            && self.y + self.height as i32 > other.y
    }

    /// Calculate the intersection of two rectangles
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }

        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.width as i32).min(other.x + other.width as i32);
        let bottom = (self.y + self.height as i32).min(other.y + other.height as i32);

        Some(Rect {
            x,
            y,
            width: (right - x) as u32,
            height: (bottom - y) as u32,
        })
    }

    /// Calculate the union (bounding box) of two rectangles
    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = (self.x + self.width as i32).max(other.x + other.width as i32);
        let bottom = (self.y + self.height as i32).max(other.y + other.height as i32);

        Rect {
            x,
            y,
            width: (right - x) as u32,
            height: (bottom - y) as u32,
        }
    }

    /// Get the area of this rectangle
    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Check if the rectangle is empty (zero area)
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::from_size(0, 0)
    }
}

/// Viewport configuration for rendering
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Horizontal scroll offset
    pub scroll_x: i32,
    /// Vertical scroll offset
    pub scroll_y: i32,
    /// Zoom level (1.0 = 100%)
    pub zoom: f32,
    /// Device pixel ratio for HiDPI displays
    pub device_pixel_ratio: f32,
}

impl Viewport {
    /// Create a new viewport with default scroll and zoom
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            scroll_x: 0,
            scroll_y: 0,
            zoom: 1.0,
            device_pixel_ratio: 1.0,
        }
    }

    /// Create a viewport with specific device pixel ratio
    pub fn with_dpr(width: u32, height: u32, device_pixel_ratio: f32) -> Self {
        Self {
            width,
            height,
            scroll_x: 0,
            scroll_y: 0,
            zoom: 1.0,
            device_pixel_ratio,
        }
    }

    /// Set scroll position
    pub fn set_scroll(&mut self, x: i32, y: i32) {
        self.scroll_x = x;
        self.scroll_y = y;
    }

    /// Set zoom level
    pub fn set_zoom(&mut self, zoom: f32) -> RenderResult<()> {
        if zoom < 0.1 || zoom > 10.0 {
            return Err(RenderError::InvalidZoom(zoom));
        }
        self.zoom = zoom;
        Ok(())
    }

    /// Get the physical (device) pixel dimensions
    pub fn physical_size(&self) -> (u32, u32) {
        (
            (self.width as f32 * self.device_pixel_ratio) as u32,
            (self.height as f32 * self.device_pixel_ratio) as u32,
        )
    }

    /// Get the visible content bounds in document coordinates
    pub fn visible_bounds(&self) -> Rect {
        Rect {
            x: self.scroll_x,
            y: self.scroll_y,
            width: (self.width as f32 / self.zoom) as u32,
            height: (self.height as f32 / self.zoom) as u32,
        }
    }

    /// Convert viewport coordinates to document coordinates
    pub fn viewport_to_document(&self, x: i32, y: i32) -> (i32, i32) {
        let doc_x = (x as f32 / self.zoom) as i32 + self.scroll_x;
        let doc_y = (y as f32 / self.zoom) as i32 + self.scroll_y;
        (doc_x, doc_y)
    }

    /// Convert document coordinates to viewport coordinates
    pub fn document_to_viewport(&self, x: i32, y: i32) -> (i32, i32) {
        let vp_x = ((x - self.scroll_x) as f32 * self.zoom) as i32;
        let vp_y = ((y - self.scroll_y) as f32 * self.zoom) as i32;
        (vp_x, vp_y)
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(800, 600)
    }
}

/// Pixel format for frame buffers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelFormat {
    /// 32-bit RGBA (8 bits per channel)
    Rgba8,
    /// 32-bit BGRA (8 bits per channel)
    Bgra8,
    /// 24-bit RGB (8 bits per channel)
    Rgb8,
    /// 16-bit RGB (5-6-5)
    Rgb565,
}

impl PixelFormat {
    /// Get bytes per pixel for this format
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::Rgba8 | PixelFormat::Bgra8 => 4,
            PixelFormat::Rgb8 => 3,
            PixelFormat::Rgb565 => 2,
        }
    }
}

impl Default for PixelFormat {
    fn default() -> Self {
        PixelFormat::Rgba8
    }
}

/// Rendered frame containing pixel data
#[derive(Debug, Clone)]
pub struct Frame {
    /// Frame width in pixels
    pub width: u32,
    /// Frame height in pixels
    pub height: u32,
    /// Pixel format
    pub format: PixelFormat,
    /// Raw pixel data
    pub data: Vec<u8>,
    /// Timestamp when frame was rendered
    pub timestamp: Duration,
    /// Dirty regions that were rendered in this frame
    pub dirty_regions: Vec<Rect>,
    /// Frame sequence number
    pub sequence: u64,
}

impl Frame {
    /// Create a new frame with allocated buffer
    pub fn new(width: u32, height: u32, format: PixelFormat) -> RenderResult<Self> {
        if width == 0 || height == 0 {
            return Err(RenderError::InvalidViewport(width, height));
        }

        let size = width as usize * height as usize * format.bytes_per_pixel();
        let data = vec![0u8; size];

        Ok(Self {
            width,
            height,
            format,
            data,
            timestamp: Duration::ZERO,
            dirty_regions: Vec::new(),
            sequence: 0,
        })
    }

    /// Create a frame from existing pixel data
    pub fn from_data(
        width: u32,
        height: u32,
        format: PixelFormat,
        data: Vec<u8>,
    ) -> RenderResult<Self> {
        let expected_size = width as usize * height as usize * format.bytes_per_pixel();
        if data.len() != expected_size {
            return Err(RenderError::RenderFailed(format!(
                "Data size mismatch: expected {}, got {}",
                expected_size,
                data.len()
            )));
        }

        Ok(Self {
            width,
            height,
            format,
            data,
            timestamp: Duration::ZERO,
            dirty_regions: Vec::new(),
            sequence: 0,
        })
    }

    /// Get pixel at coordinates (returns None if out of bounds)
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<&[u8]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let bpp = self.format.bytes_per_pixel();
        let offset = (y as usize * self.width as usize + x as usize) * bpp;
        Some(&self.data[offset..offset + bpp])
    }

    /// Set pixel at coordinates
    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: &[u8]) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let bpp = self.format.bytes_per_pixel();
        if pixel.len() != bpp {
            return false;
        }
        let offset = (y as usize * self.width as usize + x as usize) * bpp;
        self.data[offset..offset + bpp].copy_from_slice(pixel);
        true
    }

    /// Get the stride (bytes per row)
    pub fn stride(&self) -> usize {
        self.width as usize * self.format.bytes_per_pixel()
    }

    /// Get total size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if frame is empty
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// Type alias for animation frame callback
pub type FrameCallback = Box<dyn FnOnce(f64) + Send>;

/// Compositor layer for compositing operations
#[derive(Debug, Clone)]
pub struct CompositorLayer {
    /// Layer ID
    pub id: u64,
    /// Layer bounds in parent coordinates
    pub bounds: Rect,
    /// Layer opacity (0.0 - 1.0)
    pub opacity: f32,
    /// Whether the layer is visible
    pub visible: bool,
    /// Z-index for stacking order
    pub z_index: i32,
    /// Transform matrix (simplified as scale and translate)
    pub transform: LayerTransform,
}

/// Layer transform (simplified 2D transform)
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct LayerTransform {
    /// X translation
    pub translate_x: f32,
    /// Y translation
    pub translate_y: f32,
    /// X scale
    pub scale_x: f32,
    /// Y scale
    pub scale_y: f32,
    /// Rotation in radians
    pub rotation: f32,
}

impl LayerTransform {
    /// Create identity transform
    pub fn identity() -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        }
    }

    /// Create translation transform
    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            translate_x: x,
            translate_y: y,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        }
    }

    /// Create scale transform
    pub fn scale(x: f32, y: f32) -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            scale_x: x,
            scale_y: y,
            rotation: 0.0,
        }
    }

    /// Check if this is an identity transform
    pub fn is_identity(&self) -> bool {
        (self.translate_x - 0.0).abs() < f32::EPSILON
            && (self.translate_y - 0.0).abs() < f32::EPSILON
            && (self.scale_x - 1.0).abs() < f32::EPSILON
            && (self.scale_y - 1.0).abs() < f32::EPSILON
            && (self.rotation - 0.0).abs() < f32::EPSILON
    }
}

impl CompositorLayer {
    /// Create a new layer
    pub fn new(id: u64, bounds: Rect) -> Self {
        Self {
            id,
            bounds,
            opacity: 1.0,
            visible: true,
            z_index: 0,
            transform: LayerTransform::identity(),
        }
    }

    /// Set layer opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set layer z-index
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// Set layer transform
    pub fn with_transform(mut self, transform: LayerTransform) -> Self {
        self.transform = transform;
        self
    }
}

/// Render engine trait - abstraction for render engine implementations
///
/// This trait defines the interface that all render engine implementations
/// must provide for rendering web content.
pub trait RenderEngine: Send + Sync {
    /// Render a frame for the given viewport
    ///
    /// # Arguments
    /// * `viewport` - The viewport configuration for rendering
    ///
    /// # Returns
    /// A rendered frame containing pixel data
    fn render_frame(&mut self, viewport: &Viewport) -> RenderResult<Frame>;

    /// Invalidate a region, marking it for repaint
    ///
    /// # Arguments
    /// * `region` - Optional region to invalidate. If None, invalidate entire viewport
    fn invalidate(&mut self, region: Option<Rect>);

    /// Resize the render surface
    ///
    /// # Arguments
    /// * `width` - New width in pixels
    /// * `height` - New height in pixels
    fn resize(&mut self, width: u32, height: u32);

    /// Set the zoom level
    ///
    /// # Arguments
    /// * `scale` - Zoom scale (1.0 = 100%)
    fn set_zoom(&mut self, scale: f32);

    /// Request an animation frame callback
    ///
    /// # Arguments
    /// * `callback` - Callback to invoke on next frame (receives timestamp in ms)
    fn request_animation_frame(&mut self, callback: FrameCallback);

    /// Get the current frame rate (frames per second)
    fn get_frame_rate(&self) -> f32;

    /// Get pending dirty regions
    fn get_dirty_regions(&self) -> Vec<Rect>;

    /// Clear all dirty regions
    fn clear_dirty_regions(&mut self);

    /// Check if a repaint is needed
    fn needs_repaint(&self) -> bool;
}

/// Mock render engine for testing and placeholder implementation
pub struct MockRenderEngine {
    width: u32,
    height: u32,
    zoom: f32,
    dirty_regions: Mutex<Vec<Rect>>,
    animation_callbacks: Mutex<VecDeque<FrameCallback>>,
    frame_sequence: AtomicU64,
    start_time: Instant,
    last_frame_time: Mutex<Instant>,
    frame_count: Mutex<u64>,
    layers: Mutex<Vec<CompositorLayer>>,
    next_layer_id: Mutex<u64>,
}

impl MockRenderEngine {
    /// Create a new mock render engine
    pub fn new(width: u32, height: u32) -> Self {
        let now = Instant::now();
        Self {
            width,
            height,
            zoom: 1.0,
            dirty_regions: Mutex::new(Vec::new()),
            animation_callbacks: Mutex::new(VecDeque::new()),
            frame_sequence: AtomicU64::new(0),
            start_time: now,
            last_frame_time: Mutex::new(now),
            frame_count: Mutex::new(0),
            layers: Mutex::new(Vec::new()),
            next_layer_id: Mutex::new(1),
        }
    }

    /// Process pending animation frame callbacks
    pub fn process_animation_frames(&self) {
        let timestamp = self.start_time.elapsed().as_secs_f64() * 1000.0;

        let mut callbacks = self.animation_callbacks.lock().unwrap();
        while let Some(callback) = callbacks.pop_front() {
            drop(callbacks); // Release lock before calling callback
            callback(timestamp);
            callbacks = self.animation_callbacks.lock().unwrap();
        }
    }

    /// Get number of pending animation callbacks
    pub fn pending_callbacks(&self) -> usize {
        self.animation_callbacks.lock().unwrap().len()
    }

    /// Add a compositor layer
    pub fn add_layer(&self, bounds: Rect) -> u64 {
        let mut next_id = self.next_layer_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        drop(next_id);

        let mut layers = self.layers.lock().unwrap();
        layers.push(CompositorLayer::new(id, bounds));
        id
    }

    /// Remove a compositor layer
    pub fn remove_layer(&self, id: u64) -> bool {
        let mut layers = self.layers.lock().unwrap();
        if let Some(pos) = layers.iter().position(|l| l.id == id) {
            layers.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get a layer by ID (returns a clone)
    pub fn get_layer(&self, id: u64) -> Option<CompositorLayer> {
        let layers = self.layers.lock().unwrap();
        layers.iter().find(|l| l.id == id).cloned()
    }

    /// Update a layer by ID
    pub fn update_layer<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut CompositorLayer),
    {
        let mut layers = self.layers.lock().unwrap();
        if let Some(layer) = layers.iter_mut().find(|l| l.id == id) {
            f(layer);
            true
        } else {
            false
        }
    }

    /// Get all layers sorted by z-index (returns clones)
    pub fn get_layers_sorted(&self) -> Vec<CompositorLayer> {
        let layers = self.layers.lock().unwrap();
        let mut sorted: Vec<_> = layers.iter().cloned().collect();
        sorted.sort_by_key(|l| l.z_index);
        sorted
    }

    /// Get current dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get current zoom level
    pub fn current_zoom(&self) -> f32 {
        self.zoom
    }
}

impl RenderEngine for MockRenderEngine {
    fn render_frame(&mut self, viewport: &Viewport) -> RenderResult<Frame> {
        if viewport.width == 0 || viewport.height == 0 {
            return Err(RenderError::InvalidViewport(viewport.width, viewport.height));
        }

        let sequence = self.frame_sequence.fetch_add(1, Ordering::SeqCst);
        let timestamp = self.start_time.elapsed();

        // Create a mock frame with a simple pattern
        let mut frame = Frame::new(viewport.width, viewport.height, PixelFormat::Rgba8)?;
        frame.timestamp = timestamp;
        frame.sequence = sequence;
        frame.dirty_regions = self.dirty_regions.lock().unwrap().clone();

        // Fill with a simple gradient pattern (for testing)
        for y in 0..viewport.height {
            for x in 0..viewport.width {
                let r = (x as f32 / viewport.width as f32 * 255.0) as u8;
                let g = (y as f32 / viewport.height as f32 * 255.0) as u8;
                let b = 128u8;
                let a = 255u8;
                frame.set_pixel(x, y, &[r, g, b, a]);
            }
        }

        // Clear dirty regions after rendering
        self.dirty_regions.lock().unwrap().clear();
        *self.frame_count.lock().unwrap() += 1;
        *self.last_frame_time.lock().unwrap() = Instant::now();

        Ok(frame)
    }

    fn invalidate(&mut self, region: Option<Rect>) {
        let mut dirty_regions = self.dirty_regions.lock().unwrap();
        match region {
            Some(rect) => {
                // Merge with existing dirty regions if overlapping
                let mut merged = false;
                for existing in dirty_regions.iter_mut() {
                    if existing.intersects(&rect) {
                        *existing = existing.union(&rect);
                        merged = true;
                        break;
                    }
                }
                if !merged {
                    dirty_regions.push(rect);
                }
            }
            None => {
                // Invalidate entire viewport
                dirty_regions.clear();
                dirty_regions.push(Rect::from_size(self.width, self.height));
            }
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        // Invalidate entire new area
        let mut dirty_regions = self.dirty_regions.lock().unwrap();
        dirty_regions.clear();
        dirty_regions.push(Rect::from_size(width, height));
    }

    fn set_zoom(&mut self, scale: f32) {
        self.zoom = scale.clamp(0.1, 10.0);
        // Invalidate when zoom changes
        let mut dirty_regions = self.dirty_regions.lock().unwrap();
        dirty_regions.clear();
        dirty_regions.push(Rect::from_size(self.width, self.height));
    }

    fn request_animation_frame(&mut self, callback: FrameCallback) {
        self.animation_callbacks.lock().unwrap().push_back(callback);
    }

    fn get_frame_rate(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        if elapsed > 0.0 {
            *self.frame_count.lock().unwrap() as f32 / elapsed
        } else {
            0.0
        }
    }

    fn get_dirty_regions(&self) -> Vec<Rect> {
        self.dirty_regions.lock().unwrap().clone()
    }

    fn clear_dirty_regions(&mut self) {
        self.dirty_regions.lock().unwrap().clear();
    }

    fn needs_repaint(&self) -> bool {
        !self.dirty_regions.lock().unwrap().is_empty()
            || !self.animation_callbacks.lock().unwrap().is_empty()
    }
}

impl Default for MockRenderEngine {
    fn default() -> Self {
        Self::new(800, 600)
    }
}

/// Frame scheduler for managing render timing
///
/// Helps achieve smooth 60fps rendering by tracking frame timing
/// and providing vsync-like scheduling hints.
#[derive(Debug)]
pub struct FrameScheduler {
    /// Target frame rate (default: 60fps)
    target_fps: f32,
    /// Target frame duration
    target_frame_duration: Duration,
    /// Last frame timestamp
    last_frame: Instant,
    /// Frame time history for averaging
    frame_times: VecDeque<Duration>,
    /// Maximum history size
    max_history: usize,
}

impl FrameScheduler {
    /// Create a new frame scheduler targeting 60fps
    pub fn new() -> Self {
        Self::with_target_fps(60.0)
    }

    /// Create a frame scheduler with custom target FPS
    pub fn with_target_fps(fps: f32) -> Self {
        Self {
            target_fps: fps,
            target_frame_duration: Duration::from_secs_f32(1.0 / fps),
            last_frame: Instant::now(),
            frame_times: VecDeque::with_capacity(60),
            max_history: 60,
        }
    }

    /// Begin a new frame, returns time since last frame
    pub fn begin_frame(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now - self.last_frame;
        self.last_frame = now;

        // Track frame time
        self.frame_times.push_back(delta);
        if self.frame_times.len() > self.max_history {
            self.frame_times.pop_front();
        }

        delta
    }

    /// Get time remaining until next frame should start
    pub fn time_until_next_frame(&self) -> Duration {
        let elapsed = self.last_frame.elapsed();
        if elapsed >= self.target_frame_duration {
            Duration::ZERO
        } else {
            self.target_frame_duration - elapsed
        }
    }

    /// Check if it's time for the next frame
    pub fn should_render(&self) -> bool {
        self.last_frame.elapsed() >= self.target_frame_duration
    }

    /// Get average frame time over recent history
    pub fn average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return self.target_frame_duration;
        }
        let total: Duration = self.frame_times.iter().sum();
        total / self.frame_times.len() as u32
    }

    /// Get current FPS based on recent history
    pub fn current_fps(&self) -> f32 {
        let avg = self.average_frame_time();
        if avg.as_secs_f32() > 0.0 {
            1.0 / avg.as_secs_f32()
        } else {
            self.target_fps
        }
    }

    /// Get target FPS
    pub fn target_fps(&self) -> f32 {
        self.target_fps
    }

    /// Set target FPS
    pub fn set_target_fps(&mut self, fps: f32) {
        self.target_fps = fps;
        self.target_frame_duration = Duration::from_secs_f32(1.0 / fps);
    }
}

impl Default for FrameScheduler {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== DOM Integration Stubs (FEAT-013) ====================

/// DOM node types for future HTML/CSS/DOM engine integration
///
/// These stubs provide placeholder types that will be implemented
/// when integrating with an actual DOM engine (e.g., Servo, WebKit).
pub mod dom {
    use serde::{Deserialize, Serialize};

    /// Unique identifier for a DOM node
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct NodeId(pub u64);

    impl NodeId {
        /// Create a new node ID
        pub fn new(id: u64) -> Self {
            Self(id)
        }
    }

    /// Basic DOM node types
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum NodeType {
        /// Element node (e.g., <div>, <span>)
        Element,
        /// Text node
        Text,
        /// Comment node
        Comment,
        /// Document node
        Document,
        /// Document type declaration
        DocumentType,
        /// Document fragment
        DocumentFragment,
    }

    /// DOM element data (stub)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct ElementData {
        /// Tag name (e.g., "div", "span")
        pub tag_name: String,
        /// Element namespace
        pub namespace: Option<String>,
        /// Element attributes
        pub attributes: Vec<(String, String)>,
    }

    impl ElementData {
        /// Create a new element with the given tag name
        pub fn new(tag_name: impl Into<String>) -> Self {
            Self {
                tag_name: tag_name.into(),
                namespace: None,
                attributes: Vec::new(),
            }
        }

        /// Add an attribute to the element
        pub fn with_attribute(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
            self.attributes.push((name.into(), value.into()));
            self
        }

        /// Get an attribute value by name
        pub fn get_attribute(&self, name: &str) -> Option<&str> {
            self.attributes
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.as_str())
        }
    }

    /// DOM node representation (stub)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct DomNode {
        /// Unique node identifier
        pub id: NodeId,
        /// Node type
        pub node_type: NodeType,
        /// Element data (if this is an element node)
        pub element_data: Option<ElementData>,
        /// Text content (if this is a text node)
        pub text_content: Option<String>,
        /// Child node IDs
        pub children: Vec<NodeId>,
        /// Parent node ID
        pub parent: Option<NodeId>,
    }

    impl DomNode {
        /// Create a new element node
        pub fn element(id: NodeId, data: ElementData) -> Self {
            Self {
                id,
                node_type: NodeType::Element,
                element_data: Some(data),
                text_content: None,
                children: Vec::new(),
                parent: None,
            }
        }

        /// Create a new text node
        pub fn text(id: NodeId, content: impl Into<String>) -> Self {
            Self {
                id,
                node_type: NodeType::Text,
                element_data: None,
                text_content: Some(content.into()),
                children: Vec::new(),
                parent: None,
            }
        }

        /// Check if this is an element node
        pub fn is_element(&self) -> bool {
            matches!(self.node_type, NodeType::Element)
        }

        /// Check if this is a text node
        pub fn is_text(&self) -> bool {
            matches!(self.node_type, NodeType::Text)
        }

        /// Get the tag name if this is an element
        pub fn tag_name(&self) -> Option<&str> {
            self.element_data.as_ref().map(|d| d.tag_name.as_str())
        }
    }

    /// CSS computed style values (stub)
    #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
    pub struct ComputedStyle {
        /// Display property
        pub display: String,
        /// Position property
        pub position: String,
        /// Width (in pixels, or "auto")
        pub width: String,
        /// Height (in pixels, or "auto")
        pub height: String,
        /// Background color
        pub background_color: String,
        /// Text color
        pub color: String,
        /// Font family
        pub font_family: String,
        /// Font size in pixels
        pub font_size_px: f32,
    }

    impl ComputedStyle {
        /// Create default computed style
        pub fn new() -> Self {
            Self {
                display: "block".to_string(),
                position: "static".to_string(),
                width: "auto".to_string(),
                height: "auto".to_string(),
                background_color: "transparent".to_string(),
                color: "black".to_string(),
                font_family: "sans-serif".to_string(),
                font_size_px: 16.0,
            }
        }
    }

    /// DOM integration trait for future HTML/CSS engine integration
    ///
    /// This trait defines the interface that a DOM engine implementation
    /// must provide to integrate with the render engine.
    ///
    /// # Implementation Notes
    ///
    /// This is a stub trait. Actual implementation will depend on the
    /// chosen DOM engine (Servo, WebKit, etc.).
    pub trait DomIntegration: Send + Sync {
        /// Get a node by its ID
        fn get_node(&self, id: NodeId) -> Option<DomNode>;

        /// Get the document root node
        fn get_document_root(&self) -> Option<NodeId>;

        /// Get computed style for a node
        fn get_computed_style(&self, id: NodeId) -> Option<ComputedStyle>;

        /// Query nodes by CSS selector (stub - returns empty vec)
        fn query_selector(&self, selector: &str) -> Vec<NodeId>;

        /// Query all matching nodes by CSS selector (stub - returns empty vec)
        fn query_selector_all(&self, selector: &str) -> Vec<NodeId>;

        /// Get element by ID
        fn get_element_by_id(&self, id: &str) -> Option<NodeId>;

        /// Get elements by class name
        fn get_elements_by_class_name(&self, class_name: &str) -> Vec<NodeId>;

        /// Get elements by tag name
        fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<NodeId>;
    }

    /// Stub DOM integration implementation for testing
    ///
    /// This implementation returns empty/default values for all operations.
    /// It serves as a placeholder until a real DOM engine is integrated.
    #[derive(Debug, Default)]
    pub struct StubDomIntegration {
        /// Mock document root node
        root: Option<DomNode>,
    }

    impl StubDomIntegration {
        /// Create a new stub DOM integration
        pub fn new() -> Self {
            Self { root: None }
        }

        /// Create a stub integration with a mock document
        pub fn with_mock_document() -> Self {
            let root = DomNode::element(
                NodeId::new(1),
                ElementData::new("html"),
            );
            Self { root: Some(root) }
        }
    }

    impl DomIntegration for StubDomIntegration {
        fn get_node(&self, id: NodeId) -> Option<DomNode> {
            if let Some(ref root) = self.root {
                if root.id == id {
                    return Some(root.clone());
                }
            }
            None
        }

        fn get_document_root(&self) -> Option<NodeId> {
            self.root.as_ref().map(|n| n.id)
        }

        fn get_computed_style(&self, _id: NodeId) -> Option<ComputedStyle> {
            // Return default computed style for any node
            Some(ComputedStyle::new())
        }

        fn query_selector(&self, _selector: &str) -> Vec<NodeId> {
            // Stub: CSS selector queries not implemented
            Vec::new()
        }

        fn query_selector_all(&self, _selector: &str) -> Vec<NodeId> {
            // Stub: CSS selector queries not implemented
            Vec::new()
        }

        fn get_element_by_id(&self, _id: &str) -> Option<NodeId> {
            // Stub: ID queries not implemented
            None
        }

        fn get_elements_by_class_name(&self, _class_name: &str) -> Vec<NodeId> {
            // Stub: class name queries not implemented
            Vec::new()
        }

        fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<NodeId> {
            // Return root if it matches the tag name
            if let Some(ref root) = self.root {
                if root.tag_name() == Some(tag_name) {
                    return vec![root.id];
                }
            }
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::DomIntegration;

    // ==================== Rect Tests ====================

    #[test]
    fn test_rect_new() {
        let rect = Rect::new(10, 20, 100, 200);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.width, 100);
        assert_eq!(rect.height, 200);
    }

    #[test]
    fn test_rect_from_size() {
        let rect = Rect::from_size(800, 600);
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
        assert_eq!(rect.width, 800);
        assert_eq!(rect.height, 600);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 10, 100, 100);

        // Points inside
        assert!(rect.contains(10, 10));
        assert!(rect.contains(50, 50));
        assert!(rect.contains(109, 109));

        // Points outside
        assert!(!rect.contains(9, 10));
        assert!(!rect.contains(10, 9));
        assert!(!rect.contains(110, 50));
        assert!(!rect.contains(50, 110));
    }

    #[test]
    fn test_rect_intersects() {
        let rect1 = Rect::new(0, 0, 100, 100);
        let rect2 = Rect::new(50, 50, 100, 100);
        let rect3 = Rect::new(200, 200, 100, 100);

        assert!(rect1.intersects(&rect2));
        assert!(rect2.intersects(&rect1));
        assert!(!rect1.intersects(&rect3));
        assert!(!rect3.intersects(&rect1));
    }

    #[test]
    fn test_rect_intersection() {
        let rect1 = Rect::new(0, 0, 100, 100);
        let rect2 = Rect::new(50, 50, 100, 100);

        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection.x, 50);
        assert_eq!(intersection.y, 50);
        assert_eq!(intersection.width, 50);
        assert_eq!(intersection.height, 50);

        let rect3 = Rect::new(200, 200, 100, 100);
        assert!(rect1.intersection(&rect3).is_none());
    }

    #[test]
    fn test_rect_union() {
        let rect1 = Rect::new(0, 0, 50, 50);
        let rect2 = Rect::new(100, 100, 50, 50);

        let union = rect1.union(&rect2);
        assert_eq!(union.x, 0);
        assert_eq!(union.y, 0);
        assert_eq!(union.width, 150);
        assert_eq!(union.height, 150);
    }

    #[test]
    fn test_rect_area() {
        let rect = Rect::new(0, 0, 100, 200);
        assert_eq!(rect.area(), 20000);
    }

    #[test]
    fn test_rect_is_empty() {
        assert!(Rect::new(0, 0, 0, 100).is_empty());
        assert!(Rect::new(0, 0, 100, 0).is_empty());
        assert!(!Rect::new(0, 0, 100, 100).is_empty());
    }

    // ==================== Viewport Tests ====================

    #[test]
    fn test_viewport_new() {
        let vp = Viewport::new(1920, 1080);
        assert_eq!(vp.width, 1920);
        assert_eq!(vp.height, 1080);
        assert_eq!(vp.scroll_x, 0);
        assert_eq!(vp.scroll_y, 0);
        assert_eq!(vp.zoom, 1.0);
        assert_eq!(vp.device_pixel_ratio, 1.0);
    }

    #[test]
    fn test_viewport_with_dpr() {
        let vp = Viewport::with_dpr(1920, 1080, 2.0);
        assert_eq!(vp.device_pixel_ratio, 2.0);

        let (pw, ph) = vp.physical_size();
        assert_eq!(pw, 3840);
        assert_eq!(ph, 2160);
    }

    #[test]
    fn test_viewport_set_scroll() {
        let mut vp = Viewport::new(800, 600);
        vp.set_scroll(100, 200);
        assert_eq!(vp.scroll_x, 100);
        assert_eq!(vp.scroll_y, 200);
    }

    #[test]
    fn test_viewport_set_zoom() {
        let mut vp = Viewport::new(800, 600);

        // Valid zoom
        assert!(vp.set_zoom(1.5).is_ok());
        assert_eq!(vp.zoom, 1.5);

        // Invalid zoom (too small)
        assert!(matches!(vp.set_zoom(0.05), Err(RenderError::InvalidZoom(_))));

        // Invalid zoom (too large)
        assert!(matches!(vp.set_zoom(15.0), Err(RenderError::InvalidZoom(_))));
    }

    #[test]
    fn test_viewport_visible_bounds() {
        let mut vp = Viewport::new(800, 600);
        vp.set_scroll(100, 200);
        vp.zoom = 2.0;

        let bounds = vp.visible_bounds();
        assert_eq!(bounds.x, 100);
        assert_eq!(bounds.y, 200);
        assert_eq!(bounds.width, 400);
        assert_eq!(bounds.height, 300);
    }

    #[test]
    fn test_viewport_coordinate_conversion() {
        let mut vp = Viewport::new(800, 600);
        vp.set_scroll(100, 200);
        vp.zoom = 2.0;

        // Viewport to document
        let (doc_x, doc_y) = vp.viewport_to_document(200, 100);
        assert_eq!(doc_x, 200); // 200/2 + 100
        assert_eq!(doc_y, 250); // 100/2 + 200

        // Document to viewport
        let (vp_x, vp_y) = vp.document_to_viewport(200, 300);
        assert_eq!(vp_x, 200); // (200-100)*2
        assert_eq!(vp_y, 200); // (300-200)*2
    }

    // ==================== PixelFormat Tests ====================

    #[test]
    fn test_pixel_format_bytes_per_pixel() {
        assert_eq!(PixelFormat::Rgba8.bytes_per_pixel(), 4);
        assert_eq!(PixelFormat::Bgra8.bytes_per_pixel(), 4);
        assert_eq!(PixelFormat::Rgb8.bytes_per_pixel(), 3);
        assert_eq!(PixelFormat::Rgb565.bytes_per_pixel(), 2);
    }

    // ==================== Frame Tests ====================

    #[test]
    fn test_frame_new() {
        let frame = Frame::new(100, 100, PixelFormat::Rgba8).unwrap();
        assert_eq!(frame.width, 100);
        assert_eq!(frame.height, 100);
        assert_eq!(frame.data.len(), 100 * 100 * 4);
    }

    #[test]
    fn test_frame_new_invalid() {
        assert!(matches!(
            Frame::new(0, 100, PixelFormat::Rgba8),
            Err(RenderError::InvalidViewport(0, 100))
        ));
        assert!(matches!(
            Frame::new(100, 0, PixelFormat::Rgba8),
            Err(RenderError::InvalidViewport(100, 0))
        ));
    }

    #[test]
    fn test_frame_get_set_pixel() {
        let mut frame = Frame::new(10, 10, PixelFormat::Rgba8).unwrap();

        // Set a pixel
        assert!(frame.set_pixel(5, 5, &[255, 128, 64, 255]));

        // Get the pixel
        let pixel = frame.get_pixel(5, 5).unwrap();
        assert_eq!(pixel, &[255, 128, 64, 255]);

        // Out of bounds
        assert!(frame.get_pixel(10, 5).is_none());
        assert!(!frame.set_pixel(10, 5, &[0, 0, 0, 0]));
    }

    #[test]
    fn test_frame_stride() {
        let frame = Frame::new(100, 50, PixelFormat::Rgba8).unwrap();
        assert_eq!(frame.stride(), 400); // 100 * 4
    }

    #[test]
    fn test_frame_from_data() {
        let data = vec![0u8; 100 * 100 * 4];
        let frame = Frame::from_data(100, 100, PixelFormat::Rgba8, data).unwrap();
        assert_eq!(frame.width, 100);
        assert_eq!(frame.height, 100);
    }

    #[test]
    fn test_frame_from_data_size_mismatch() {
        let data = vec![0u8; 50]; // Too small
        assert!(matches!(
            Frame::from_data(100, 100, PixelFormat::Rgba8, data),
            Err(RenderError::RenderFailed(_))
        ));
    }

    // ==================== LayerTransform Tests ====================

    #[test]
    fn test_layer_transform_identity() {
        let transform = LayerTransform::identity();
        assert!(transform.is_identity());
    }

    #[test]
    fn test_layer_transform_translate() {
        let transform = LayerTransform::translate(100.0, 200.0);
        assert!(!transform.is_identity());
        assert_eq!(transform.translate_x, 100.0);
        assert_eq!(transform.translate_y, 200.0);
    }

    #[test]
    fn test_layer_transform_scale() {
        let transform = LayerTransform::scale(2.0, 2.0);
        assert!(!transform.is_identity());
        assert_eq!(transform.scale_x, 2.0);
        assert_eq!(transform.scale_y, 2.0);
    }

    // ==================== CompositorLayer Tests ====================

    #[test]
    fn test_compositor_layer_new() {
        let layer = CompositorLayer::new(1, Rect::from_size(100, 100));
        assert_eq!(layer.id, 1);
        assert_eq!(layer.bounds.width, 100);
        assert_eq!(layer.opacity, 1.0);
        assert!(layer.visible);
        assert_eq!(layer.z_index, 0);
    }

    #[test]
    fn test_compositor_layer_builder() {
        let layer = CompositorLayer::new(1, Rect::from_size(100, 100))
            .with_opacity(0.5)
            .with_z_index(10)
            .with_transform(LayerTransform::translate(50.0, 50.0));

        assert_eq!(layer.opacity, 0.5);
        assert_eq!(layer.z_index, 10);
        assert_eq!(layer.transform.translate_x, 50.0);
    }

    // ==================== MockRenderEngine Tests ====================

    #[test]
    fn test_mock_engine_new() {
        let engine = MockRenderEngine::new(1920, 1080);
        assert_eq!(engine.dimensions(), (1920, 1080));
        assert_eq!(engine.current_zoom(), 1.0);
    }

    #[test]
    fn test_mock_engine_render_frame() {
        let mut engine = MockRenderEngine::new(100, 100);
        let viewport = Viewport::new(100, 100);

        let frame = engine.render_frame(&viewport).unwrap();
        assert_eq!(frame.width, 100);
        assert_eq!(frame.height, 100);
        assert_eq!(frame.sequence, 0);
    }

    #[test]
    fn test_mock_engine_invalidate_region() {
        let mut engine = MockRenderEngine::new(800, 600);

        engine.invalidate(Some(Rect::new(100, 100, 200, 200)));
        assert_eq!(engine.get_dirty_regions().len(), 1);

        engine.invalidate(Some(Rect::new(150, 150, 200, 200))); // Overlapping
        assert_eq!(engine.get_dirty_regions().len(), 1); // Should merge

        engine.invalidate(Some(Rect::new(500, 500, 50, 50))); // Non-overlapping
        assert_eq!(engine.get_dirty_regions().len(), 2);
    }

    #[test]
    fn test_mock_engine_invalidate_all() {
        let mut engine = MockRenderEngine::new(800, 600);

        engine.invalidate(Some(Rect::new(100, 100, 200, 200)));
        engine.invalidate(None); // Invalidate all

        let regions = engine.get_dirty_regions();
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].width, 800);
        assert_eq!(regions[0].height, 600);
    }

    #[test]
    fn test_mock_engine_resize() {
        let mut engine = MockRenderEngine::new(800, 600);
        engine.resize(1920, 1080);

        assert_eq!(engine.dimensions(), (1920, 1080));

        let regions = engine.get_dirty_regions();
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].width, 1920);
        assert_eq!(regions[0].height, 1080);
    }

    #[test]
    fn test_mock_engine_set_zoom() {
        let mut engine = MockRenderEngine::new(800, 600);
        engine.set_zoom(2.0);

        assert_eq!(engine.current_zoom(), 2.0);

        // Clamped values
        engine.set_zoom(0.01);
        assert_eq!(engine.current_zoom(), 0.1);

        engine.set_zoom(100.0);
        assert_eq!(engine.current_zoom(), 10.0);
    }

    #[test]
    fn test_mock_engine_animation_callbacks() {
        let mut engine = MockRenderEngine::new(800, 600);

        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        engine.request_animation_frame(Box::new(move |_timestamp| {
            called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        }));

        assert_eq!(engine.pending_callbacks(), 1);
        assert!(engine.needs_repaint());

        engine.process_animation_frames();

        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(engine.pending_callbacks(), 0);
    }

    #[test]
    fn test_mock_engine_layers() {
        let engine = MockRenderEngine::new(800, 600);

        // Add layers
        let layer1 = engine.add_layer(Rect::new(0, 0, 200, 200));
        let layer2 = engine.add_layer(Rect::new(100, 100, 200, 200));

        assert!(engine.get_layer(layer1).is_some());
        assert!(engine.get_layer(layer2).is_some());

        // Modify layer
        engine.update_layer(layer1, |layer| {
            layer.opacity = 0.5;
            layer.z_index = 5;
        });

        let layer = engine.get_layer(layer1).unwrap();
        assert_eq!(layer.opacity, 0.5);
        assert_eq!(layer.z_index, 5);

        // Get sorted layers
        let sorted = engine.get_layers_sorted();
        assert_eq!(sorted.len(), 2);
        assert!(sorted[0].z_index <= sorted[1].z_index);

        // Remove layer
        assert!(engine.remove_layer(layer1));
        assert!(engine.get_layer(layer1).is_none());
        assert!(!engine.remove_layer(layer1)); // Already removed
    }

    #[test]
    fn test_mock_engine_needs_repaint() {
        let mut engine = MockRenderEngine::new(800, 600);

        assert!(!engine.needs_repaint());

        engine.invalidate(Some(Rect::new(0, 0, 100, 100)));
        assert!(engine.needs_repaint());

        engine.clear_dirty_regions();
        assert!(!engine.needs_repaint());

        engine.request_animation_frame(Box::new(|_| {}));
        assert!(engine.needs_repaint());
    }

    // ==================== FrameScheduler Tests ====================

    #[test]
    fn test_frame_scheduler_new() {
        let scheduler = FrameScheduler::new();
        assert_eq!(scheduler.target_fps(), 60.0);
    }

    #[test]
    fn test_frame_scheduler_custom_fps() {
        let scheduler = FrameScheduler::with_target_fps(30.0);
        assert_eq!(scheduler.target_fps(), 30.0);
    }

    #[test]
    fn test_frame_scheduler_begin_frame() {
        let mut scheduler = FrameScheduler::new();

        std::thread::sleep(std::time::Duration::from_millis(10));
        let delta = scheduler.begin_frame();

        assert!(delta >= std::time::Duration::from_millis(10));
    }

    #[test]
    fn test_frame_scheduler_should_render() {
        let scheduler = FrameScheduler::with_target_fps(1.0); // 1 FPS for testing

        // Should not render immediately (just created)
        // Note: This might be flaky depending on timing
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert!(!scheduler.should_render());

        std::thread::sleep(std::time::Duration::from_millis(600));
        assert!(scheduler.should_render());
    }

    #[test]
    fn test_frame_scheduler_set_target_fps() {
        let mut scheduler = FrameScheduler::new();
        scheduler.set_target_fps(120.0);
        assert_eq!(scheduler.target_fps(), 120.0);
    }

    // ==================== RenderError Tests ====================

    #[test]
    fn test_render_error_display() {
        let err = RenderError::InvalidViewport(0, 100);
        assert!(err.to_string().contains("0x100"));

        let err = RenderError::InvalidZoom(-1.0);
        assert!(err.to_string().contains("-1"));

        let err = RenderError::RenderFailed("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_render_cycle() {
        let mut engine = MockRenderEngine::new(800, 600);
        let mut viewport = Viewport::new(800, 600);

        // Initial render
        let frame1 = engine.render_frame(&viewport).unwrap();
        assert_eq!(frame1.sequence, 0);

        // Scroll and re-render
        viewport.set_scroll(100, 200);
        engine.invalidate(None);
        let frame2 = engine.render_frame(&viewport).unwrap();
        assert_eq!(frame2.sequence, 1);

        // Zoom and re-render
        viewport.set_zoom(1.5).unwrap();
        engine.set_zoom(1.5);
        let frame3 = engine.render_frame(&viewport).unwrap();
        assert_eq!(frame3.sequence, 2);
    }

    #[test]
    fn test_animation_frame_timing() {
        let mut engine = MockRenderEngine::new(800, 600);
        let timestamps = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let ts_clone = timestamps.clone();

        engine.request_animation_frame(Box::new(move |ts| {
            ts_clone.lock().unwrap().push(ts);
        }));

        std::thread::sleep(std::time::Duration::from_millis(10));

        let ts_clone2 = timestamps.clone();
        engine.request_animation_frame(Box::new(move |ts| {
            ts_clone2.lock().unwrap().push(ts);
        }));

        engine.process_animation_frames();

        let ts = timestamps.lock().unwrap();
        assert_eq!(ts.len(), 2);
        // Both should have similar timestamps since processed together
        assert!((ts[0] - ts[1]).abs() < 1.0);
    }

    // ==================== DOM Integration Tests (FEAT-013) ====================

    #[test]
    fn test_node_id() {
        let id = dom::NodeId::new(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_element_data_new() {
        let elem = dom::ElementData::new("div");
        assert_eq!(elem.tag_name, "div");
        assert!(elem.namespace.is_none());
        assert!(elem.attributes.is_empty());
    }

    #[test]
    fn test_element_data_with_attribute() {
        let elem = dom::ElementData::new("div")
            .with_attribute("id", "main")
            .with_attribute("class", "container");

        assert_eq!(elem.get_attribute("id"), Some("main"));
        assert_eq!(elem.get_attribute("class"), Some("container"));
        assert_eq!(elem.get_attribute("missing"), None);
    }

    #[test]
    fn test_dom_node_element() {
        let data = dom::ElementData::new("div");
        let node = dom::DomNode::element(dom::NodeId::new(1), data);

        assert!(node.is_element());
        assert!(!node.is_text());
        assert_eq!(node.tag_name(), Some("div"));
        assert!(node.text_content.is_none());
    }

    #[test]
    fn test_dom_node_text() {
        let node = dom::DomNode::text(dom::NodeId::new(2), "Hello, World!");

        assert!(node.is_text());
        assert!(!node.is_element());
        assert_eq!(node.tag_name(), None);
        assert_eq!(node.text_content, Some("Hello, World!".to_string()));
    }

    #[test]
    fn test_computed_style_default() {
        let style = dom::ComputedStyle::new();

        assert_eq!(style.display, "block");
        assert_eq!(style.position, "static");
        assert_eq!(style.width, "auto");
        assert_eq!(style.font_size_px, 16.0);
    }

    #[test]
    fn test_stub_dom_integration_empty() {
        let stub = dom::StubDomIntegration::new();

        assert!(stub.get_document_root().is_none());
        assert!(stub.get_node(dom::NodeId::new(1)).is_none());
        assert!(stub.query_selector("div").is_empty());
        assert!(stub.query_selector_all("div").is_empty());
        assert!(stub.get_element_by_id("main").is_none());
        assert!(stub.get_elements_by_class_name("container").is_empty());
        assert!(stub.get_elements_by_tag_name("div").is_empty());
    }

    #[test]
    fn test_stub_dom_integration_with_mock_document() {
        let stub = dom::StubDomIntegration::with_mock_document();

        // Should have a document root
        let root_id = stub.get_document_root();
        assert!(root_id.is_some());

        // Should be able to get the root node
        let root_id = root_id.unwrap();
        let root = stub.get_node(root_id);
        assert!(root.is_some());

        let root = root.unwrap();
        assert_eq!(root.tag_name(), Some("html"));

        // Should have computed style for root
        let style = stub.get_computed_style(root_id);
        assert!(style.is_some());

        // Should find html element by tag name
        let html_elements = stub.get_elements_by_tag_name("html");
        assert_eq!(html_elements.len(), 1);
        assert_eq!(html_elements[0], root_id);

        // Should not find non-existent elements
        let div_elements = stub.get_elements_by_tag_name("div");
        assert!(div_elements.is_empty());
    }

    #[test]
    fn test_dom_node_serialization() {
        let data = dom::ElementData::new("span").with_attribute("class", "highlight");
        let node = dom::DomNode::element(dom::NodeId::new(42), data);

        let json = serde_json::to_string(&node).unwrap();
        let deserialized: dom::DomNode = serde_json::from_str(&json).unwrap();

        assert_eq!(node.id, deserialized.id);
        assert_eq!(node.node_type, deserialized.node_type);
        assert_eq!(node.element_data, deserialized.element_data);
    }

    #[test]
    fn test_computed_style_serialization() {
        let style = dom::ComputedStyle::new();

        let json = serde_json::to_string(&style).unwrap();
        let deserialized: dom::ComputedStyle = serde_json::from_str(&json).unwrap();

        assert_eq!(style, deserialized);
    }
}
