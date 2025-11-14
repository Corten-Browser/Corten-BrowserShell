//! BrowserApp - GUI application wrapper for browser shell and UI chrome

use crate::{init_logging, AppConfig};
use browser_shell::{BrowserShell, ShellConfig};
use shared_types::{ComponentError, WindowConfig};
use std::sync::Arc;
use tokio::sync::RwLock;
use ui_chrome::UiChrome;

/// Main browser application that integrates browser_shell and ui_chrome with eframe
pub struct BrowserApp {
    /// Browser shell instance
    ///
    /// Note: Currently unused but kept for future UI-to-shell interaction.
    /// Will be accessed when implementing features like navigation, page control, etc.
    #[allow(dead_code)]
    browser_shell: Arc<RwLock<BrowserShell>>,

    /// UI chrome instance
    ui_chrome: Arc<RwLock<UiChrome>>,

    /// Application configuration
    config: AppConfig,
}

impl BrowserApp {
    /// Create a new BrowserApp instance
    ///
    /// Initializes the browser shell and UI chrome.
    ///
    /// # Arguments
    /// * `config` - Application configuration
    ///
    /// # Returns
    /// * `Result<Self, ComponentError>` - New BrowserApp instance or error
    pub async fn new(config: AppConfig) -> Result<Self, ComponentError> {
        // Initialize logging
        init_logging(&config.log_level);

        tracing::info!("Initializing BrowserApp...");

        // Create browser shell
        let mut browser_shell = BrowserShell::new();

        // Create shell configuration from app config
        let user_data_dir = config.user_data_dir.clone().unwrap_or_else(|| {
            // Default user data directory
            dirs::data_local_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap())
                .join("corten-browser")
                .to_str()
                .unwrap()
                .to_string()
        });

        let window_config = WindowConfig {
            title: "CortenBrowser".to_string(),
            width: if config.fullscreen { 1920 } else { 1280 },
            height: if config.fullscreen { 1080 } else { 720 },
            x: None,
            y: None,
            fullscreen: config.fullscreen,
            resizable: true,
            decorations: true,
            always_on_top: false,
            skip_taskbar: false,
        };

        let shell_config = ShellConfig {
            window_config,
            enable_devtools: config.enable_devtools,
            enable_extensions: false,
            user_data_dir,
        };

        // Initialize browser shell (skip in headless mode for testing)
        if !config.headless {
            browser_shell.initialize(shell_config).await?;
        } else {
            tracing::info!("Skipping browser shell initialization in headless mode");
        }

        // Create UI chrome
        let ui_chrome = UiChrome::new();

        // If initial URL provided, set it in address bar
        if let Some(ref url) = config.initial_url {
            tracing::info!("Initial URL: {}", url);
            // Note: ui_chrome doesn't have a public method to set address bar text
            // This will be handled when we navigate
        }

        Ok(Self {
            browser_shell: Arc::new(RwLock::new(browser_shell)),
            ui_chrome: Arc::new(RwLock::new(ui_chrome)),
            config,
        })
    }

    /// Launch the GUI application
    ///
    /// # Arguments
    /// * `self` - BrowserApp instance
    ///
    /// # Returns
    /// * `Result<(), ComponentError>` - Success or error
    pub fn launch(self) -> Result<(), ComponentError> {
        if self.config.headless {
            tracing::info!("Running in headless mode - GUI not launched");
            return Ok(());
        }

        tracing::info!("Launching GUI with eframe...");

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([
                    if self.config.fullscreen {
                        1920.0
                    } else {
                        1280.0
                    },
                    if self.config.fullscreen {
                        1080.0
                    } else {
                        720.0
                    },
                ])
                .with_fullscreen(self.config.fullscreen),
            ..Default::default()
        };

        eframe::run_native(
            "CortenBrowser",
            native_options,
            Box::new(|_cc| Ok(Box::new(self) as Box<dyn eframe::App>)),
        )
        .map_err(|e| ComponentError::InitializationFailed(e.to_string()))?;

        Ok(())
    }
}

impl eframe::App for BrowserApp {
    /// Update function called by eframe on each frame
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Delegate rendering to ui_chrome
        let mut ui_chrome = self.ui_chrome.blocking_write();

        if let Err(e) = ui_chrome.render(ctx) {
            tracing::error!("UI rendering error: {}", e);
        }
    }
}

// Helper function to add dirs dependency
// This will be added to Cargo.toml
