//! Shell App - Application entry point and CLI for the browser shell
//!
//! This component provides the main entry point for the CortenBrowser application,
//! handling command-line argument parsing and initialization of the browser shell.

mod app;

pub use app::BrowserApp;

use clap::Parser;
use shared_types::ComponentError;
use std::fmt;
use std::str::FromStr;

/// Log level for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
#[allow(missing_docs)]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Error => write!(f, "error"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
        }
    }
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    /// User data directory path
    pub user_data_dir: Option<String>,

    /// Initial URL to open on startup
    pub initial_url: Option<String>,

    /// Start in fullscreen mode
    pub fullscreen: bool,

    /// Run without UI (for testing)
    pub headless: bool,

    /// Enable developer tools
    pub enable_devtools: bool,

    /// Logging level
    pub log_level: LogLevel,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            user_data_dir: None,
            initial_url: None,
            fullscreen: false,
            headless: false,
            enable_devtools: false,
            log_level: LogLevel::Info,
        }
    }
}

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(name = "corten-browser")]
#[command(about = "CortenBrowser - A Rust-based web browser", long_about = None)]
struct Cli {
    /// User data directory
    #[arg(long, value_name = "PATH")]
    user_data_dir: Option<String>,

    /// Initial URL to open
    #[arg(long, value_name = "URL")]
    initial_url: Option<String>,

    /// Start in fullscreen mode
    #[arg(long)]
    fullscreen: bool,

    /// Run without UI (for testing)
    #[arg(long)]
    headless: bool,

    /// Enable developer tools
    #[arg(long)]
    enable_devtools: bool,

    /// Set logging level
    #[arg(long, value_enum, default_value = "info")]
    log_level: LogLevel,
}

/// Main shell application struct
pub struct ShellApp;

impl ShellApp {
    /// Parse command-line arguments
    ///
    /// # Arguments
    /// * `args` - Vector of argument strings
    ///
    /// # Returns
    /// * `Result<AppConfig, ComponentError>` - Parsed configuration or error
    ///
    /// # Example
    /// ```
    /// use shell_app::ShellApp;
    ///
    /// let args = vec!["browser".to_string(), "--fullscreen".to_string()];
    /// let config = ShellApp::parse_args(args).unwrap();
    /// assert_eq!(config.fullscreen, true);
    /// ```
    pub fn parse_args(args: Vec<String>) -> Result<AppConfig, ComponentError> {
        let cli =
            Cli::try_parse_from(args).map_err(|e| ComponentError::InvalidState(e.to_string()))?;

        Ok(AppConfig {
            user_data_dir: cli.user_data_dir,
            initial_url: cli.initial_url,
            fullscreen: cli.fullscreen,
            headless: cli.headless,
            enable_devtools: cli.enable_devtools,
            log_level: cli.log_level,
        })
    }

    /// Main application entry point
    ///
    /// # Arguments
    /// * `args` - Vector of command-line arguments
    ///
    /// # Returns
    /// * `Result<(), ComponentError>` - Success or error
    pub async fn main(args: Vec<String>) -> Result<(), ComponentError> {
        // Parse arguments
        let config = Self::parse_args(args)?;

        tracing::debug!("Configuration: {:?}", config);

        // Create BrowserApp instance
        let app = BrowserApp::new(config).await?;

        // Launch GUI (or run headless)
        app.launch()?;

        Ok(())
    }
}

/// Initialize logging based on log level
fn init_logging(level: &LogLevel) {
    use tracing_subscriber::EnvFilter;

    let filter_level = match level {
        LogLevel::Error => "error",
        LogLevel::Warn => "warn",
        LogLevel::Info => "info",
        LogLevel::Debug => "debug",
        LogLevel::Trace => "trace",
    };

    // Use try_init to avoid panic when called multiple times (e.g. in tests)
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter_level))
        .try_init();
}

/// Run the application
///
/// This is the main entry point called from main.rs
///
/// # Arguments
/// * `args` - Vector of command-line arguments
///
/// # Returns
/// * `Result<(), ComponentError>` - Success or error
pub async fn run(args: Vec<String>) -> Result<(), ComponentError> {
    ShellApp::main(args).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("error").unwrap(), LogLevel::Error);
        assert_eq!(LogLevel::from_str("ERROR").unwrap(), LogLevel::Error);
        assert!(LogLevel::from_str("invalid").is_err());
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Error.to_string(), "error");
        assert_eq!(LogLevel::Info.to_string(), "info");
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.log_level, LogLevel::Info);
        assert_eq!(config.fullscreen, false);
    }
}
