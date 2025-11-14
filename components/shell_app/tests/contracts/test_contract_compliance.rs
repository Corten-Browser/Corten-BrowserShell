//! Contract compliance tests
//!
//! These tests verify that shell_app implements the EXACT API defined in
//! contracts/shell_app.yaml. This is CRITICAL for integration with other components.

use shared_types::ComponentError;
use shell_app::{AppConfig, LogLevel, ShellApp};

#[cfg(test)]
mod contract_tests {
    use super::*;

    /// Test that ShellApp exports the exact API from contract
    #[test]
    fn test_shell_app_has_main_method() {
        // Contract specifies: ShellApp.main(args: Vec<String>) -> Result<(), ComponentError>
        // Verify the method exists and can be called
        let args = vec!["test".to_string()];

        // This should compile if the signature matches the contract
        let _: fn(
            Vec<String>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), ComponentError>> + Send>,
        > = |args| Box::pin(ShellApp::main(args));
    }

    #[test]
    fn test_shell_app_has_parse_args_method() {
        // Contract specifies: ShellApp.parse_args(args: Vec<String>) -> Result<AppConfig, ComponentError>
        let args = vec!["test".to_string()];
        let result = ShellApp::parse_args(args);

        // Verify return type matches contract
        assert!(result.is_ok());
        let config: AppConfig = result.unwrap();

        // Verify config has all required fields from contract
        let _ = config.user_data_dir;
        let _ = config.initial_url;
        let _ = config.fullscreen;
        let _ = config.headless;
        let _ = config.enable_devtools;
        let _ = config.log_level;
    }

    /// Test that AppConfig has all required fields from contract
    #[test]
    fn test_app_config_has_required_fields() {
        let config = AppConfig {
            user_data_dir: None,
            initial_url: None,
            fullscreen: false,
            headless: false,
            enable_devtools: false,
            log_level: LogLevel::Info,
        };

        // Verify all fields are accessible (contract compliance)
        assert_eq!(config.user_data_dir, None);
        assert_eq!(config.initial_url, None);
        assert_eq!(config.fullscreen, false);
        assert_eq!(config.headless, false);
        assert_eq!(config.enable_devtools, false);
        assert_eq!(config.log_level, LogLevel::Info);
    }

    /// Test that LogLevel has all required variants from contract
    #[test]
    fn test_log_level_has_required_variants() {
        // Contract specifies: LogLevel enum with Error, Warn, Info, Debug, Trace
        let _error = LogLevel::Error;
        let _warn = LogLevel::Warn;
        let _info = LogLevel::Info;
        let _debug = LogLevel::Debug;
        let _trace = LogLevel::Trace;

        // All variants must be constructible
        assert!(true); // If we got here, all variants exist
    }

    /// Test that parse_args returns correct error type
    #[test]
    fn test_parse_args_returns_component_error() {
        let args = vec!["test".to_string(), "--invalid-flag".to_string()];
        let result = ShellApp::parse_args(args);

        // Contract specifies: returns Result<AppConfig, ComponentError>
        assert!(result.is_err());

        // Verify it's actually ComponentError
        let err: ComponentError = result.unwrap_err();
        let _err_string = format!("{}", err); // ComponentError must implement Display
    }

    /// Test that main is async (contract specifies async: true)
    #[tokio::test]
    async fn test_main_is_async() {
        // Contract specifies main is async
        let args = vec!["test".to_string()];
        let result = ShellApp::main(args).await;

        // Should return Result<(), ComponentError>
        assert!(result.is_ok());
    }

    /// Test exact field types match contract
    #[test]
    fn test_app_config_field_types() {
        let config = AppConfig::default();

        // Contract specifies:
        // user_data_dir: Option<String>
        let _: Option<String> = config.user_data_dir;

        // initial_url: Option<String>
        let _: Option<String> = config.initial_url;

        // fullscreen: bool
        let _: bool = config.fullscreen;

        // headless: bool
        let _: bool = config.headless;

        // enable_devtools: bool
        let _: bool = config.enable_devtools;

        // log_level: LogLevel
        let _: LogLevel = config.log_level;
    }

    /// Test that LogLevel variants are exactly as specified
    #[test]
    fn test_log_level_variant_names() {
        // Contract specifies exact variant names
        match LogLevel::Error {
            LogLevel::Error => assert!(true),
            _ => panic!("LogLevel::Error variant must exist"),
        }

        match LogLevel::Warn {
            LogLevel::Warn => assert!(true),
            _ => panic!("LogLevel::Warn variant must exist"),
        }

        match LogLevel::Info {
            LogLevel::Info => assert!(true),
            _ => panic!("LogLevel::Info variant must exist"),
        }

        match LogLevel::Debug {
            LogLevel::Debug => assert!(true),
            _ => panic!("LogLevel::Debug variant must exist"),
        }

        match LogLevel::Trace {
            LogLevel::Trace => assert!(true),
            _ => panic!("LogLevel::Trace variant must exist"),
        }
    }

    /// Test parse_args with all contract-specified arguments
    #[test]
    fn test_parse_args_accepts_all_contract_args() {
        // Contract specifies these arguments:
        // --user-data-dir, --initial-url, --fullscreen, --headless,
        // --enable-devtools, --log-level

        let args = vec![
            "browser".to_string(),
            "--user-data-dir".to_string(),
            "/path".to_string(),
            "--initial-url".to_string(),
            "https://example.com".to_string(),
            "--fullscreen".to_string(),
            "--headless".to_string(),
            "--enable-devtools".to_string(),
            "--log-level".to_string(),
            "debug".to_string(),
        ];

        let config = ShellApp::parse_args(args).unwrap();

        // Verify all values are set correctly
        assert_eq!(config.user_data_dir, Some("/path".to_string()));
        assert_eq!(config.initial_url, Some("https://example.com".to_string()));
        assert_eq!(config.fullscreen, true);
        assert_eq!(config.headless, true);
        assert_eq!(config.enable_devtools, true);
        assert_eq!(config.log_level, LogLevel::Debug);
    }
}
