//! Unit tests for CLI argument parsing
//!
//! These tests verify that command-line arguments are parsed correctly.

use shell_app::{AppConfig, LogLevel, ShellApp};

#[cfg(test)]
mod parse_args_tests {
    use super::*;

    #[test]
    fn test_parse_args_empty() {
        // Given no command-line arguments
        // When parsing arguments
        // Then default configuration should be returned
        let args = vec!["browser".to_string()];
        let config = ShellApp::parse_args(args).unwrap();

        assert_eq!(config.user_data_dir, None);
        assert_eq!(config.initial_url, None);
        assert_eq!(config.fullscreen, false);
        assert_eq!(config.headless, false);
        assert_eq!(config.enable_devtools, false);
        assert_eq!(config.log_level, LogLevel::Info);
    }

    #[test]
    fn test_parse_args_user_data_dir() {
        // Given --user-data-dir argument
        // When parsing arguments
        // Then user_data_dir should be set
        let args = vec![
            "browser".to_string(),
            "--user-data-dir".to_string(),
            "/custom/path".to_string(),
        ];
        let config = ShellApp::parse_args(args).unwrap();

        assert_eq!(config.user_data_dir, Some("/custom/path".to_string()));
    }

    #[test]
    fn test_parse_args_initial_url() {
        // Given --initial-url argument
        // When parsing arguments
        // Then initial_url should be set
        let args = vec![
            "browser".to_string(),
            "--initial-url".to_string(),
            "https://rust-lang.org".to_string(),
        ];
        let config = ShellApp::parse_args(args).unwrap();

        assert_eq!(
            config.initial_url,
            Some("https://rust-lang.org".to_string())
        );
    }

    #[test]
    fn test_parse_args_fullscreen() {
        // Given --fullscreen flag
        // When parsing arguments
        // Then fullscreen should be true
        let args = vec!["browser".to_string(), "--fullscreen".to_string()];
        let config = ShellApp::parse_args(args).unwrap();

        assert_eq!(config.fullscreen, true);
    }

    #[test]
    fn test_parse_args_headless() {
        // Given --headless flag
        // When parsing arguments
        // Then headless should be true
        let args = vec!["browser".to_string(), "--headless".to_string()];
        let config = ShellApp::parse_args(args).unwrap();

        assert_eq!(config.headless, true);
    }

    #[test]
    fn test_parse_args_enable_devtools() {
        // Given --enable-devtools flag
        // When parsing arguments
        // Then enable_devtools should be true
        let args = vec!["browser".to_string(), "--enable-devtools".to_string()];
        let config = ShellApp::parse_args(args).unwrap();

        assert_eq!(config.enable_devtools, true);
    }

    #[test]
    fn test_parse_args_log_level_error() {
        // Given --log-level error
        // When parsing arguments
        // Then log_level should be Error
        let args = vec![
            "browser".to_string(),
            "--log-level".to_string(),
            "error".to_string(),
        ];
        let config = ShellApp::parse_args(args).unwrap();

        assert_eq!(config.log_level, LogLevel::Error);
    }

    #[test]
    fn test_parse_args_log_level_debug() {
        // Given --log-level debug
        // When parsing arguments
        // Then log_level should be Debug
        let args = vec![
            "browser".to_string(),
            "--log-level".to_string(),
            "debug".to_string(),
        ];
        let config = ShellApp::parse_args(args).unwrap();

        assert_eq!(config.log_level, LogLevel::Debug);
    }

    #[test]
    fn test_parse_args_combined() {
        // Given multiple arguments
        // When parsing arguments
        // Then all fields should be set correctly
        let args = vec![
            "browser".to_string(),
            "--user-data-dir".to_string(),
            "/my/data".to_string(),
            "--initial-url".to_string(),
            "https://example.com".to_string(),
            "--fullscreen".to_string(),
            "--enable-devtools".to_string(),
            "--log-level".to_string(),
            "trace".to_string(),
        ];
        let config = ShellApp::parse_args(args).unwrap();

        assert_eq!(config.user_data_dir, Some("/my/data".to_string()));
        assert_eq!(config.initial_url, Some("https://example.com".to_string()));
        assert_eq!(config.fullscreen, true);
        assert_eq!(config.headless, false);
        assert_eq!(config.enable_devtools, true);
        assert_eq!(config.log_level, LogLevel::Trace);
    }

    #[test]
    fn test_parse_args_invalid_log_level() {
        // Given --log-level with invalid value
        // When parsing arguments
        // Then an error should be returned
        let args = vec![
            "browser".to_string(),
            "--log-level".to_string(),
            "invalid".to_string(),
        ];
        let result = ShellApp::parse_args(args);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_unknown_flag() {
        // Given an unknown flag
        // When parsing arguments
        // Then an error should be returned
        let args = vec!["browser".to_string(), "--unknown-flag".to_string()];
        let result = ShellApp::parse_args(args);

        assert!(result.is_err());
    }
}
