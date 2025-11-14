//! Unit tests for shell_app types (LogLevel and AppConfig)
//!
//! These tests follow TDD - written FIRST to define expected behavior.

use shell_app::{AppConfig, LogLevel};
use std::str::FromStr;

#[cfg(test)]
mod log_level_tests {
    use super::*;

    #[test]
    fn test_log_level_from_str_error() {
        // Given a valid log level string
        // When converted from string
        // Then the correct LogLevel variant is returned
        assert_eq!(LogLevel::from_str("error").unwrap(), LogLevel::Error);
    }

    #[test]
    fn test_log_level_from_str_warn() {
        assert_eq!(LogLevel::from_str("warn").unwrap(), LogLevel::Warn);
    }

    #[test]
    fn test_log_level_from_str_info() {
        assert_eq!(LogLevel::from_str("info").unwrap(), LogLevel::Info);
    }

    #[test]
    fn test_log_level_from_str_debug() {
        assert_eq!(LogLevel::from_str("debug").unwrap(), LogLevel::Debug);
    }

    #[test]
    fn test_log_level_from_str_trace() {
        assert_eq!(LogLevel::from_str("trace").unwrap(), LogLevel::Trace);
    }

    #[test]
    fn test_log_level_from_str_case_insensitive() {
        // Given a log level string with mixed case
        // When converted from string
        // Then it should be case-insensitive
        assert_eq!(LogLevel::from_str("ERROR").unwrap(), LogLevel::Error);
        assert_eq!(LogLevel::from_str("WaRn").unwrap(), LogLevel::Warn);
    }

    #[test]
    #[should_panic]
    fn test_log_level_from_str_invalid() {
        // Given an invalid log level string
        // When converted from string
        // Then an error should be returned
        LogLevel::from_str("invalid").unwrap();
    }

    #[test]
    fn test_log_level_default() {
        // Given no log level specified
        // When using default
        // Then Info level should be used
        assert_eq!(LogLevel::default(), LogLevel::Info);
    }

    #[test]
    fn test_log_level_display() {
        // Given a LogLevel variant
        // When converted to string
        // Then the correct lowercase string is returned
        assert_eq!(LogLevel::Error.to_string(), "error");
        assert_eq!(LogLevel::Warn.to_string(), "warn");
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Trace.to_string(), "trace");
    }
}

#[cfg(test)]
mod app_config_tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        // Given no configuration specified
        // When using default
        // Then all fields should have sensible defaults
        let config = AppConfig::default();

        assert_eq!(config.user_data_dir, None);
        assert_eq!(config.initial_url, None);
        assert_eq!(config.fullscreen, false);
        assert_eq!(config.headless, false);
        assert_eq!(config.enable_devtools, false);
        assert_eq!(config.log_level, LogLevel::Info);
    }

    #[test]
    fn test_app_config_with_user_data_dir() {
        // Given a user data directory is specified
        // When creating AppConfig
        // Then it should be stored correctly
        let config = AppConfig {
            user_data_dir: Some("/path/to/data".to_string()),
            ..Default::default()
        };

        assert_eq!(config.user_data_dir, Some("/path/to/data".to_string()));
    }

    #[test]
    fn test_app_config_with_initial_url() {
        // Given an initial URL is specified
        // When creating AppConfig
        // Then it should be stored correctly
        let config = AppConfig {
            initial_url: Some("https://example.com".to_string()),
            ..Default::default()
        };

        assert_eq!(config.initial_url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_app_config_fullscreen_mode() {
        // Given fullscreen mode is enabled
        // When creating AppConfig
        // Then fullscreen should be true
        let config = AppConfig {
            fullscreen: true,
            ..Default::default()
        };

        assert_eq!(config.fullscreen, true);
    }

    #[test]
    fn test_app_config_headless_mode() {
        // Given headless mode is enabled
        // When creating AppConfig
        // Then headless should be true
        let config = AppConfig {
            headless: true,
            ..Default::default()
        };

        assert_eq!(config.headless, true);
    }

    #[test]
    fn test_app_config_devtools_enabled() {
        // Given devtools are enabled
        // When creating AppConfig
        // Then enable_devtools should be true
        let config = AppConfig {
            enable_devtools: true,
            ..Default::default()
        };

        assert_eq!(config.enable_devtools, true);
    }

    #[test]
    fn test_app_config_custom_log_level() {
        // Given a custom log level is specified
        // When creating AppConfig
        // Then it should be stored correctly
        let config = AppConfig {
            log_level: LogLevel::Debug,
            ..Default::default()
        };

        assert_eq!(config.log_level, LogLevel::Debug);
    }

    #[test]
    fn test_app_config_full() {
        // Given all configuration options are specified
        // When creating AppConfig
        // Then all fields should be set correctly
        let config = AppConfig {
            user_data_dir: Some("/data".to_string()),
            initial_url: Some("https://test.com".to_string()),
            fullscreen: true,
            headless: true,
            enable_devtools: true,
            log_level: LogLevel::Trace,
        };

        assert_eq!(config.user_data_dir, Some("/data".to_string()));
        assert_eq!(config.initial_url, Some("https://test.com".to_string()));
        assert_eq!(config.fullscreen, true);
        assert_eq!(config.headless, true);
        assert_eq!(config.enable_devtools, true);
        assert_eq!(config.log_level, LogLevel::Trace);
    }
}
