//! macOS platform window implementation
//!
//! This module provides a macOS window implementation following cocoa/AppKit patterns.
//! The API surface matches what would be used with the cocoa crate for native
//! macOS window management.
//!
//! # cocoa/AppKit Integration Notes
//!
//! When building with actual cocoa dependency:
//! - Use `NSWindow::alloc().initWithContentRect_styleMask_backing_defer_`
//! - Use `NSWindow::makeKeyAndOrderFront_` for showing
//! - Use `NSWindow::orderOut_` for hiding
//! - Use `NSWindow::setFrame_display_` for resize/move
//! - Use `NSWindow::makeKeyWindow` for focus

use crate::{MacOSHandle, PlatformHandle};
use shared_types::{WindowConfig, WindowError};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global counter for generating unique NSWindow pointers
static NSWINDOW_COUNTER: AtomicUsize = AtomicUsize::new(0x7FFF0000);

/// Global counter for generating unique NSView pointers
static NSVIEW_COUNTER: AtomicUsize = AtomicUsize::new(0x7FFE0000);

/// NSWindow style mask options (matching NSWindowStyleMask)
#[derive(Debug, Clone, Copy)]
pub struct WindowStyleMask {
    /// Raw style mask value
    pub raw: u64,
}

impl WindowStyleMask {
    /// NSWindowStyleMaskBorderless - No title bar, borders
    pub const BORDERLESS: u64 = 0;
    /// NSWindowStyleMaskTitled - Window has title bar
    pub const TITLED: u64 = 1 << 0;
    /// NSWindowStyleMaskClosable - Window has close button
    pub const CLOSABLE: u64 = 1 << 1;
    /// NSWindowStyleMaskMiniaturizable - Window can be minimized
    pub const MINIATURIZABLE: u64 = 1 << 2;
    /// NSWindowStyleMaskResizable - Window is resizable
    pub const RESIZABLE: u64 = 1 << 3;
    /// NSWindowStyleMaskTexturedBackground - Textured background
    pub const TEXTURED_BACKGROUND: u64 = 1 << 8;
    /// NSWindowStyleMaskUnifiedTitleAndToolbar - Unified title/toolbar
    pub const UNIFIED_TITLE_AND_TOOLBAR: u64 = 1 << 12;
    /// NSWindowStyleMaskFullScreen - Fullscreen mode
    pub const FULLSCREEN: u64 = 1 << 14;
    /// NSWindowStyleMaskFullSizeContentView - Content extends under title
    pub const FULL_SIZE_CONTENT_VIEW: u64 = 1 << 15;

    /// Standard window (titled, closable, miniaturizable, resizable)
    pub fn standard() -> Self {
        Self {
            raw: Self::TITLED | Self::CLOSABLE | Self::MINIATURIZABLE | Self::RESIZABLE,
        }
    }

    /// Borderless window
    pub fn borderless() -> Self {
        Self {
            raw: Self::BORDERLESS,
        }
    }

    /// Modern window with full-size content view
    pub fn modern() -> Self {
        Self {
            raw: Self::TITLED
                | Self::CLOSABLE
                | Self::MINIATURIZABLE
                | Self::RESIZABLE
                | Self::FULL_SIZE_CONTENT_VIEW,
        }
    }
}

impl Default for WindowStyleMask {
    fn default() -> Self {
        Self::standard()
    }
}

/// NSBackingStoreType options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackingStoreType {
    /// NSBackingStoreRetained (deprecated)
    Retained = 0,
    /// NSBackingStoreNonretained (deprecated)
    Nonretained = 1,
    /// NSBackingStoreBuffered (standard)
    Buffered = 2,
}

impl Default for BackingStoreType {
    fn default() -> Self {
        Self::Buffered
    }
}

/// NSWindowCollectionBehavior options
#[derive(Debug, Clone, Copy)]
pub struct CollectionBehavior {
    /// Raw behavior value
    pub raw: u64,
}

impl CollectionBehavior {
    /// NSWindowCollectionBehaviorDefault
    pub const DEFAULT: u64 = 0;
    /// NSWindowCollectionBehaviorCanJoinAllSpaces
    pub const CAN_JOIN_ALL_SPACES: u64 = 1 << 0;
    /// NSWindowCollectionBehaviorMoveToActiveSpace
    pub const MOVE_TO_ACTIVE_SPACE: u64 = 1 << 1;
    /// NSWindowCollectionBehaviorManaged
    pub const MANAGED: u64 = 1 << 2;
    /// NSWindowCollectionBehaviorTransient
    pub const TRANSIENT: u64 = 1 << 3;
    /// NSWindowCollectionBehaviorStationary
    pub const STATIONARY: u64 = 1 << 4;
    /// NSWindowCollectionBehaviorParticipatesInCycle
    pub const PARTICIPATES_IN_CYCLE: u64 = 1 << 5;
    /// NSWindowCollectionBehaviorIgnoresCycle
    pub const IGNORES_CYCLE: u64 = 1 << 6;
    /// NSWindowCollectionBehaviorFullScreenPrimary
    pub const FULLSCREEN_PRIMARY: u64 = 1 << 7;
    /// NSWindowCollectionBehaviorFullScreenAuxiliary
    pub const FULLSCREEN_AUXILIARY: u64 = 1 << 8;

    /// Default behavior
    pub fn default_behavior() -> Self {
        Self { raw: Self::DEFAULT }
    }
}

impl Default for CollectionBehavior {
    fn default() -> Self {
        Self::default_behavior()
    }
}

/// macOS window state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacWindowState {
    /// Window is visible (ordered in)
    pub visible: bool,
    /// Window is key (has keyboard focus)
    pub is_key: bool,
    /// Window is main (main window of app)
    pub is_main: bool,
    /// Window is miniaturized (minimized)
    pub miniaturized: bool,
    /// Window is zoomed (maximized)
    pub zoomed: bool,
    /// Window is in fullscreen mode
    pub fullscreen: bool,
}

impl Default for MacWindowState {
    fn default() -> Self {
        Self {
            visible: false,
            is_key: false,
            is_main: false,
            miniaturized: false,
            zoomed: false,
            fullscreen: false,
        }
    }
}

/// NSRect-style rectangle (using CGRect conventions)
#[derive(Debug, Clone, Copy)]
pub struct NSRect {
    /// Origin X (bottom-left in AppKit coordinates)
    pub x: f64,
    /// Origin Y (bottom-left in AppKit coordinates)
    pub y: f64,
    /// Width
    pub width: f64,
    /// Height
    pub height: f64,
}

impl NSRect {
    /// Create a new rect
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    /// Create from integer coordinates
    pub fn from_coords(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x: x as f64,
            y: y as f64,
            width: width as f64,
            height: height as f64,
        }
    }
}

/// Appearance mode (light/dark)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppearanceMode {
    /// NSAppearanceNameAqua - Light mode
    Aqua,
    /// NSAppearanceNameDarkAqua - Dark mode
    DarkAqua,
    /// NSAppearanceNameVibrantLight
    VibrantLight,
    /// NSAppearanceNameVibrantDark
    VibrantDark,
    /// System default
    System,
}

impl Default for AppearanceMode {
    fn default() -> Self {
        Self::System
    }
}

/// macOS window implementation
///
/// This implementation provides an API surface compatible with cocoa/AppKit patterns.
/// In a full implementation, this would wrap actual NSWindow objects.
#[derive(Debug)]
pub struct MacWindow {
    /// NSWindow pointer value
    ns_window: usize,
    /// NSView pointer for content view
    ns_view: usize,
    /// CALayer pointer (if layer-backed)
    ca_layer: usize,
    /// Window state
    state: MacWindowState,
    /// Style mask
    style_mask: WindowStyleMask,
    /// Collection behavior
    collection_behavior: CollectionBehavior,
    /// Window frame (in screen coordinates)
    frame: NSRect,
    /// Content rect (excludes title bar)
    content_rect: NSRect,
    /// Window title
    title: String,
    /// Backing store type
    backing: BackingStoreType,
    /// Window level (NSWindowLevel)
    level: i64,
    /// Appearance mode
    appearance: AppearanceMode,
    /// Title bar transparency
    titlebar_transparent: bool,
    /// Title visibility
    title_visible: bool,
    /// Toolbar style (macOS 11+)
    toolbar_style: u64,
}

impl MacWindow {
    /// Create with custom options
    pub fn create_with_options(
        config: &WindowConfig,
        style_mask: WindowStyleMask,
        backing: BackingStoreType,
    ) -> Result<Self, WindowError> {
        let ns_window = NSWINDOW_COUNTER.fetch_add(0x1000, Ordering::SeqCst);
        let ns_view = NSVIEW_COUNTER.fetch_add(0x1000, Ordering::SeqCst);
        // Mock CA layer (created when layer-backed)
        let ca_layer = ns_view + 0x100;

        log::debug!(
            "MacWindow::create_with_options - NSWindow: 0x{:X}, title: '{}', size: {}x{}",
            ns_window,
            config.title,
            config.width,
            config.height
        );

        // macOS uses bottom-left origin, so flip Y coordinate
        // (in a real implementation, we'd query screen height)
        let screen_height = 1080.0;
        let y = screen_height - config.y.unwrap_or(100) as f64 - config.height as f64;

        let frame = NSRect::from_coords(
            config.x.unwrap_or(100),
            y as i32,
            config.width,
            config.height,
        );

        // Content rect excludes title bar (approximately 22 pixels)
        let content_rect = NSRect::new(0.0, 0.0, config.width as f64, config.height as f64 - 22.0);

        Ok(Self {
            ns_window,
            ns_view,
            ca_layer,
            state: MacWindowState::default(),
            style_mask,
            collection_behavior: CollectionBehavior::default(),
            frame,
            content_rect,
            title: config.title.clone(),
            backing,
            level: 0, // NSNormalWindowLevel
            appearance: AppearanceMode::System,
            titlebar_transparent: false,
            title_visible: true,
            toolbar_style: 0, // NSWindowToolbarStyleAutomatic
        })
    }

    /// Get NSWindow pointer
    pub fn ns_window(&self) -> usize {
        self.ns_window
    }

    /// Get content NSView pointer
    pub fn ns_view(&self) -> usize {
        self.ns_view
    }

    /// Get CALayer pointer
    pub fn ca_layer(&self) -> usize {
        self.ca_layer
    }

    /// Get current state
    pub fn state(&self) -> &MacWindowState {
        &self.state
    }

    /// Get window frame
    pub fn frame(&self) -> &NSRect {
        &self.frame
    }

    /// Get content rect
    pub fn content_rect(&self) -> &NSRect {
        &self.content_rect
    }

    /// Set window title
    pub fn set_title(&mut self, title: &str) -> Result<(), WindowError> {
        log::debug!(
            "MacWindow::set_title - NSWindow: 0x{:X}, title: '{}'",
            self.ns_window,
            title
        );
        self.title = title.to_string();
        // In real implementation: [window setTitle:title]
        Ok(())
    }

    /// Miniaturize (minimize) the window
    pub fn miniaturize(&mut self) -> Result<(), WindowError> {
        log::debug!("MacWindow::miniaturize - NSWindow: 0x{:X}", self.ns_window);
        self.state.miniaturized = true;
        // In real implementation: [window miniaturize:nil]
        Ok(())
    }

    /// Deminiaturize (restore from dock)
    pub fn deminiaturize(&mut self) -> Result<(), WindowError> {
        log::debug!("MacWindow::deminiaturize - NSWindow: 0x{:X}", self.ns_window);
        self.state.miniaturized = false;
        // In real implementation: [window deminiaturize:nil]
        Ok(())
    }

    /// Zoom (maximize) the window
    pub fn zoom(&mut self) -> Result<(), WindowError> {
        log::debug!("MacWindow::zoom - NSWindow: 0x{:X}", self.ns_window);
        self.state.zoomed = !self.state.zoomed;
        // In real implementation: [window zoom:nil]
        Ok(())
    }

    /// Toggle fullscreen mode
    pub fn toggle_fullscreen(&mut self) -> Result<(), WindowError> {
        log::debug!("MacWindow::toggle_fullscreen - NSWindow: 0x{:X}", self.ns_window);
        self.state.fullscreen = !self.state.fullscreen;
        // In real implementation: [window toggleFullScreen:nil]
        Ok(())
    }

    /// Set fullscreen directly
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result<(), WindowError> {
        log::debug!(
            "MacWindow::set_fullscreen - NSWindow: 0x{:X}, fullscreen: {}",
            self.ns_window,
            fullscreen
        );
        if fullscreen != self.state.fullscreen {
            self.state.fullscreen = fullscreen;
            // In real implementation: Toggle fullscreen if needed
        }
        Ok(())
    }

    /// Set window level (z-ordering)
    pub fn set_level(&mut self, level: i64) -> Result<(), WindowError> {
        log::debug!(
            "MacWindow::set_level - NSWindow: 0x{:X}, level: {}",
            self.ns_window,
            level
        );
        self.level = level;
        // In real implementation: [window setLevel:level]
        Ok(())
    }

    /// NSNormalWindowLevel = 0
    pub const LEVEL_NORMAL: i64 = 0;
    /// NSFloatingWindowLevel = 3
    pub const LEVEL_FLOATING: i64 = 3;
    /// NSStatusWindowLevel = 25
    pub const LEVEL_STATUS: i64 = 25;
    /// NSModalPanelWindowLevel = 8
    pub const LEVEL_MODAL: i64 = 8;

    /// Set appearance mode
    pub fn set_appearance(&mut self, appearance: AppearanceMode) -> Result<(), WindowError> {
        log::debug!(
            "MacWindow::set_appearance - NSWindow: 0x{:X}, appearance: {:?}",
            self.ns_window,
            appearance
        );
        self.appearance = appearance;
        // In real implementation: [window setAppearance:...]
        Ok(())
    }

    /// Set title bar transparency
    pub fn set_titlebar_transparent(&mut self, transparent: bool) -> Result<(), WindowError> {
        log::debug!(
            "MacWindow::set_titlebar_transparent - NSWindow: 0x{:X}, transparent: {}",
            self.ns_window,
            transparent
        );
        self.titlebar_transparent = transparent;
        // In real implementation: [window setTitlebarAppearsTransparent:transparent]
        Ok(())
    }

    /// Set title visibility
    pub fn set_title_visibility(&mut self, visible: bool) -> Result<(), WindowError> {
        log::debug!(
            "MacWindow::set_title_visibility - NSWindow: 0x{:X}, visible: {}",
            self.ns_window,
            visible
        );
        self.title_visible = visible;
        // In real implementation: [window setTitleVisibility:visible ? NSWindowTitleVisible : NSWindowTitleHidden]
        Ok(())
    }

    /// Set collection behavior
    pub fn set_collection_behavior(&mut self, behavior: CollectionBehavior) -> Result<(), WindowError> {
        log::debug!(
            "MacWindow::set_collection_behavior - NSWindow: 0x{:X}, behavior: 0x{:X}",
            self.ns_window,
            behavior.raw
        );
        self.collection_behavior = behavior;
        // In real implementation: [window setCollectionBehavior:behavior]
        Ok(())
    }

    /// Center window on screen
    pub fn center(&mut self) -> Result<(), WindowError> {
        log::debug!("MacWindow::center - NSWindow: 0x{:X}", self.ns_window);
        // In real implementation: [window center]
        // Update frame to centered position (simplified)
        self.frame.x = 400.0;
        self.frame.y = 300.0;
        Ok(())
    }

    /// Make window opaque or transparent
    pub fn set_opaque(&mut self, opaque: bool) -> Result<(), WindowError> {
        log::debug!(
            "MacWindow::set_opaque - NSWindow: 0x{:X}, opaque: {}",
            self.ns_window,
            opaque
        );
        // In real implementation: [window setOpaque:opaque]
        Ok(())
    }

    /// Set background color (as rgba values 0.0-1.0)
    pub fn set_background_color(&self, r: f64, g: f64, b: f64, a: f64) -> Result<(), WindowError> {
        log::debug!(
            "MacWindow::set_background_color - NSWindow: 0x{:X}, rgba: ({}, {}, {}, {})",
            self.ns_window,
            r, g, b, a
        );
        // In real implementation: [window setBackgroundColor:[NSColor colorWithRed:...]]
        Ok(())
    }

    /// Invalidate shadow
    pub fn invalidate_shadow(&self) -> Result<(), WindowError> {
        log::trace!("MacWindow::invalidate_shadow - NSWindow: 0x{:X}", self.ns_window);
        // In real implementation: [window invalidateShadow]
        Ok(())
    }

    /// Display if needed
    pub fn display_if_needed(&self) -> Result<(), WindowError> {
        log::trace!("MacWindow::display_if_needed - NSWindow: 0x{:X}", self.ns_window);
        // In real implementation: [window displayIfNeeded]
        Ok(())
    }
}

impl crate::PlatformWindow for MacWindow {
    fn create(config: &WindowConfig) -> Result<Self, WindowError> {
        let style_mask = if config.decorations {
            if config.resizable {
                WindowStyleMask::standard()
            } else {
                WindowStyleMask {
                    raw: WindowStyleMask::TITLED | WindowStyleMask::CLOSABLE | WindowStyleMask::MINIATURIZABLE,
                }
            }
        } else {
            WindowStyleMask::borderless()
        };

        Self::create_with_options(config, style_mask, BackingStoreType::Buffered)
    }

    fn destroy(&mut self) -> Result<(), WindowError> {
        log::debug!("MacWindow::destroy - NSWindow: 0x{:X}", self.ns_window);
        // In real implementation: [window close] or [window release]
        Ok(())
    }

    fn show(&mut self) -> Result<(), WindowError> {
        log::debug!("MacWindow::show - NSWindow: 0x{:X}", self.ns_window);
        self.state.visible = true;
        // In real implementation: [window makeKeyAndOrderFront:nil]
        Ok(())
    }

    fn hide(&mut self) -> Result<(), WindowError> {
        log::debug!("MacWindow::hide - NSWindow: 0x{:X}", self.ns_window);
        self.state.visible = false;
        // In real implementation: [window orderOut:nil]
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), WindowError> {
        log::debug!(
            "MacWindow::resize - NSWindow: 0x{:X}, size: {}x{}",
            self.ns_window,
            width,
            height
        );
        self.frame.width = width as f64;
        self.frame.height = height as f64;
        self.content_rect.width = width as f64;
        self.content_rect.height = height as f64 - 22.0;
        // In real implementation: [window setFrame:frame display:YES]
        Ok(())
    }

    fn move_to(&mut self, x: i32, y: i32) -> Result<(), WindowError> {
        log::debug!(
            "MacWindow::move_to - NSWindow: 0x{:X}, position: ({}, {})",
            self.ns_window,
            x,
            y
        );
        // Convert to AppKit coordinates (flip Y)
        let screen_height = 1080.0;
        self.frame.x = x as f64;
        self.frame.y = screen_height - y as f64 - self.frame.height;
        // In real implementation: [window setFrameTopLeftPoint:point]
        Ok(())
    }

    fn focus(&mut self) -> Result<(), WindowError> {
        log::debug!("MacWindow::focus - NSWindow: 0x{:X}", self.ns_window);
        self.state.is_key = true;
        self.state.is_main = true;
        // In real implementation: [window makeKeyAndOrderFront:nil]
        Ok(())
    }

    fn get_handle(&self) -> PlatformHandle {
        PlatformHandle::MacOS(MacOSHandle {
            ns_window: self.ns_window,
            ns_view: self.ns_view,
            ca_layer: self.ca_layer,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PlatformWindow;

    #[test]
    fn test_mac_window_create() {
        let config = WindowConfig::default();
        let result = MacWindow::create(&config);
        assert!(result.is_ok());
        let window = result.unwrap();
        assert!(window.ns_window() >= 0x7FFF0000);
    }

    #[test]
    fn test_mac_window_operations() {
        let config = WindowConfig::default();
        let mut window = MacWindow::create(&config).unwrap();

        assert!(window.show().is_ok());
        assert!(window.state().visible);

        assert!(window.hide().is_ok());
        assert!(!window.state().visible);

        assert!(window.resize(800, 600).is_ok());
        assert!((window.frame().width - 800.0).abs() < 0.001);

        assert!(window.move_to(100, 200).is_ok());
        assert!(window.focus().is_ok());
        assert!(window.destroy().is_ok());
    }

    #[test]
    fn test_mac_window_handle() {
        let config = WindowConfig::default();
        let window = MacWindow::create(&config).unwrap();
        let handle = window.get_handle();

        assert!(handle.is_macos());
        match handle {
            PlatformHandle::MacOS(h) => {
                assert_eq!(h.ns_window, window.ns_window());
                assert_eq!(h.ns_view, window.ns_view());
            }
            _ => panic!("Expected MacOS handle"),
        }
    }

    #[test]
    fn test_mac_window_miniaturize() {
        let config = WindowConfig::default();
        let mut window = MacWindow::create(&config).unwrap();

        assert!(window.miniaturize().is_ok());
        assert!(window.state().miniaturized);

        assert!(window.deminiaturize().is_ok());
        assert!(!window.state().miniaturized);
    }

    #[test]
    fn test_mac_window_fullscreen() {
        let config = WindowConfig::default();
        let mut window = MacWindow::create(&config).unwrap();

        assert!(!window.state().fullscreen);
        assert!(window.toggle_fullscreen().is_ok());
        assert!(window.state().fullscreen);
        assert!(window.toggle_fullscreen().is_ok());
        assert!(!window.state().fullscreen);
    }

    #[test]
    fn test_mac_window_appearance() {
        let config = WindowConfig::default();
        let mut window = MacWindow::create(&config).unwrap();

        assert!(window.set_appearance(AppearanceMode::DarkAqua).is_ok());
        assert_eq!(window.appearance, AppearanceMode::DarkAqua);
    }

    #[test]
    fn test_mac_window_level() {
        let config = WindowConfig::default();
        let mut window = MacWindow::create(&config).unwrap();

        assert!(window.set_level(MacWindow::LEVEL_FLOATING).is_ok());
        assert_eq!(window.level, MacWindow::LEVEL_FLOATING);
    }

    #[test]
    fn test_mac_window_style_mask() {
        let config = WindowConfig {
            decorations: false,
            ..Default::default()
        };

        let window = MacWindow::create(&config).unwrap();
        assert_eq!(window.style_mask.raw, WindowStyleMask::BORDERLESS);
    }
}
