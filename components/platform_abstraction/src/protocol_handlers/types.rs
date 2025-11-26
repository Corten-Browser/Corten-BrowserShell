//! Protocol handler types and data structures
//!
//! This module defines the core types used for URL protocol handling.

use serde::{Deserialize, Serialize};
use std::fmt;

/// URL protocols that can be handled by the browser
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    /// HTTP protocol (http://)
    Http,
    /// HTTPS protocol (https://)
    Https,
    /// File protocol (file://)
    File,
    /// Custom protocol (user-defined)
    Custom,
}

impl Protocol {
    /// Get the protocol scheme string
    pub fn scheme(&self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::File => "file",
            Self::Custom => "",
        }
    }

    /// Get a human-readable description of this protocol
    pub fn description(&self) -> &'static str {
        match self {
            Self::Http => "HTTP protocol (http://)",
            Self::Https => "HTTPS protocol (https://)",
            Self::File => "File protocol (file://)",
            Self::Custom => "Custom protocol",
        }
    }

    /// Get all standard protocols
    pub fn standard_protocols() -> &'static [Protocol] {
        &[Self::Http, Self::Https, Self::File]
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Configuration for protocol handler registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    /// Application name for protocol registration
    pub app_name: String,
    /// Application identifier
    pub app_id: String,
    /// Application executable path
    pub executable_path: Option<String>,
    /// Application icon path
    pub icon_path: Option<String>,
    /// Command template for handling URLs (e.g., "%s" for URL placeholder)
    pub command_template: Option<String>,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            app_name: String::from("Corten Browser"),
            app_id: String::from("corten-browser"),
            executable_path: None,
            icon_path: None,
            command_template: Some("%s".to_string()),
        }
    }
}

impl ProtocolConfig {
    /// Create a new configuration with the given app name and ID
    pub fn new(app_name: impl Into<String>, app_id: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            app_id: app_id.into(),
            ..Default::default()
        }
    }

    /// Set the executable path
    pub fn with_executable(mut self, path: impl Into<String>) -> Self {
        self.executable_path = Some(path.into());
        self
    }

    /// Set the icon path
    pub fn with_icon(mut self, path: impl Into<String>) -> Self {
        self.icon_path = Some(path.into());
        self
    }

    /// Set the command template
    pub fn with_command_template(mut self, template: impl Into<String>) -> Self {
        self.command_template = Some(template.into());
        self
    }
}

/// Status of a protocol handler registration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolStatus {
    /// This application is the registered handler
    Registered,
    /// Another application is the registered handler
    RegisteredOther,
    /// No handler is registered
    NotRegistered,
    /// Unable to determine status
    Unknown,
}

impl Default for ProtocolStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// A URL to be handled by a protocol handler
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolUrl {
    /// The full URL string
    pub url: String,
    /// The protocol scheme (e.g., "http", "https", "custom")
    pub scheme: String,
    /// The host part of the URL (if applicable)
    pub host: Option<String>,
    /// The path part of the URL (if applicable)
    pub path: Option<String>,
}

impl ProtocolUrl {
    /// Create a new protocol URL from a string
    ///
    /// This performs basic parsing to extract scheme, host, and path.
    pub fn new(url: impl Into<String>) -> Self {
        let url = url.into();
        let parts: Vec<&str> = url.splitn(2, "://").collect();

        if parts.len() == 2 {
            let scheme = parts[0].to_string();
            let rest = parts[1];
            let path_parts: Vec<&str> = rest.splitn(2, '/').collect();

            // Host is present (even if empty) if we have path parts
            let host = if !path_parts.is_empty() {
                Some(path_parts[0].to_string())
            } else {
                None
            };

            let path = if path_parts.len() == 2 {
                Some(format!("/{}", path_parts[1]))
            } else {
                None
            };

            Self {
                url,
                scheme,
                host,
                path,
            }
        } else {
            Self {
                url: url.clone(),
                scheme: String::new(),
                host: None,
                path: Some(url),
            }
        }
    }

    /// Get the protocol type for this URL
    pub fn protocol(&self) -> Protocol {
        match self.scheme.as_str() {
            "http" => Protocol::Http,
            "https" => Protocol::Https,
            "file" => Protocol::File,
            _ => Protocol::Custom,
        }
    }
}

impl fmt::Display for ProtocolUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}

impl From<String> for ProtocolUrl {
    fn from(url: String) -> Self {
        Self::new(url)
    }
}

impl From<&str> for ProtocolUrl {
    fn from(url: &str) -> Self {
        Self::new(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_scheme() {
        assert_eq!(Protocol::Http.scheme(), "http");
        assert_eq!(Protocol::Https.scheme(), "https");
        assert_eq!(Protocol::File.scheme(), "file");
    }

    #[test]
    fn test_protocol_description() {
        assert!(!Protocol::Http.description().is_empty());
        assert!(!Protocol::Https.description().is_empty());
    }

    #[test]
    fn test_protocol_display() {
        let http = Protocol::Http;
        let display = format!("{}", http);
        assert!(display.contains("http://"));
    }

    #[test]
    fn test_protocol_standard_protocols() {
        let protocols = Protocol::standard_protocols();
        assert_eq!(protocols.len(), 3);
        assert!(protocols.contains(&Protocol::Http));
        assert!(protocols.contains(&Protocol::Https));
        assert!(protocols.contains(&Protocol::File));
    }

    #[test]
    fn test_protocol_config_default() {
        let config = ProtocolConfig::default();
        assert_eq!(config.app_name, "Corten Browser");
        assert_eq!(config.app_id, "corten-browser");
        assert!(config.executable_path.is_none());
        assert!(config.icon_path.is_none());
        assert_eq!(config.command_template, Some("%s".to_string()));
    }

    #[test]
    fn test_protocol_config_builder() {
        let config = ProtocolConfig::new("Test Browser", "test-browser")
            .with_executable("/usr/bin/test-browser")
            .with_icon("/usr/share/icons/test.png")
            .with_command_template("%u");

        assert_eq!(config.app_name, "Test Browser");
        assert_eq!(config.app_id, "test-browser");
        assert_eq!(
            config.executable_path,
            Some("/usr/bin/test-browser".to_string())
        );
        assert_eq!(config.icon_path, Some("/usr/share/icons/test.png".to_string()));
        assert_eq!(config.command_template, Some("%u".to_string()));
    }

    #[test]
    fn test_protocol_status_default() {
        let status = ProtocolStatus::default();
        assert_eq!(status, ProtocolStatus::Unknown);
    }

    #[test]
    fn test_protocol_url_new() {
        let url = ProtocolUrl::new("https://example.com/path");
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, Some("example.com".to_string()));
        assert_eq!(url.path, Some("/path".to_string()));
    }

    #[test]
    fn test_protocol_url_http() {
        let url = ProtocolUrl::new("http://localhost:8080/test");
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, Some("localhost:8080".to_string()));
        assert_eq!(url.path, Some("/test".to_string()));
    }

    #[test]
    fn test_protocol_url_file() {
        let url = ProtocolUrl::new("file:///home/user/document.html");
        assert_eq!(url.scheme, "file");
        assert_eq!(url.host, Some("".to_string()));
        assert_eq!(url.path, Some("/home/user/document.html".to_string()));
    }

    #[test]
    fn test_protocol_url_protocol() {
        let http_url = ProtocolUrl::new("http://example.com");
        assert_eq!(http_url.protocol(), Protocol::Http);

        let https_url = ProtocolUrl::new("https://example.com");
        assert_eq!(https_url.protocol(), Protocol::Https);

        let file_url = ProtocolUrl::new("file:///path");
        assert_eq!(file_url.protocol(), Protocol::File);

        let custom_url = ProtocolUrl::new("myprotocol://data");
        assert_eq!(custom_url.protocol(), Protocol::Custom);
    }

    #[test]
    fn test_protocol_url_display() {
        let url = ProtocolUrl::new("https://example.com/path");
        let display = format!("{}", url);
        assert_eq!(display, "https://example.com/path");
    }

    #[test]
    fn test_protocol_url_from_string() {
        let url: ProtocolUrl = "https://example.com".to_string().into();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, Some("example.com".to_string()));
    }

    #[test]
    fn test_protocol_url_from_str() {
        let url: ProtocolUrl = "https://example.com".into();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, Some("example.com".to_string()));
    }
}
