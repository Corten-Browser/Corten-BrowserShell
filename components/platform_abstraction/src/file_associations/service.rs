//! File association service trait and implementations
//!
//! This module provides the service interface and platform-specific implementations
//! for registering file type and protocol associations.

use super::types::{AssociationConfig, AssociationStatus, FileAssociation};
use thiserror::Error;

/// Errors that can occur during file association operations
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AssociationError {
    /// Registration is not supported on this platform
    #[error("file association registration is not supported on this platform")]
    NotSupported,

    /// Insufficient permissions to modify system settings
    #[error("insufficient permissions to register file associations")]
    PermissionDenied,

    /// The association type is not supported
    #[error("association type not supported: {0}")]
    UnsupportedAssociation(String),

    /// Registration failed for a platform-specific reason
    #[error("registration failed: {0}")]
    RegistrationFailed(String),

    /// Unregistration failed for a platform-specific reason
    #[error("unregistration failed: {0}")]
    UnregistrationFailed(String),

    /// Failed to query association status
    #[error("failed to query association status: {0}")]
    QueryFailed(String),

    /// Configuration is invalid or incomplete
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// Platform-specific error
    #[error("platform error: {0}")]
    PlatformError(String),
}

/// Result type for association operations
pub type AssociationResult<T> = Result<T, AssociationError>;

/// Service trait for managing file type and protocol associations
///
/// This trait provides a cross-platform interface for registering the browser
/// as the default handler for various file types and URL protocols.
///
/// # Platform-Specific Behavior
///
/// - **Linux**: Creates/modifies `.desktop` files and uses `xdg-mime`
/// - **Windows**: Modifies the Windows Registry
/// - **macOS**: Modifies `Info.plist` and uses Launch Services
///
/// # Example
///
/// ```rust,no_run
/// use platform_abstraction::file_associations::{
///     AssociationService, FileAssociation, SystemAssociationService, AssociationConfig,
/// };
///
/// let config = AssociationConfig::default();
/// let service = SystemAssociationService::new(config);
///
/// // Check if we can register associations
/// if service.can_register() {
///     // Register as handler for HTML files
///     service.register(FileAssociation::HtmlFile).unwrap();
///
///     // Register for HTTPS protocol
///     service.register(FileAssociation::HttpsProtocol).unwrap();
/// }
///
/// // Check if we're the default browser
/// if service.is_default_browser() {
///     println!("We are the default browser!");
/// }
/// ```
pub trait AssociationService {
    /// Register this application as the handler for the given association
    ///
    /// # Arguments
    ///
    /// * `association` - The file type or protocol to register for
    ///
    /// # Returns
    ///
    /// * `Ok(())` if registration was successful
    /// * `Err(AssociationError)` if registration failed
    ///
    /// # Errors
    ///
    /// - `NotSupported`: Platform doesn't support this operation
    /// - `PermissionDenied`: Insufficient permissions
    /// - `RegistrationFailed`: Platform-specific registration failure
    fn register(&self, association: FileAssociation) -> AssociationResult<()>;

    /// Unregister this application as the handler for the given association
    ///
    /// # Arguments
    ///
    /// * `association` - The file type or protocol to unregister
    ///
    /// # Returns
    ///
    /// * `Ok(())` if unregistration was successful
    /// * `Err(AssociationError)` if unregistration failed
    fn unregister(&self, association: FileAssociation) -> AssociationResult<()>;

    /// Check if this application is registered as the handler for the given association
    ///
    /// # Arguments
    ///
    /// * `association` - The file type or protocol to check
    ///
    /// # Returns
    ///
    /// `true` if this application is the registered handler, `false` otherwise
    fn is_registered(&self, association: FileAssociation) -> bool;

    /// Get the detailed status of an association
    ///
    /// # Arguments
    ///
    /// * `association` - The file type or protocol to check
    ///
    /// # Returns
    ///
    /// The current registration status
    fn get_status(&self, association: FileAssociation) -> AssociationStatus;

    /// Check if this application is the default web browser
    ///
    /// This checks if the application is registered for HTTP and HTTPS protocols.
    ///
    /// # Returns
    ///
    /// `true` if this application is the default browser, `false` otherwise
    fn is_default_browser(&self) -> bool;

    /// Check if this application can register file associations
    ///
    /// This checks platform capabilities and permissions.
    ///
    /// # Returns
    ///
    /// `true` if registration is possible, `false` otherwise
    fn can_register(&self) -> bool;

    /// Register as the default browser (HTTP, HTTPS, and common file types)
    ///
    /// This is a convenience method that registers for:
    /// - HTTP protocol
    /// - HTTPS protocol
    /// - HTML files
    /// - HTM files
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all registrations were successful
    /// * `Err(AssociationError)` if any registration failed
    fn register_as_default_browser(&self) -> AssociationResult<()> {
        self.register(FileAssociation::HttpProtocol)?;
        self.register(FileAssociation::HttpsProtocol)?;
        self.register(FileAssociation::HtmlFile)?;
        self.register(FileAssociation::HtmFile)?;
        Ok(())
    }

    /// Register for all supported associations
    ///
    /// This registers for all file types and protocols.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all registrations were successful
    /// * `Err(AssociationError)` if any registration failed
    fn register_all(&self) -> AssociationResult<()> {
        for association in FileAssociation::all() {
            self.register(*association)?;
        }
        Ok(())
    }
}

/// System implementation of the association service
///
/// This provides the actual platform-specific implementation for registering
/// file associations. Currently implements stub behavior with proper error
/// handling, with full platform integration planned for future phases.
pub struct SystemAssociationService {
    config: AssociationConfig,
}

impl SystemAssociationService {
    /// Create a new system association service with the given configuration
    pub fn new(config: AssociationConfig) -> Self {
        Self { config }
    }

    /// Create a new system association service with default configuration
    pub fn with_defaults() -> Self {
        Self::new(AssociationConfig::default())
    }

    /// Get the current configuration
    pub fn config(&self) -> &AssociationConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: AssociationConfig) {
        self.config = config;
    }
}

// Platform-specific implementations

#[cfg(target_os = "linux")]
impl AssociationService for SystemAssociationService {
    fn register(&self, association: FileAssociation) -> AssociationResult<()> {
        // TODO: Implement using xdg-mime and .desktop files
        // For now, return a stub implementation that logs the intent
        log::info!(
            "Linux: Would register {} for {} (stub)",
            self.config.app_id,
            association
        );

        // Validate configuration
        if self.config.app_id.is_empty() {
            return Err(AssociationError::InvalidConfig(
                "app_id is required".to_string(),
            ));
        }

        // Stub: Pretend registration succeeded
        Ok(())
    }

    fn unregister(&self, association: FileAssociation) -> AssociationResult<()> {
        log::info!(
            "Linux: Would unregister {} for {} (stub)",
            self.config.app_id,
            association
        );
        Ok(())
    }

    fn is_registered(&self, association: FileAssociation) -> bool {
        // Stub: Always return false until full implementation
        log::debug!(
            "Linux: Checking if {} is registered for {} (stub)",
            self.config.app_id,
            association
        );
        false
    }

    fn get_status(&self, association: FileAssociation) -> AssociationStatus {
        log::debug!("Linux: Getting status for {} (stub)", association);
        AssociationStatus::Unknown
    }

    fn is_default_browser(&self) -> bool {
        // Stub: Check would use xdg-settings get default-web-browser
        log::debug!("Linux: Checking if default browser (stub)");
        false
    }

    fn can_register(&self) -> bool {
        // On Linux, we can typically register if:
        // 1. We can write to ~/.local/share/applications/
        // 2. xdg-mime is available
        // Stub: Always return true for now
        true
    }
}

#[cfg(target_os = "windows")]
impl AssociationService for SystemAssociationService {
    fn register(&self, association: FileAssociation) -> AssociationResult<()> {
        // TODO: Implement using Windows Registry
        log::info!(
            "Windows: Would register {} for {} (stub)",
            self.config.app_id,
            association
        );

        if self.config.app_id.is_empty() {
            return Err(AssociationError::InvalidConfig(
                "app_id is required".to_string(),
            ));
        }

        Ok(())
    }

    fn unregister(&self, association: FileAssociation) -> AssociationResult<()> {
        log::info!(
            "Windows: Would unregister {} for {} (stub)",
            self.config.app_id,
            association
        );
        Ok(())
    }

    fn is_registered(&self, association: FileAssociation) -> bool {
        log::debug!(
            "Windows: Checking if {} is registered for {} (stub)",
            self.config.app_id,
            association
        );
        false
    }

    fn get_status(&self, association: FileAssociation) -> AssociationStatus {
        log::debug!("Windows: Getting status for {} (stub)", association);
        AssociationStatus::Unknown
    }

    fn is_default_browser(&self) -> bool {
        // Stub: Check would query registry for default browser
        log::debug!("Windows: Checking if default browser (stub)");
        false
    }

    fn can_register(&self) -> bool {
        // On Windows, registration requires admin or user-level registry access
        // Stub: Always return true for now
        true
    }
}

#[cfg(target_os = "macos")]
impl AssociationService for SystemAssociationService {
    fn register(&self, association: FileAssociation) -> AssociationResult<()> {
        // TODO: Implement using Launch Services
        log::info!(
            "macOS: Would register {} for {} (stub)",
            self.config.app_id,
            association
        );

        if self.config.app_id.is_empty() {
            return Err(AssociationError::InvalidConfig(
                "app_id is required".to_string(),
            ));
        }

        Ok(())
    }

    fn unregister(&self, association: FileAssociation) -> AssociationResult<()> {
        log::info!(
            "macOS: Would unregister {} for {} (stub)",
            self.config.app_id,
            association
        );
        Ok(())
    }

    fn is_registered(&self, association: FileAssociation) -> bool {
        log::debug!(
            "macOS: Checking if {} is registered for {} (stub)",
            self.config.app_id,
            association
        );
        false
    }

    fn get_status(&self, association: FileAssociation) -> AssociationStatus {
        log::debug!("macOS: Getting status for {} (stub)", association);
        AssociationStatus::Unknown
    }

    fn is_default_browser(&self) -> bool {
        // Stub: Check would use LSCopyDefaultHandlerForURLScheme
        log::debug!("macOS: Checking if default browser (stub)");
        false
    }

    fn can_register(&self) -> bool {
        // On macOS, registration typically works from app bundles
        // Stub: Always return true for now
        true
    }
}

// Fallback implementation for other platforms
#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
impl AssociationService for SystemAssociationService {
    fn register(&self, association: FileAssociation) -> AssociationResult<()> {
        Err(AssociationError::NotSupported)
    }

    fn unregister(&self, association: FileAssociation) -> AssociationResult<()> {
        Err(AssociationError::NotSupported)
    }

    fn is_registered(&self, _association: FileAssociation) -> bool {
        false
    }

    fn get_status(&self, _association: FileAssociation) -> AssociationStatus {
        AssociationStatus::Unknown
    }

    fn is_default_browser(&self) -> bool {
        false
    }

    fn can_register(&self) -> bool {
        false
    }
}

/// Check if file association registration is supported on this platform
pub fn associations_supported() -> bool {
    cfg!(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_association_error_display() {
        let error = AssociationError::NotSupported;
        assert!(error.to_string().contains("not supported"));

        let error = AssociationError::PermissionDenied;
        assert!(error.to_string().contains("permission"));

        let error = AssociationError::RegistrationFailed("test".to_string());
        assert!(error.to_string().contains("test"));
    }

    #[test]
    fn test_system_association_service_creation() {
        let service = SystemAssociationService::with_defaults();
        assert_eq!(service.config().app_name, "Corten Browser");
        assert_eq!(service.config().app_id, "corten-browser");
    }

    #[test]
    fn test_system_association_service_with_config() {
        let config =
            AssociationConfig::new("Test Browser", "test-browser").with_executable("/usr/bin/test");
        let service = SystemAssociationService::new(config);

        assert_eq!(service.config().app_name, "Test Browser");
        assert_eq!(service.config().app_id, "test-browser");
        assert_eq!(
            service.config().executable_path,
            Some("/usr/bin/test".to_string())
        );
    }

    #[test]
    fn test_system_association_service_set_config() {
        let mut service = SystemAssociationService::with_defaults();
        let new_config = AssociationConfig::new("New Browser", "new-browser");
        service.set_config(new_config);

        assert_eq!(service.config().app_name, "New Browser");
    }

    #[test]
    fn test_associations_supported() {
        // This test verifies the function compiles and returns a boolean
        let supported = associations_supported();
        // On Linux/Windows/macOS it should be true, on other platforms false
        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        assert!(supported);
    }

    #[test]
    fn test_register_stub() {
        let service = SystemAssociationService::with_defaults();

        // Stub implementation should succeed on supported platforms
        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        {
            let result = service.register(FileAssociation::HtmlFile);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_unregister_stub() {
        let service = SystemAssociationService::with_defaults();

        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        {
            let result = service.unregister(FileAssociation::HttpProtocol);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_is_registered_stub() {
        let service = SystemAssociationService::with_defaults();

        // Stub always returns false
        assert!(!service.is_registered(FileAssociation::HtmlFile));
        assert!(!service.is_registered(FileAssociation::HttpProtocol));
    }

    #[test]
    fn test_get_status_stub() {
        let service = SystemAssociationService::with_defaults();

        // Stub returns Unknown
        let status = service.get_status(FileAssociation::HtmlFile);
        assert_eq!(status, AssociationStatus::Unknown);
    }

    #[test]
    fn test_is_default_browser_stub() {
        let service = SystemAssociationService::with_defaults();

        // Stub returns false
        assert!(!service.is_default_browser());
    }

    #[test]
    fn test_can_register() {
        let service = SystemAssociationService::with_defaults();

        // On supported platforms, should return true
        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        assert!(service.can_register());

        // On unsupported platforms, should return false
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        assert!(!service.can_register());
    }

    #[test]
    fn test_register_with_empty_app_id() {
        let config = AssociationConfig::new("Test", ""); // Empty app_id
        let service = SystemAssociationService::new(config);

        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        {
            let result = service.register(FileAssociation::HtmlFile);
            assert!(result.is_err());
            if let Err(AssociationError::InvalidConfig(msg)) = result {
                assert!(msg.contains("app_id"));
            }
        }
    }

    #[test]
    fn test_register_as_default_browser() {
        let service = SystemAssociationService::with_defaults();

        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        {
            let result = service.register_as_default_browser();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_register_all() {
        let service = SystemAssociationService::with_defaults();

        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        {
            let result = service.register_all();
            assert!(result.is_ok());
        }
    }
}
