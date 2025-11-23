//! File association types and data structures
//!
//! This module defines the core types used for file type associations and protocol handling.

use serde::{Deserialize, Serialize};
use std::fmt;

/// File types and protocols that can be associated with the browser
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileAssociation {
    /// HTML files (.html extension)
    HtmlFile,
    /// HTM files (.htm extension)
    HtmFile,
    /// PDF files (.pdf extension)
    PdfFile,
    /// HTTP protocol (http://)
    HttpProtocol,
    /// HTTPS protocol (https://)
    HttpsProtocol,
    /// File protocol (file://)
    FileProtocol,
}

impl FileAssociation {
    /// Get the file extension for file-based associations
    ///
    /// Returns `Some(extension)` for file types, `None` for protocols
    pub fn extension(&self) -> Option<&'static str> {
        match self {
            Self::HtmlFile => Some("html"),
            Self::HtmFile => Some("htm"),
            Self::PdfFile => Some("pdf"),
            Self::HttpProtocol | Self::HttpsProtocol | Self::FileProtocol => None,
        }
    }

    /// Get the MIME type for file-based associations
    ///
    /// Returns `Some(mime_type)` for file types, `None` for protocols
    pub fn mime_type(&self) -> Option<&'static str> {
        match self {
            Self::HtmlFile | Self::HtmFile => Some("text/html"),
            Self::PdfFile => Some("application/pdf"),
            Self::HttpProtocol | Self::HttpsProtocol | Self::FileProtocol => None,
        }
    }

    /// Get the protocol scheme for protocol-based associations
    ///
    /// Returns `Some(scheme)` for protocols, `None` for file types
    pub fn protocol_scheme(&self) -> Option<&'static str> {
        match self {
            Self::HttpProtocol => Some("http"),
            Self::HttpsProtocol => Some("https"),
            Self::FileProtocol => Some("file"),
            Self::HtmlFile | Self::HtmFile | Self::PdfFile => None,
        }
    }

    /// Check if this association is for a file type (vs protocol)
    pub fn is_file_type(&self) -> bool {
        self.extension().is_some()
    }

    /// Check if this association is for a protocol (vs file type)
    pub fn is_protocol(&self) -> bool {
        self.protocol_scheme().is_some()
    }

    /// Get a human-readable description of this association
    pub fn description(&self) -> &'static str {
        match self {
            Self::HtmlFile => "HTML files (.html)",
            Self::HtmFile => "HTML files (.htm)",
            Self::PdfFile => "PDF documents (.pdf)",
            Self::HttpProtocol => "HTTP protocol (http://)",
            Self::HttpsProtocol => "HTTPS protocol (https://)",
            Self::FileProtocol => "File protocol (file://)",
        }
    }

    /// Get all file type associations
    pub fn file_types() -> &'static [FileAssociation] {
        &[Self::HtmlFile, Self::HtmFile, Self::PdfFile]
    }

    /// Get all protocol associations
    pub fn protocols() -> &'static [FileAssociation] {
        &[Self::HttpProtocol, Self::HttpsProtocol, Self::FileProtocol]
    }

    /// Get all associations
    pub fn all() -> &'static [FileAssociation] {
        &[
            Self::HtmlFile,
            Self::HtmFile,
            Self::PdfFile,
            Self::HttpProtocol,
            Self::HttpsProtocol,
            Self::FileProtocol,
        ]
    }
}

impl fmt::Display for FileAssociation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Status of a file association registration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssociationStatus {
    /// This application is the registered handler
    Registered,
    /// Another application is the registered handler
    RegisteredOther,
    /// No handler is registered
    NotRegistered,
    /// Unable to determine status
    Unknown,
}

impl Default for AssociationStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Configuration for the association service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationConfig {
    /// Application name for registration
    pub app_name: String,
    /// Application identifier (e.g., app bundle ID, desktop file name)
    pub app_id: String,
    /// Application executable path
    pub executable_path: Option<String>,
    /// Application icon path
    pub icon_path: Option<String>,
    /// Generic name for the application
    pub generic_name: String,
    /// Comment/description for the application
    pub comment: String,
}

impl Default for AssociationConfig {
    fn default() -> Self {
        Self {
            app_name: String::from("Corten Browser"),
            app_id: String::from("corten-browser"),
            executable_path: None,
            icon_path: None,
            generic_name: String::from("Web Browser"),
            comment: String::from("Browse the World Wide Web"),
        }
    }
}

impl AssociationConfig {
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

    /// Set the generic name
    pub fn with_generic_name(mut self, name: impl Into<String>) -> Self {
        self.generic_name = name.into();
        self
    }

    /// Set the comment/description
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = comment.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_association_extension() {
        assert_eq!(FileAssociation::HtmlFile.extension(), Some("html"));
        assert_eq!(FileAssociation::HtmFile.extension(), Some("htm"));
        assert_eq!(FileAssociation::PdfFile.extension(), Some("pdf"));
        assert_eq!(FileAssociation::HttpProtocol.extension(), None);
        assert_eq!(FileAssociation::HttpsProtocol.extension(), None);
        assert_eq!(FileAssociation::FileProtocol.extension(), None);
    }

    #[test]
    fn test_file_association_mime_type() {
        assert_eq!(FileAssociation::HtmlFile.mime_type(), Some("text/html"));
        assert_eq!(FileAssociation::HtmFile.mime_type(), Some("text/html"));
        assert_eq!(FileAssociation::PdfFile.mime_type(), Some("application/pdf"));
        assert_eq!(FileAssociation::HttpProtocol.mime_type(), None);
    }

    #[test]
    fn test_file_association_protocol_scheme() {
        assert_eq!(FileAssociation::HttpProtocol.protocol_scheme(), Some("http"));
        assert_eq!(
            FileAssociation::HttpsProtocol.protocol_scheme(),
            Some("https")
        );
        assert_eq!(FileAssociation::FileProtocol.protocol_scheme(), Some("file"));
        assert_eq!(FileAssociation::HtmlFile.protocol_scheme(), None);
    }

    #[test]
    fn test_file_association_is_file_type() {
        assert!(FileAssociation::HtmlFile.is_file_type());
        assert!(FileAssociation::HtmFile.is_file_type());
        assert!(FileAssociation::PdfFile.is_file_type());
        assert!(!FileAssociation::HttpProtocol.is_file_type());
        assert!(!FileAssociation::HttpsProtocol.is_file_type());
        assert!(!FileAssociation::FileProtocol.is_file_type());
    }

    #[test]
    fn test_file_association_is_protocol() {
        assert!(!FileAssociation::HtmlFile.is_protocol());
        assert!(!FileAssociation::HtmFile.is_protocol());
        assert!(!FileAssociation::PdfFile.is_protocol());
        assert!(FileAssociation::HttpProtocol.is_protocol());
        assert!(FileAssociation::HttpsProtocol.is_protocol());
        assert!(FileAssociation::FileProtocol.is_protocol());
    }

    #[test]
    fn test_file_association_description() {
        assert!(!FileAssociation::HtmlFile.description().is_empty());
        assert!(!FileAssociation::HttpProtocol.description().is_empty());
    }

    #[test]
    fn test_file_association_display() {
        let html = FileAssociation::HtmlFile;
        let display = format!("{}", html);
        assert!(display.contains("html"));
    }

    #[test]
    fn test_file_association_file_types() {
        let file_types = FileAssociation::file_types();
        assert_eq!(file_types.len(), 3);
        assert!(file_types.contains(&FileAssociation::HtmlFile));
        assert!(file_types.contains(&FileAssociation::HtmFile));
        assert!(file_types.contains(&FileAssociation::PdfFile));
    }

    #[test]
    fn test_file_association_protocols() {
        let protocols = FileAssociation::protocols();
        assert_eq!(protocols.len(), 3);
        assert!(protocols.contains(&FileAssociation::HttpProtocol));
        assert!(protocols.contains(&FileAssociation::HttpsProtocol));
        assert!(protocols.contains(&FileAssociation::FileProtocol));
    }

    #[test]
    fn test_file_association_all() {
        let all = FileAssociation::all();
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_association_status_default() {
        let status = AssociationStatus::default();
        assert_eq!(status, AssociationStatus::Unknown);
    }

    #[test]
    fn test_association_config_default() {
        let config = AssociationConfig::default();
        assert_eq!(config.app_name, "Corten Browser");
        assert_eq!(config.app_id, "corten-browser");
        assert!(config.executable_path.is_none());
        assert!(config.icon_path.is_none());
    }

    #[test]
    fn test_association_config_builder() {
        let config = AssociationConfig::new("Test Browser", "test-browser")
            .with_executable("/usr/bin/test-browser")
            .with_icon("/usr/share/icons/test.png")
            .with_generic_name("Web Browser")
            .with_comment("A test browser");

        assert_eq!(config.app_name, "Test Browser");
        assert_eq!(config.app_id, "test-browser");
        assert_eq!(
            config.executable_path,
            Some("/usr/bin/test-browser".to_string())
        );
        assert_eq!(
            config.icon_path,
            Some("/usr/share/icons/test.png".to_string())
        );
        assert_eq!(config.generic_name, "Web Browser");
        assert_eq!(config.comment, "A test browser");
    }
}
