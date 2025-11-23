//! Windows platform window implementation
//!
//! This module provides a Windows window implementation following windows-rs patterns.
//! The API surface matches what would be used with the windows crate for native
//! Win32 window management.
//!
//! # windows-rs Integration Notes
//!
//! When building with actual windows crate dependency:
//! - Use `CreateWindowExW` for window creation
//! - Use `ShowWindow` with SW_SHOW/SW_HIDE
//! - Use `SetWindowPos` for resize/move
//! - Use `SetForegroundWindow` for focus
//! - Use `DestroyWindow` for cleanup

use crate::{PlatformHandle, WindowsHandle};
use shared_types::{WindowConfig, WindowError};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global counter for generating unique HWND values
static HWND_COUNTER: AtomicUsize = AtomicUsize::new(0x00010000);

/// Mock HINSTANCE value (module handle)
static HINSTANCE_VALUE: AtomicUsize = AtomicUsize::new(0x00400000);

/// Win32 window styles (matching WS_* constants)
#[derive(Debug, Clone, Copy)]
pub struct WindowStyle {
    /// Base window style (WS_*)
    pub style: u32,
    /// Extended window style (WS_EX_*)
    pub ex_style: u32,
}

impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            // WS_OVERLAPPEDWINDOW = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU |
            //                       WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX
            style: 0x00CF_0000,
            // WS_EX_APPWINDOW (appears on taskbar)
            ex_style: 0x0004_0000,
        }
    }
}

impl WindowStyle {
    /// WS_VISIBLE - Window is initially visible
    pub const WS_VISIBLE: u32 = 0x1000_0000;
    /// WS_CAPTION - Window has title bar
    pub const WS_CAPTION: u32 = 0x00C0_0000;
    /// WS_SYSMENU - Window has system menu
    pub const WS_SYSMENU: u32 = 0x0008_0000;
    /// WS_THICKFRAME - Window has sizing border
    pub const WS_THICKFRAME: u32 = 0x0004_0000;
    /// WS_MINIMIZEBOX - Window has minimize button
    pub const WS_MINIMIZEBOX: u32 = 0x0002_0000;
    /// WS_MAXIMIZEBOX - Window has maximize button
    pub const WS_MAXIMIZEBOX: u32 = 0x0001_0000;
    /// WS_POPUP - Popup window
    pub const WS_POPUP: u32 = 0x8000_0000;
    /// WS_CHILD - Child window
    pub const WS_CHILD: u32 = 0x4000_0000;

    /// WS_EX_TOPMOST - Window stays on top
    pub const WS_EX_TOPMOST: u32 = 0x0000_0008;
    /// WS_EX_TOOLWINDOW - Tool window (smaller title bar)
    pub const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
    /// WS_EX_APPWINDOW - Force onto taskbar
    pub const WS_EX_APPWINDOW: u32 = 0x0004_0000;
    /// WS_EX_LAYERED - Layered window (for transparency)
    pub const WS_EX_LAYERED: u32 = 0x0008_0000;
    /// WS_EX_COMPOSITED - Double-buffered painting
    pub const WS_EX_COMPOSITED: u32 = 0x0200_0000;

    /// Create borderless window style
    pub fn borderless() -> Self {
        Self {
            style: Self::WS_POPUP | Self::WS_VISIBLE,
            ex_style: Self::WS_EX_APPWINDOW,
        }
    }

    /// Create always-on-top window style
    pub fn always_on_top() -> Self {
        let mut style = Self::default();
        style.ex_style |= Self::WS_EX_TOPMOST;
        style
    }
}

/// Windows window state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsWindowState {
    /// Window is visible
    pub visible: bool,
    /// Window has focus
    pub focused: bool,
    /// Window is minimized
    pub minimized: bool,
    /// Window is maximized
    pub maximized: bool,
    /// Window is enabled (accepts input)
    pub enabled: bool,
    /// Window is active (foreground)
    pub active: bool,
}

impl Default for WindowsWindowState {
    fn default() -> Self {
        Self {
            visible: false,
            focused: false,
            minimized: false,
            maximized: false,
            enabled: true,
            active: false,
        }
    }
}

/// Windows window rectangle (matching RECT structure)
#[derive(Debug, Clone, Copy)]
pub struct WindowRect {
    /// Left coordinate
    pub left: i32,
    /// Top coordinate
    pub top: i32,
    /// Right coordinate
    pub right: i32,
    /// Bottom coordinate
    pub bottom: i32,
}

impl WindowRect {
    /// Create from position and size
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            left: x,
            top: y,
            right: x + width as i32,
            bottom: y + height as i32,
        }
    }

    /// Get width
    pub fn width(&self) -> u32 {
        (self.right - self.left) as u32
    }

    /// Get height
    pub fn height(&self) -> u32 {
        (self.bottom - self.top) as u32
    }
}

/// DWM (Desktop Window Manager) attributes
#[derive(Debug, Clone)]
pub struct DwmAttributes {
    /// Enable blur behind window
    pub blur_behind: bool,
    /// Use dark mode title bar
    pub dark_mode: bool,
    /// Mica effect (Windows 11)
    pub mica_effect: bool,
    /// Custom caption color (COLORREF)
    pub caption_color: Option<u32>,
    /// Custom border color (COLORREF)
    pub border_color: Option<u32>,
}

impl Default for DwmAttributes {
    fn default() -> Self {
        Self {
            blur_behind: false,
            dark_mode: false,
            mica_effect: false,
            caption_color: None,
            border_color: None,
        }
    }
}

/// Windows window implementation
///
/// This implementation provides an API surface compatible with windows-rs patterns.
/// In a full implementation, this would wrap actual Win32 window handles.
#[derive(Debug)]
pub struct WindowsWindow {
    /// HWND value
    hwnd: usize,
    /// HINSTANCE value
    hinstance: usize,
    /// Window class name hash
    class_name_hash: u32,
    /// Window state
    state: WindowsWindowState,
    /// Window style
    style: WindowStyle,
    /// Window rectangle
    rect: WindowRect,
    /// Client rectangle
    client_rect: WindowRect,
    /// Window title
    title: String,
    /// DWM attributes
    dwm_attributes: DwmAttributes,
    /// DPI awareness
    dpi: u32,
}

impl WindowsWindow {
    /// Create with custom style
    pub fn create_with_style(
        config: &WindowConfig,
        style: WindowStyle,
    ) -> Result<Self, WindowError> {
        let hwnd = HWND_COUNTER.fetch_add(0x10, Ordering::SeqCst);
        let hinstance = HINSTANCE_VALUE.load(Ordering::SeqCst);
        let class_name_hash = Self::hash_class_name("CortenBrowserWindow");

        log::debug!(
            "WindowsWindow::create_with_style - HWND: 0x{:X}, title: '{}', size: {}x{}",
            hwnd,
            config.title,
            config.width,
            config.height
        );

        let x = config.x.unwrap_or(100); // CW_USEDEFAULT would be -2147483648
        let y = config.y.unwrap_or(100);

        let rect = WindowRect::new(x, y, config.width, config.height);
        // Client rect is window rect minus borders (simplified)
        let client_rect = WindowRect::new(0, 0, config.width, config.height);

        Ok(Self {
            hwnd,
            hinstance,
            class_name_hash,
            state: WindowsWindowState::default(),
            style,
            rect,
            client_rect,
            title: config.title.clone(),
            dwm_attributes: DwmAttributes::default(),
            dpi: 96, // Default DPI
        })
    }

    /// Hash window class name
    fn hash_class_name(name: &str) -> u32 {
        let mut hash: u32 = 5381;
        for byte in name.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
        }
        hash
    }

    /// Get HWND value
    pub fn hwnd(&self) -> usize {
        self.hwnd
    }

    /// Get HINSTANCE value
    pub fn hinstance(&self) -> usize {
        self.hinstance
    }

    /// Get current state
    pub fn state(&self) -> &WindowsWindowState {
        &self.state
    }

    /// Get window rectangle
    pub fn window_rect(&self) -> &WindowRect {
        &self.rect
    }

    /// Get client rectangle
    pub fn client_rect(&self) -> &WindowRect {
        &self.client_rect
    }

    /// Get current DPI
    pub fn dpi(&self) -> u32 {
        self.dpi
    }

    /// Set window title
    pub fn set_title(&mut self, title: &str) -> Result<(), WindowError> {
        log::debug!(
            "WindowsWindow::set_title - HWND: 0x{:X}, title: '{}'",
            self.hwnd,
            title
        );
        self.title = title.to_string();
        // In real implementation: SetWindowTextW(hwnd, title)
        Ok(())
    }

    /// Minimize the window
    pub fn minimize(&mut self) -> Result<(), WindowError> {
        log::debug!("WindowsWindow::minimize - HWND: 0x{:X}", self.hwnd);
        self.state.minimized = true;
        self.state.maximized = false;
        // In real implementation: ShowWindow(hwnd, SW_MINIMIZE)
        Ok(())
    }

    /// Maximize the window
    pub fn maximize(&mut self) -> Result<(), WindowError> {
        log::debug!("WindowsWindow::maximize - HWND: 0x{:X}", self.hwnd);
        self.state.maximized = true;
        self.state.minimized = false;
        // In real implementation: ShowWindow(hwnd, SW_MAXIMIZE)
        Ok(())
    }

    /// Restore the window from minimized/maximized state
    pub fn restore(&mut self) -> Result<(), WindowError> {
        log::debug!("WindowsWindow::restore - HWND: 0x{:X}", self.hwnd);
        self.state.minimized = false;
        self.state.maximized = false;
        // In real implementation: ShowWindow(hwnd, SW_RESTORE)
        Ok(())
    }

    /// Set fullscreen state
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result<(), WindowError> {
        log::debug!(
            "WindowsWindow::set_fullscreen - HWND: 0x{:X}, fullscreen: {}",
            self.hwnd,
            fullscreen
        );
        if fullscreen {
            // In real implementation: Remove WS_OVERLAPPEDWINDOW, set WS_POPUP,
            // SetWindowPos to cover entire monitor
            self.style.style = WindowStyle::WS_POPUP | WindowStyle::WS_VISIBLE;
        } else {
            // In real implementation: Restore original style and position
            self.style = WindowStyle::default();
            self.style.style |= WindowStyle::WS_VISIBLE;
        }
        Ok(())
    }

    /// Set always on top state
    pub fn set_always_on_top(&mut self, on_top: bool) -> Result<(), WindowError> {
        log::debug!(
            "WindowsWindow::set_always_on_top - HWND: 0x{:X}, on_top: {}",
            self.hwnd,
            on_top
        );
        if on_top {
            self.style.ex_style |= WindowStyle::WS_EX_TOPMOST;
        } else {
            self.style.ex_style &= !WindowStyle::WS_EX_TOPMOST;
        }
        // In real implementation: SetWindowPos(hwnd, HWND_TOPMOST/HWND_NOTOPMOST, ...)
        Ok(())
    }

    /// Enable/disable the window
    pub fn set_enabled(&mut self, enabled: bool) -> Result<(), WindowError> {
        log::debug!(
            "WindowsWindow::set_enabled - HWND: 0x{:X}, enabled: {}",
            self.hwnd,
            enabled
        );
        self.state.enabled = enabled;
        // In real implementation: EnableWindow(hwnd, enabled)
        Ok(())
    }

    /// Set DWM attributes (for Aero/DWM effects)
    pub fn set_dwm_attributes(&mut self, attributes: DwmAttributes) -> Result<(), WindowError> {
        log::debug!(
            "WindowsWindow::set_dwm_attributes - HWND: 0x{:X}, dark_mode: {}, mica: {}",
            self.hwnd,
            attributes.dark_mode,
            attributes.mica_effect
        );
        self.dwm_attributes = attributes;
        // In real implementation: DwmSetWindowAttribute with various attributes
        Ok(())
    }

    /// Flash the window in taskbar
    pub fn flash(&self, count: u32) -> Result<(), WindowError> {
        log::debug!(
            "WindowsWindow::flash - HWND: 0x{:X}, count: {}",
            self.hwnd,
            count
        );
        // In real implementation: FlashWindowEx
        Ok(())
    }

    /// Set window icon
    pub fn set_icon(&self, icon_data: &[u8], _width: u32, _height: u32) -> Result<(), WindowError> {
        log::debug!(
            "WindowsWindow::set_icon - HWND: 0x{:X}, icon_size: {} bytes",
            self.hwnd,
            icon_data.len()
        );
        // In real implementation: CreateIconFromResource, SendMessage WM_SETICON
        Ok(())
    }

    /// Invalidate window for repainting
    pub fn invalidate(&self, erase: bool) -> Result<(), WindowError> {
        log::trace!(
            "WindowsWindow::invalidate - HWND: 0x{:X}, erase: {}",
            self.hwnd,
            erase
        );
        // In real implementation: InvalidateRect(hwnd, NULL, erase)
        Ok(())
    }

    /// Send a custom window message
    pub fn send_message(&self, msg: u32, wparam: usize, lparam: isize) -> Result<isize, WindowError> {
        log::trace!(
            "WindowsWindow::send_message - HWND: 0x{:X}, msg: 0x{:X}, wparam: {}, lparam: {}",
            self.hwnd,
            msg,
            wparam,
            lparam
        );
        // In real implementation: SendMessage(hwnd, msg, wparam, lparam)
        Ok(0)
    }

    /// Post a custom window message (async)
    pub fn post_message(&self, msg: u32, wparam: usize, lparam: isize) -> Result<(), WindowError> {
        log::trace!(
            "WindowsWindow::post_message - HWND: 0x{:X}, msg: 0x{:X}, wparam: {}, lparam: {}",
            self.hwnd,
            msg,
            wparam,
            lparam
        );
        // In real implementation: PostMessage(hwnd, msg, wparam, lparam)
        Ok(())
    }
}

impl crate::PlatformWindow for WindowsWindow {
    fn create(config: &WindowConfig) -> Result<Self, WindowError> {
        let mut style = WindowStyle::default();

        if !config.decorations {
            style = WindowStyle::borderless();
        }

        if config.always_on_top {
            style.ex_style |= WindowStyle::WS_EX_TOPMOST;
        }

        if !config.resizable {
            style.style &= !WindowStyle::WS_THICKFRAME;
            style.style &= !WindowStyle::WS_MAXIMIZEBOX;
        }

        Self::create_with_style(config, style)
    }

    fn destroy(&mut self) -> Result<(), WindowError> {
        log::debug!("WindowsWindow::destroy - HWND: 0x{:X}", self.hwnd);
        // In real implementation: DestroyWindow(hwnd)
        Ok(())
    }

    fn show(&mut self) -> Result<(), WindowError> {
        log::debug!("WindowsWindow::show - HWND: 0x{:X}", self.hwnd);
        self.state.visible = true;
        // In real implementation: ShowWindow(hwnd, SW_SHOW)
        Ok(())
    }

    fn hide(&mut self) -> Result<(), WindowError> {
        log::debug!("WindowsWindow::hide - HWND: 0x{:X}", self.hwnd);
        self.state.visible = false;
        // In real implementation: ShowWindow(hwnd, SW_HIDE)
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), WindowError> {
        log::debug!(
            "WindowsWindow::resize - HWND: 0x{:X}, size: {}x{}",
            self.hwnd,
            width,
            height
        );
        self.rect = WindowRect::new(self.rect.left, self.rect.top, width, height);
        self.client_rect = WindowRect::new(0, 0, width, height);
        // In real implementation: SetWindowPos(hwnd, NULL, 0, 0, width, height, SWP_NOMOVE | SWP_NOZORDER)
        Ok(())
    }

    fn move_to(&mut self, x: i32, y: i32) -> Result<(), WindowError> {
        log::debug!(
            "WindowsWindow::move_to - HWND: 0x{:X}, position: ({}, {})",
            self.hwnd,
            x,
            y
        );
        let width = self.rect.width();
        let height = self.rect.height();
        self.rect = WindowRect::new(x, y, width, height);
        // In real implementation: SetWindowPos(hwnd, NULL, x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER)
        Ok(())
    }

    fn focus(&mut self) -> Result<(), WindowError> {
        log::debug!("WindowsWindow::focus - HWND: 0x{:X}", self.hwnd);
        self.state.focused = true;
        self.state.active = true;
        // In real implementation: SetForegroundWindow(hwnd) and SetFocus(hwnd)
        Ok(())
    }

    fn get_handle(&self) -> PlatformHandle {
        PlatformHandle::Windows(WindowsHandle {
            hwnd: self.hwnd,
            hinstance: self.hinstance,
            class_name_hash: self.class_name_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PlatformWindow;

    #[test]
    fn test_windows_window_create() {
        let config = WindowConfig::default();
        let result = WindowsWindow::create(&config);
        assert!(result.is_ok());
        let window = result.unwrap();
        assert!(window.hwnd() >= 0x10000);
    }

    #[test]
    fn test_windows_window_operations() {
        let config = WindowConfig::default();
        let mut window = WindowsWindow::create(&config).unwrap();

        assert!(window.show().is_ok());
        assert!(window.state().visible);

        assert!(window.hide().is_ok());
        assert!(!window.state().visible);

        assert!(window.resize(800, 600).is_ok());
        assert_eq!(window.window_rect().width(), 800);
        assert_eq!(window.window_rect().height(), 600);

        assert!(window.move_to(100, 200).is_ok());
        assert_eq!(window.window_rect().left, 100);
        assert_eq!(window.window_rect().top, 200);

        assert!(window.focus().is_ok());
        assert!(window.destroy().is_ok());
    }

    #[test]
    fn test_windows_window_handle() {
        let config = WindowConfig::default();
        let window = WindowsWindow::create(&config).unwrap();
        let handle = window.get_handle();

        assert!(handle.is_windows());
        match handle {
            PlatformHandle::Windows(h) => {
                assert_eq!(h.hwnd, window.hwnd());
            }
            _ => panic!("Expected Windows handle"),
        }
    }

    #[test]
    fn test_windows_window_minimize_maximize() {
        let config = WindowConfig::default();
        let mut window = WindowsWindow::create(&config).unwrap();

        assert!(window.minimize().is_ok());
        assert!(window.state().minimized);
        assert!(!window.state().maximized);

        assert!(window.maximize().is_ok());
        assert!(window.state().maximized);
        assert!(!window.state().minimized);

        assert!(window.restore().is_ok());
        assert!(!window.state().minimized);
        assert!(!window.state().maximized);
    }

    #[test]
    fn test_windows_window_style() {
        let config = WindowConfig {
            decorations: false,
            always_on_top: true,
            ..Default::default()
        };

        let window = WindowsWindow::create(&config).unwrap();
        assert!((window.style.ex_style & WindowStyle::WS_EX_TOPMOST) != 0);
    }

    #[test]
    fn test_windows_dwm_attributes() {
        let config = WindowConfig::default();
        let mut window = WindowsWindow::create(&config).unwrap();

        let attrs = DwmAttributes {
            dark_mode: true,
            mica_effect: true,
            ..Default::default()
        };

        assert!(window.set_dwm_attributes(attrs).is_ok());
        assert!(window.dwm_attributes.dark_mode);
        assert!(window.dwm_attributes.mica_effect);
    }
}
