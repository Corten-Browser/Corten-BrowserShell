//! PWA (Progressive Web App) Manager Component
//!
//! This component provides PWA installation and management support for the
//! CortenBrowser Browser Shell, including:
//!
//! - Web App Manifest parsing (manifest.json)
//! - PWA installation and uninstallation
//! - Installed PWA tracking and persistence
//! - Standalone app window management
//! - Service worker registration interface (stub)
//! - Install prompt handling
//!
//! # Example
//!
//! ```rust,ignore
//! use pwa_manager::{PwaManager, WebAppManifest};
//!
//! let manager = PwaManager::new();
//!
//! // Parse a manifest
//! let manifest_json = r#"{"name": "My App", "start_url": "/"}"#;
//! let manifest = WebAppManifest::from_json(manifest_json)?;
//!
//! // Install the PWA
//! let pwa = manager.install(manifest, "https://example.com").await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use url::Url;
use uuid::Uuid;

/// Unique identifier for an installed PWA
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PwaId(Uuid);

impl PwaId {
    /// Create a new random PWA ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a PWA ID from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for PwaId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PwaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// PWA-related errors
#[derive(Debug, Error)]
pub enum PwaError {
    /// Failed to parse manifest JSON
    #[error("Failed to parse manifest: {0}")]
    ManifestParseError(String),

    /// Invalid manifest data
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    /// PWA not found
    #[error("PWA not found: {0}")]
    NotFound(PwaId),

    /// PWA already installed
    #[error("PWA already installed from origin: {0}")]
    AlreadyInstalled(String),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Service worker error
    #[error("Service worker error: {0}")]
    ServiceWorkerError(String),

    /// Window error
    #[error("Window error: {0}")]
    WindowError(String),
}

/// Result type for PWA operations
pub type Result<T> = std::result::Result<T, PwaError>;

/// Display mode for PWA windows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PwaDisplayMode {
    /// Opens in a standalone window without browser UI
    #[default]
    Standalone,
    /// Opens in fullscreen mode
    Fullscreen,
    /// Opens with minimal browser UI
    MinimalUi,
    /// Opens in a regular browser tab
    Browser,
}

impl PwaDisplayMode {
    /// Parse display mode from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fullscreen" => Self::Fullscreen,
            "standalone" => Self::Standalone,
            "minimal-ui" | "minimal_ui" => Self::MinimalUi,
            "browser" => Self::Browser,
            _ => Self::default(),
        }
    }

    /// Check if this mode shows browser chrome
    pub fn shows_browser_chrome(&self) -> bool {
        matches!(self, Self::MinimalUi | Self::Browser)
    }

    /// Check if this mode is a standalone window
    pub fn is_standalone_window(&self) -> bool {
        matches!(self, Self::Standalone | Self::Fullscreen | Self::MinimalUi)
    }
}

/// Icon definition in a PWA manifest
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PwaIcon {
    /// URL of the icon (absolute or relative to manifest)
    pub src: String,

    /// Icon sizes (e.g., "192x192", "512x512")
    #[serde(default)]
    pub sizes: String,

    /// MIME type of the icon
    #[serde(rename = "type", default)]
    pub icon_type: String,

    /// Purpose of the icon (any, maskable, monochrome)
    #[serde(default)]
    pub purpose: String,
}

impl PwaIcon {
    /// Create a new PWA icon
    pub fn new(src: impl Into<String>, sizes: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            sizes: sizes.into(),
            icon_type: String::new(),
            purpose: String::new(),
        }
    }

    /// Set the icon type
    pub fn with_type(mut self, icon_type: impl Into<String>) -> Self {
        self.icon_type = icon_type.into();
        self
    }

    /// Set the icon purpose
    pub fn with_purpose(mut self, purpose: impl Into<String>) -> Self {
        self.purpose = purpose.into();
        self
    }

    /// Parse the sizes string into width and height pairs
    pub fn parse_sizes(&self) -> Vec<(u32, u32)> {
        self.sizes
            .split_whitespace()
            .filter_map(|size| {
                let lowercase = size.to_lowercase();
                let parts: Vec<&str> = lowercase.split('x').collect();
                if parts.len() == 2 {
                    let width = parts[0].parse().ok()?;
                    let height = parts[1].parse().ok()?;
                    Some((width, height))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the largest icon size
    pub fn largest_size(&self) -> Option<(u32, u32)> {
        self.parse_sizes()
            .into_iter()
            .max_by_key(|(w, h)| w * h)
    }

    /// Check if icon is maskable
    pub fn is_maskable(&self) -> bool {
        self.purpose.to_lowercase().contains("maskable")
    }
}

/// Shortcut definition for PWA
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PwaShortcut {
    /// Name of the shortcut
    pub name: String,

    /// Short name for limited space
    #[serde(default)]
    pub short_name: String,

    /// Description of the shortcut
    #[serde(default)]
    pub description: String,

    /// URL to open when shortcut is activated
    pub url: String,

    /// Icons for the shortcut
    #[serde(default)]
    pub icons: Vec<PwaIcon>,
}

impl PwaShortcut {
    /// Create a new PWA shortcut
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            short_name: String::new(),
            description: String::new(),
            url: url.into(),
            icons: Vec::new(),
        }
    }
}

/// Screenshot definition for PWA
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PwaScreenshot {
    /// URL of the screenshot
    pub src: String,

    /// Size of the screenshot
    #[serde(default)]
    pub sizes: String,

    /// MIME type
    #[serde(rename = "type", default)]
    pub screenshot_type: String,

    /// Form factor (wide, narrow)
    #[serde(default)]
    pub form_factor: String,

    /// Label for the screenshot
    #[serde(default)]
    pub label: String,
}

/// Web App Manifest structure
///
/// Represents the parsed content of a manifest.json file for PWA installation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebAppManifest {
    /// Full name of the application
    pub name: String,

    /// Short name for display in limited space
    #[serde(default)]
    pub short_name: String,

    /// Description of the application
    #[serde(default)]
    pub description: String,

    /// URL to open when the app is launched
    #[serde(default)]
    pub start_url: String,

    /// Scope of URLs that are part of the app
    #[serde(default)]
    pub scope: String,

    /// Display mode for the application
    #[serde(default)]
    pub display: PwaDisplayMode,

    /// Background color for the splash screen
    #[serde(default)]
    pub background_color: String,

    /// Theme color for browser UI
    #[serde(default)]
    pub theme_color: String,

    /// Text direction (ltr, rtl, auto)
    #[serde(default)]
    pub dir: String,

    /// Primary language
    #[serde(default)]
    pub lang: String,

    /// Preferred orientation
    #[serde(default)]
    pub orientation: String,

    /// Application icons
    #[serde(default)]
    pub icons: Vec<PwaIcon>,

    /// Application shortcuts
    #[serde(default)]
    pub shortcuts: Vec<PwaShortcut>,

    /// Screenshots for app store display
    #[serde(default)]
    pub screenshots: Vec<PwaScreenshot>,

    /// Categories for the application
    #[serde(default)]
    pub categories: Vec<String>,

    /// Related applications
    #[serde(default)]
    pub related_applications: Vec<RelatedApplication>,

    /// Whether to prefer related applications
    #[serde(default)]
    pub prefer_related_applications: bool,

    /// Unique identifier for the app
    #[serde(default)]
    pub id: String,
}

/// Related application reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedApplication {
    /// Platform (play, itunes, windows, webapp, etc.)
    pub platform: String,

    /// URL to the application
    #[serde(default)]
    pub url: String,

    /// Application ID on the platform
    #[serde(default)]
    pub id: String,
}

impl WebAppManifest {
    /// Create a new manifest with minimal required fields
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            short_name: String::new(),
            description: String::new(),
            start_url: "/".to_string(),
            scope: "/".to_string(),
            display: PwaDisplayMode::default(),
            background_color: String::new(),
            theme_color: String::new(),
            dir: String::new(),
            lang: String::new(),
            orientation: String::new(),
            icons: Vec::new(),
            shortcuts: Vec::new(),
            screenshots: Vec::new(),
            categories: Vec::new(),
            related_applications: Vec::new(),
            prefer_related_applications: false,
            id: String::new(),
        }
    }

    /// Parse manifest from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| PwaError::ManifestParseError(e.to_string()))
    }

    /// Serialize manifest to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PwaError::SerializationError(e.to_string()))
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(PwaError::InvalidManifest(
                "Manifest must have a name".to_string(),
            ));
        }
        Ok(())
    }

    /// Get the display name (prefer short_name if available)
    pub fn display_name(&self) -> &str {
        if self.short_name.is_empty() {
            &self.name
        } else {
            &self.short_name
        }
    }

    /// Get the best icon for a given size
    pub fn best_icon_for_size(&self, target_size: u32) -> Option<&PwaIcon> {
        self.icons
            .iter()
            .filter_map(|icon| {
                let sizes = icon.parse_sizes();
                let best_match = sizes
                    .iter()
                    .min_by_key(|(w, _)| (*w as i32 - target_size as i32).abs())
                    .copied();
                best_match.map(|size| (icon, size))
            })
            .min_by_key(|(_, (w, _))| (*w as i32 - target_size as i32).abs())
            .map(|(icon, _)| icon)
    }

    /// Get the largest available icon
    pub fn largest_icon(&self) -> Option<&PwaIcon> {
        self.icons
            .iter()
            .filter_map(|icon| icon.largest_size().map(|size| (icon, size)))
            .max_by_key(|(_, (w, h))| w * h)
            .map(|(icon, _)| icon)
    }

    /// Resolve a relative URL against a base URL
    pub fn resolve_url(&self, base_url: &Url, relative: &str) -> Result<Url> {
        base_url
            .join(relative)
            .map_err(|e| PwaError::InvalidUrl(e.to_string()))
    }
}

/// Installed PWA information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPwa {
    /// Unique identifier
    pub id: PwaId,

    /// The PWA manifest
    pub manifest: WebAppManifest,

    /// Origin URL where the PWA was installed from
    pub origin: String,

    /// Installation timestamp
    pub install_date: DateTime<Utc>,

    /// Last used timestamp
    pub last_used: DateTime<Utc>,

    /// Installation location on disk
    pub install_location: PathBuf,

    /// Whether the PWA is enabled
    pub enabled: bool,

    /// Launch count
    pub launch_count: u64,

    /// Custom user notes
    #[serde(default)]
    pub user_notes: String,
}

impl InstalledPwa {
    /// Create a new installed PWA
    pub fn new(manifest: WebAppManifest, origin: impl Into<String>, location: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            id: PwaId::new(),
            manifest,
            origin: origin.into(),
            install_date: now,
            last_used: now,
            install_location: location,
            enabled: true,
            launch_count: 0,
            user_notes: String::new(),
        }
    }

    /// Record a launch of this PWA
    pub fn record_launch(&mut self) {
        self.last_used = Utc::now();
        self.launch_count += 1;
    }

    /// Get the display name
    pub fn display_name(&self) -> &str {
        self.manifest.display_name()
    }

    /// Get the start URL resolved against the origin
    pub fn resolved_start_url(&self) -> Result<Url> {
        let base = Url::parse(&self.origin).map_err(|e| PwaError::InvalidUrl(e.to_string()))?;
        self.manifest.resolve_url(&base, &self.manifest.start_url)
    }
}

/// PWA installation prompt configuration
#[derive(Debug, Clone)]
pub struct PwaInstallPrompt {
    /// The manifest for the PWA to install
    pub manifest: WebAppManifest,

    /// Origin URL
    pub origin: String,

    /// Whether the prompt has been shown
    pub shown: bool,

    /// Whether the user has made a choice
    pub choice_made: bool,

    /// User's choice (true = install, false = dismiss)
    pub user_accepted: bool,
}

impl PwaInstallPrompt {
    /// Create a new install prompt
    pub fn new(manifest: WebAppManifest, origin: impl Into<String>) -> Self {
        Self {
            manifest,
            origin: origin.into(),
            shown: false,
            choice_made: false,
            user_accepted: false,
        }
    }

    /// Mark the prompt as shown
    pub fn mark_shown(&mut self) {
        self.shown = true;
    }

    /// Record user's choice
    pub fn record_choice(&mut self, accepted: bool) {
        self.choice_made = true;
        self.user_accepted = accepted;
    }

    /// Check if installation should proceed
    pub fn should_install(&self) -> bool {
        self.choice_made && self.user_accepted
    }
}

/// PWA window configuration
#[derive(Debug, Clone)]
pub struct PwaWindowConfig {
    /// Window title
    pub title: String,

    /// Window width
    pub width: u32,

    /// Window height
    pub height: u32,

    /// Minimum window width
    pub min_width: Option<u32>,

    /// Minimum window height
    pub min_height: Option<u32>,

    /// Whether window is resizable
    pub resizable: bool,

    /// Display mode
    pub display_mode: PwaDisplayMode,

    /// Theme color for window decorations
    pub theme_color: Option<String>,

    /// Background color
    pub background_color: Option<String>,
}

impl Default for PwaWindowConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            width: 1024,
            height: 768,
            min_width: Some(320),
            min_height: Some(240),
            resizable: true,
            display_mode: PwaDisplayMode::Standalone,
            theme_color: None,
            background_color: None,
        }
    }
}

impl PwaWindowConfig {
    /// Create configuration from installed PWA
    pub fn from_pwa(pwa: &InstalledPwa) -> Self {
        Self {
            title: pwa.display_name().to_string(),
            display_mode: pwa.manifest.display,
            theme_color: if pwa.manifest.theme_color.is_empty() {
                None
            } else {
                Some(pwa.manifest.theme_color.clone())
            },
            background_color: if pwa.manifest.background_color.is_empty() {
                None
            } else {
                Some(pwa.manifest.background_color.clone())
            },
            ..Default::default()
        }
    }
}

/// Represents a standalone PWA window
#[derive(Debug)]
pub struct PwaWindow {
    /// Window ID
    pub id: Uuid,

    /// Associated PWA
    pub pwa_id: PwaId,

    /// Window configuration
    pub config: PwaWindowConfig,

    /// Current URL being displayed
    pub current_url: String,

    /// Whether the window is visible
    pub visible: bool,

    /// Whether the window is focused
    pub focused: bool,
}

impl PwaWindow {
    /// Create a new PWA window
    pub fn new(pwa: &InstalledPwa) -> Result<Self> {
        let start_url = pwa.resolved_start_url()?;
        Ok(Self {
            id: Uuid::new_v4(),
            pwa_id: pwa.id,
            config: PwaWindowConfig::from_pwa(pwa),
            current_url: start_url.to_string(),
            visible: true,
            focused: true,
        })
    }

    /// Navigate to a URL within the PWA scope
    pub fn navigate(&mut self, url: impl Into<String>) {
        self.current_url = url.into();
    }

    /// Show the window
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the window
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Focus the window
    pub fn focus(&mut self) {
        self.focused = true;
    }

    /// Unfocus the window
    pub fn blur(&mut self) {
        self.focused = false;
    }

    /// Check if URL is within PWA scope
    pub fn is_in_scope(&self, url: &str, scope: &str) -> bool {
        url.starts_with(scope)
    }
}

/// Service worker registration state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceWorkerState {
    /// Not registered
    Unregistered,
    /// Registration in progress
    Installing,
    /// Installed and waiting
    Waiting,
    /// Active and controlling
    Active,
    /// Failed to register
    Failed,
}

/// Service worker registration (stub interface)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceWorkerRegistration {
    /// Script URL of the service worker
    pub script_url: String,

    /// Scope of the service worker
    pub scope: String,

    /// Current state
    pub state: ServiceWorkerState,

    /// Registration timestamp
    pub registered_at: Option<DateTime<Utc>>,
}

impl ServiceWorkerRegistration {
    /// Create a new unregistered service worker
    pub fn new(script_url: impl Into<String>, scope: impl Into<String>) -> Self {
        Self {
            script_url: script_url.into(),
            scope: scope.into(),
            state: ServiceWorkerState::Unregistered,
            registered_at: None,
        }
    }

    /// Mark as installing
    pub fn mark_installing(&mut self) {
        self.state = ServiceWorkerState::Installing;
    }

    /// Mark as active
    pub fn mark_active(&mut self) {
        self.state = ServiceWorkerState::Active;
        self.registered_at = Some(Utc::now());
    }

    /// Mark as failed
    pub fn mark_failed(&mut self) {
        self.state = ServiceWorkerState::Failed;
    }

    /// Check if active
    pub fn is_active(&self) -> bool {
        self.state == ServiceWorkerState::Active
    }
}

/// PWA Manager
///
/// Central manager for PWA installation, lifecycle, and window management.
pub struct PwaManager {
    /// Installed PWAs indexed by ID
    installed: Arc<RwLock<HashMap<PwaId, InstalledPwa>>>,

    /// Service worker registrations by origin
    service_workers: Arc<RwLock<HashMap<String, ServiceWorkerRegistration>>>,

    /// Active PWA windows
    windows: Arc<RwLock<HashMap<Uuid, PwaWindow>>>,

    /// Installation directory
    install_dir: PathBuf,
}

impl PwaManager {
    /// Create a new PWA manager
    pub fn new() -> Self {
        let install_dir = directories::ProjectDirs::from("com", "CortenBrowser", "PWAs")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("./pwas"));

        Self {
            installed: Arc::new(RwLock::new(HashMap::new())),
            service_workers: Arc::new(RwLock::new(HashMap::new())),
            windows: Arc::new(RwLock::new(HashMap::new())),
            install_dir,
        }
    }

    /// Create a PWA manager with a custom install directory
    pub fn with_install_dir(install_dir: PathBuf) -> Self {
        Self {
            installed: Arc::new(RwLock::new(HashMap::new())),
            service_workers: Arc::new(RwLock::new(HashMap::new())),
            windows: Arc::new(RwLock::new(HashMap::new())),
            install_dir,
        }
    }

    /// Get the installation directory
    pub fn install_dir(&self) -> &PathBuf {
        &self.install_dir
    }

    /// Install a PWA from a manifest
    pub async fn install(&self, manifest: WebAppManifest, origin: &str) -> Result<InstalledPwa> {
        // Validate manifest
        manifest.validate()?;

        // Check if already installed from this origin
        let installed = self.installed.read().await;
        for pwa in installed.values() {
            if pwa.origin == origin {
                return Err(PwaError::AlreadyInstalled(origin.to_string()));
            }
        }
        drop(installed);

        // Create installation location
        let pwa_id = PwaId::new();
        let install_location = self.install_dir.join(pwa_id.to_string());

        // Create the PWA record
        let pwa = InstalledPwa::new(manifest, origin, install_location);
        let id = pwa.id;

        // Store the installation
        let mut installed = self.installed.write().await;
        installed.insert(id, pwa.clone());

        Ok(pwa)
    }

    /// Uninstall a PWA
    pub async fn uninstall(&self, id: PwaId) -> Result<()> {
        let mut installed = self.installed.write().await;
        installed.remove(&id).ok_or(PwaError::NotFound(id))?;

        // Close any windows for this PWA
        let mut windows = self.windows.write().await;
        windows.retain(|_, window| window.pwa_id != id);

        Ok(())
    }

    /// Get an installed PWA by ID
    pub async fn get(&self, id: PwaId) -> Option<InstalledPwa> {
        let installed = self.installed.read().await;
        installed.get(&id).cloned()
    }

    /// Get all installed PWAs
    pub async fn list_installed(&self) -> Vec<InstalledPwa> {
        let installed = self.installed.read().await;
        installed.values().cloned().collect()
    }

    /// Find a PWA by origin
    pub async fn find_by_origin(&self, origin: &str) -> Option<InstalledPwa> {
        let installed = self.installed.read().await;
        installed.values().find(|pwa| pwa.origin == origin).cloned()
    }

    /// Check if a PWA is installed from an origin
    pub async fn is_installed(&self, origin: &str) -> bool {
        self.find_by_origin(origin).await.is_some()
    }

    /// Launch a PWA and create a window
    pub async fn launch(&self, id: PwaId) -> Result<Uuid> {
        // Get and update the PWA
        let mut installed = self.installed.write().await;
        let pwa = installed.get_mut(&id).ok_or(PwaError::NotFound(id))?;
        pwa.record_launch();
        let pwa_clone = pwa.clone();
        drop(installed);

        // Create the window
        let window = PwaWindow::new(&pwa_clone)?;
        let window_id = window.id;

        let mut windows = self.windows.write().await;
        windows.insert(window_id, window);

        Ok(window_id)
    }

    /// Close a PWA window
    pub async fn close_window(&self, window_id: Uuid) -> Result<()> {
        let mut windows = self.windows.write().await;
        windows
            .remove(&window_id)
            .ok_or_else(|| PwaError::WindowError("Window not found".to_string()))?;
        Ok(())
    }

    /// Get a PWA window
    pub async fn get_window(&self, window_id: Uuid) -> Option<PwaWindow> {
        let windows = self.windows.read().await;
        windows.get(&window_id).cloned()
    }

    /// Get all windows for a PWA
    pub async fn get_pwa_windows(&self, pwa_id: PwaId) -> Vec<Uuid> {
        let windows = self.windows.read().await;
        windows
            .iter()
            .filter(|(_, w)| w.pwa_id == pwa_id)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Register a service worker
    pub async fn register_service_worker(
        &self,
        origin: &str,
        script_url: &str,
        scope: &str,
    ) -> Result<()> {
        let mut sw = ServiceWorkerRegistration::new(script_url, scope);
        sw.mark_installing();

        // In a real implementation, this would actually register the service worker
        // For now, we just store the registration
        sw.mark_active();

        let mut service_workers = self.service_workers.write().await;
        service_workers.insert(origin.to_string(), sw);

        Ok(())
    }

    /// Get service worker registration for an origin
    pub async fn get_service_worker(&self, origin: &str) -> Option<ServiceWorkerRegistration> {
        let service_workers = self.service_workers.read().await;
        service_workers.get(origin).cloned()
    }

    /// Unregister a service worker
    pub async fn unregister_service_worker(&self, origin: &str) -> Result<()> {
        let mut service_workers = self.service_workers.write().await;
        service_workers
            .remove(origin)
            .ok_or_else(|| PwaError::ServiceWorkerError("Not registered".to_string()))?;
        Ok(())
    }

    /// Update the last used timestamp for a PWA
    pub async fn update_last_used(&self, id: PwaId) -> Result<()> {
        let mut installed = self.installed.write().await;
        let pwa = installed.get_mut(&id).ok_or(PwaError::NotFound(id))?;
        pwa.last_used = Utc::now();
        Ok(())
    }

    /// Enable a PWA
    pub async fn enable(&self, id: PwaId) -> Result<()> {
        let mut installed = self.installed.write().await;
        let pwa = installed.get_mut(&id).ok_or(PwaError::NotFound(id))?;
        pwa.enabled = true;
        Ok(())
    }

    /// Disable a PWA
    pub async fn disable(&self, id: PwaId) -> Result<()> {
        let mut installed = self.installed.write().await;
        let pwa = installed.get_mut(&id).ok_or(PwaError::NotFound(id))?;
        pwa.enabled = false;
        Ok(())
    }

    /// Get count of installed PWAs
    pub async fn installed_count(&self) -> usize {
        let installed = self.installed.read().await;
        installed.len()
    }

    /// Create an install prompt for a manifest
    pub fn create_install_prompt(
        &self,
        manifest: WebAppManifest,
        origin: &str,
    ) -> PwaInstallPrompt {
        PwaInstallPrompt::new(manifest, origin)
    }
}

impl Default for PwaManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PwaWindow {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            pwa_id: self.pwa_id,
            config: self.config.clone(),
            current_url: self.current_url.clone(),
            visible: self.visible,
            focused: self.focused,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================
    // WebAppManifest Tests
    // =====================

    #[test]
    fn test_manifest_new() {
        let manifest = WebAppManifest::new("My App");
        assert_eq!(manifest.name, "My App");
        assert_eq!(manifest.start_url, "/");
        assert_eq!(manifest.display, PwaDisplayMode::Standalone);
    }

    #[test]
    fn test_manifest_from_json_minimal() {
        let json = r#"{"name": "Test App"}"#;
        let manifest = WebAppManifest::from_json(json).unwrap();
        assert_eq!(manifest.name, "Test App");
    }

    #[test]
    fn test_manifest_from_json_full() {
        let json = r##"{
            "name": "Full Test App",
            "short_name": "Test",
            "description": "A test application",
            "start_url": "/app",
            "scope": "/",
            "display": "standalone",
            "background_color": "#ffffff",
            "theme_color": "#007bff",
            "icons": [
                {"src": "/icon-192.png", "sizes": "192x192", "type": "image/png"},
                {"src": "/icon-512.png", "sizes": "512x512", "type": "image/png"}
            ]
        }"##;

        let manifest = WebAppManifest::from_json(json).unwrap();
        assert_eq!(manifest.name, "Full Test App");
        assert_eq!(manifest.short_name, "Test");
        assert_eq!(manifest.start_url, "/app");
        assert_eq!(manifest.theme_color, "#007bff");
        assert_eq!(manifest.icons.len(), 2);
    }

    #[test]
    fn test_manifest_from_json_invalid() {
        let json = r#"{"invalid json"#;
        assert!(WebAppManifest::from_json(json).is_err());
    }

    #[test]
    fn test_manifest_validate_empty_name() {
        let manifest = WebAppManifest::new("");
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_manifest_validate_whitespace_name() {
        let manifest = WebAppManifest::new("   ");
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_manifest_display_name_prefers_short_name() {
        let mut manifest = WebAppManifest::new("Full Name");
        manifest.short_name = "Short".to_string();
        assert_eq!(manifest.display_name(), "Short");
    }

    #[test]
    fn test_manifest_display_name_fallback() {
        let manifest = WebAppManifest::new("Full Name");
        assert_eq!(manifest.display_name(), "Full Name");
    }

    #[test]
    fn test_manifest_to_json() {
        let manifest = WebAppManifest::new("Test");
        let json = manifest.to_json().unwrap();
        assert!(json.contains("\"name\": \"Test\""));
    }

    #[test]
    fn test_manifest_best_icon_for_size() {
        let mut manifest = WebAppManifest::new("Test");
        manifest.icons = vec![
            PwaIcon::new("/icon-48.png", "48x48"),
            PwaIcon::new("/icon-192.png", "192x192"),
            PwaIcon::new("/icon-512.png", "512x512"),
        ];

        let icon = manifest.best_icon_for_size(200);
        assert!(icon.is_some());
        assert_eq!(icon.unwrap().src, "/icon-192.png");
    }

    #[test]
    fn test_manifest_largest_icon() {
        let mut manifest = WebAppManifest::new("Test");
        manifest.icons = vec![
            PwaIcon::new("/icon-48.png", "48x48"),
            PwaIcon::new("/icon-512.png", "512x512"),
        ];

        let icon = manifest.largest_icon();
        assert!(icon.is_some());
        assert_eq!(icon.unwrap().src, "/icon-512.png");
    }

    // =====================
    // PwaDisplayMode Tests
    // =====================

    #[test]
    fn test_display_mode_from_str() {
        assert_eq!(PwaDisplayMode::from_str("standalone"), PwaDisplayMode::Standalone);
        assert_eq!(PwaDisplayMode::from_str("fullscreen"), PwaDisplayMode::Fullscreen);
        assert_eq!(PwaDisplayMode::from_str("minimal-ui"), PwaDisplayMode::MinimalUi);
        assert_eq!(PwaDisplayMode::from_str("browser"), PwaDisplayMode::Browser);
        assert_eq!(PwaDisplayMode::from_str("unknown"), PwaDisplayMode::Standalone);
    }

    #[test]
    fn test_display_mode_shows_browser_chrome() {
        assert!(!PwaDisplayMode::Standalone.shows_browser_chrome());
        assert!(!PwaDisplayMode::Fullscreen.shows_browser_chrome());
        assert!(PwaDisplayMode::MinimalUi.shows_browser_chrome());
        assert!(PwaDisplayMode::Browser.shows_browser_chrome());
    }

    #[test]
    fn test_display_mode_is_standalone_window() {
        assert!(PwaDisplayMode::Standalone.is_standalone_window());
        assert!(PwaDisplayMode::Fullscreen.is_standalone_window());
        assert!(PwaDisplayMode::MinimalUi.is_standalone_window());
        assert!(!PwaDisplayMode::Browser.is_standalone_window());
    }

    // =====================
    // PwaIcon Tests
    // =====================

    #[test]
    fn test_icon_new() {
        let icon = PwaIcon::new("/icon.png", "192x192");
        assert_eq!(icon.src, "/icon.png");
        assert_eq!(icon.sizes, "192x192");
    }

    #[test]
    fn test_icon_parse_sizes() {
        let icon = PwaIcon::new("/icon.png", "48x48 192x192 512x512");
        let sizes = icon.parse_sizes();
        assert_eq!(sizes.len(), 3);
        assert!(sizes.contains(&(48, 48)));
        assert!(sizes.contains(&(192, 192)));
        assert!(sizes.contains(&(512, 512)));
    }

    #[test]
    fn test_icon_largest_size() {
        let icon = PwaIcon::new("/icon.png", "48x48 192x192 512x512");
        assert_eq!(icon.largest_size(), Some((512, 512)));
    }

    #[test]
    fn test_icon_is_maskable() {
        let icon = PwaIcon::new("/icon.png", "192x192").with_purpose("maskable");
        assert!(icon.is_maskable());

        let icon2 = PwaIcon::new("/icon.png", "192x192").with_purpose("any");
        assert!(!icon2.is_maskable());
    }

    // =====================
    // PwaInstallPrompt Tests
    // =====================

    #[test]
    fn test_install_prompt_new() {
        let manifest = WebAppManifest::new("Test");
        let prompt = PwaInstallPrompt::new(manifest, "https://example.com");
        assert!(!prompt.shown);
        assert!(!prompt.choice_made);
    }

    #[test]
    fn test_install_prompt_mark_shown() {
        let manifest = WebAppManifest::new("Test");
        let mut prompt = PwaInstallPrompt::new(manifest, "https://example.com");
        prompt.mark_shown();
        assert!(prompt.shown);
    }

    #[test]
    fn test_install_prompt_record_choice_accepted() {
        let manifest = WebAppManifest::new("Test");
        let mut prompt = PwaInstallPrompt::new(manifest, "https://example.com");
        prompt.record_choice(true);
        assert!(prompt.choice_made);
        assert!(prompt.user_accepted);
        assert!(prompt.should_install());
    }

    #[test]
    fn test_install_prompt_record_choice_rejected() {
        let manifest = WebAppManifest::new("Test");
        let mut prompt = PwaInstallPrompt::new(manifest, "https://example.com");
        prompt.record_choice(false);
        assert!(prompt.choice_made);
        assert!(!prompt.user_accepted);
        assert!(!prompt.should_install());
    }

    // =====================
    // PwaManager Tests
    // =====================

    #[tokio::test]
    async fn test_manager_new() {
        let manager = PwaManager::new();
        assert_eq!(manager.installed_count().await, 0);
    }

    #[tokio::test]
    async fn test_manager_install() {
        let manager = PwaManager::with_install_dir(PathBuf::from("/tmp/pwa_test"));
        let manifest = WebAppManifest::new("Test App");

        let pwa = manager.install(manifest, "https://example.com").await.unwrap();
        assert_eq!(pwa.manifest.name, "Test App");
        assert_eq!(pwa.origin, "https://example.com");
        assert!(pwa.enabled);
        assert_eq!(pwa.launch_count, 0);
    }

    #[tokio::test]
    async fn test_manager_install_duplicate() {
        let manager = PwaManager::with_install_dir(PathBuf::from("/tmp/pwa_test"));
        let manifest = WebAppManifest::new("Test App");

        manager.install(manifest.clone(), "https://example.com").await.unwrap();
        let result = manager.install(manifest, "https://example.com").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_manager_uninstall() {
        let manager = PwaManager::with_install_dir(PathBuf::from("/tmp/pwa_test"));
        let manifest = WebAppManifest::new("Test App");

        let pwa = manager.install(manifest, "https://example.com").await.unwrap();
        let id = pwa.id;

        manager.uninstall(id).await.unwrap();
        assert!(manager.get(id).await.is_none());
    }

    #[tokio::test]
    async fn test_manager_uninstall_not_found() {
        let manager = PwaManager::with_install_dir(PathBuf::from("/tmp/pwa_test"));
        let result = manager.uninstall(PwaId::new()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_manager_list_installed() {
        let manager = PwaManager::with_install_dir(PathBuf::from("/tmp/pwa_test"));

        manager.install(WebAppManifest::new("App 1"), "https://app1.com").await.unwrap();
        manager.install(WebAppManifest::new("App 2"), "https://app2.com").await.unwrap();

        let installed = manager.list_installed().await;
        assert_eq!(installed.len(), 2);
    }

    #[tokio::test]
    async fn test_manager_find_by_origin() {
        let manager = PwaManager::with_install_dir(PathBuf::from("/tmp/pwa_test"));
        manager.install(WebAppManifest::new("Test"), "https://example.com").await.unwrap();

        let found = manager.find_by_origin("https://example.com").await;
        assert!(found.is_some());

        let not_found = manager.find_by_origin("https://other.com").await;
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_manager_is_installed() {
        let manager = PwaManager::with_install_dir(PathBuf::from("/tmp/pwa_test"));
        manager.install(WebAppManifest::new("Test"), "https://example.com").await.unwrap();

        assert!(manager.is_installed("https://example.com").await);
        assert!(!manager.is_installed("https://other.com").await);
    }

    #[tokio::test]
    async fn test_manager_launch() {
        let manager = PwaManager::with_install_dir(PathBuf::from("/tmp/pwa_test"));
        let manifest = WebAppManifest::new("Test App");

        let pwa = manager.install(manifest, "https://example.com").await.unwrap();
        let window_id = manager.launch(pwa.id).await.unwrap();

        // Verify window was created
        let window = manager.get_window(window_id).await;
        assert!(window.is_some());

        // Verify launch count increased
        let updated_pwa = manager.get(pwa.id).await.unwrap();
        assert_eq!(updated_pwa.launch_count, 1);
    }

    #[tokio::test]
    async fn test_manager_close_window() {
        let manager = PwaManager::with_install_dir(PathBuf::from("/tmp/pwa_test"));
        let manifest = WebAppManifest::new("Test App");

        let pwa = manager.install(manifest, "https://example.com").await.unwrap();
        let window_id = manager.launch(pwa.id).await.unwrap();

        manager.close_window(window_id).await.unwrap();
        assert!(manager.get_window(window_id).await.is_none());
    }

    #[tokio::test]
    async fn test_manager_enable_disable() {
        let manager = PwaManager::with_install_dir(PathBuf::from("/tmp/pwa_test"));
        let manifest = WebAppManifest::new("Test App");

        let pwa = manager.install(manifest, "https://example.com").await.unwrap();

        manager.disable(pwa.id).await.unwrap();
        let disabled = manager.get(pwa.id).await.unwrap();
        assert!(!disabled.enabled);

        manager.enable(pwa.id).await.unwrap();
        let enabled = manager.get(pwa.id).await.unwrap();
        assert!(enabled.enabled);
    }

    // =====================
    // ServiceWorker Tests
    // =====================

    #[tokio::test]
    async fn test_service_worker_registration() {
        let manager = PwaManager::with_install_dir(PathBuf::from("/tmp/pwa_test"));

        manager.register_service_worker(
            "https://example.com",
            "/sw.js",
            "/"
        ).await.unwrap();

        let sw = manager.get_service_worker("https://example.com").await;
        assert!(sw.is_some());
        assert!(sw.unwrap().is_active());
    }

    #[tokio::test]
    async fn test_service_worker_unregister() {
        let manager = PwaManager::with_install_dir(PathBuf::from("/tmp/pwa_test"));

        manager.register_service_worker(
            "https://example.com",
            "/sw.js",
            "/"
        ).await.unwrap();

        manager.unregister_service_worker("https://example.com").await.unwrap();
        assert!(manager.get_service_worker("https://example.com").await.is_none());
    }

    // =====================
    // PwaWindow Tests
    // =====================

    #[test]
    fn test_pwa_window_navigate() {
        let manifest = WebAppManifest::new("Test");
        let pwa = InstalledPwa::new(manifest, "https://example.com", PathBuf::from("/tmp"));
        let mut window = PwaWindow::new(&pwa).unwrap();

        window.navigate("https://example.com/page2");
        assert_eq!(window.current_url, "https://example.com/page2");
    }

    #[test]
    fn test_pwa_window_visibility() {
        let manifest = WebAppManifest::new("Test");
        let pwa = InstalledPwa::new(manifest, "https://example.com", PathBuf::from("/tmp"));
        let mut window = PwaWindow::new(&pwa).unwrap();

        assert!(window.visible);
        window.hide();
        assert!(!window.visible);
        window.show();
        assert!(window.visible);
    }

    #[test]
    fn test_pwa_window_focus() {
        let manifest = WebAppManifest::new("Test");
        let pwa = InstalledPwa::new(manifest, "https://example.com", PathBuf::from("/tmp"));
        let mut window = PwaWindow::new(&pwa).unwrap();

        assert!(window.focused);
        window.blur();
        assert!(!window.focused);
        window.focus();
        assert!(window.focused);
    }

    #[test]
    fn test_pwa_window_scope_check() {
        let manifest = WebAppManifest::new("Test");
        let pwa = InstalledPwa::new(manifest, "https://example.com", PathBuf::from("/tmp"));
        let window = PwaWindow::new(&pwa).unwrap();

        assert!(window.is_in_scope("https://example.com/app/page", "https://example.com/app/"));
        assert!(!window.is_in_scope("https://example.com/other", "https://example.com/app/"));
    }

    // =====================
    // InstalledPwa Tests
    // =====================

    #[test]
    fn test_installed_pwa_record_launch() {
        let manifest = WebAppManifest::new("Test");
        let mut pwa = InstalledPwa::new(manifest, "https://example.com", PathBuf::from("/tmp"));

        assert_eq!(pwa.launch_count, 0);
        pwa.record_launch();
        assert_eq!(pwa.launch_count, 1);
        pwa.record_launch();
        assert_eq!(pwa.launch_count, 2);
    }

    #[test]
    fn test_installed_pwa_resolved_start_url() {
        let mut manifest = WebAppManifest::new("Test");
        manifest.start_url = "/app".to_string();
        let pwa = InstalledPwa::new(manifest, "https://example.com", PathBuf::from("/tmp"));

        let url = pwa.resolved_start_url().unwrap();
        assert_eq!(url.as_str(), "https://example.com/app");
    }

    // =====================
    // PwaShortcut Tests
    // =====================

    #[test]
    fn test_shortcut_new() {
        let shortcut = PwaShortcut::new("Open Editor", "/editor");
        assert_eq!(shortcut.name, "Open Editor");
        assert_eq!(shortcut.url, "/editor");
    }

    // =====================
    // PwaId Tests
    // =====================

    #[test]
    fn test_pwa_id_display() {
        let id = PwaId::new();
        let display = format!("{}", id);
        assert!(!display.is_empty());
    }

    #[test]
    fn test_pwa_id_equality() {
        let id1 = PwaId::new();
        let id2 = PwaId::new();
        assert_ne!(id1, id2);

        let id3 = id1;
        assert_eq!(id1, id3);
    }
}
