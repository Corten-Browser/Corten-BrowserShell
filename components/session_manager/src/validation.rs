//! Input validation utilities

use anyhow::{anyhow, Result};
use std::path::Path;

/// Validate database path
///
/// Ensures the parent directory exists or can be created
pub fn validate_db_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(anyhow!("Database path cannot be empty"));
    }

    let path = Path::new(path);

    // Check if parent directory exists or can be validated
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            // Parent directory doesn't exist - this is okay, we'll create it
            // Just validate it's not an invalid path
            if parent.to_str().is_none() {
                return Err(anyhow!("Invalid database path encoding"));
            }
        }
    }

    Ok(())
}

/// Validate URL (basic validation)
///
/// Currently allows empty URLs (for new tabs)
pub fn validate_url(url: &str) -> Result<()> {
    // Empty URLs are allowed (new tab scenario)
    if url.is_empty() {
        return Ok(());
    }

    // Basic validation - just check for common invalid characters
    if url.contains('\0') {
        return Err(anyhow!("URL contains null character"));
    }

    Ok(())
}

/// Validate session ID
pub fn validate_session_id(id: i64) -> Result<()> {
    if id <= 0 {
        return Err(anyhow!("Session ID must be positive"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_db_path_empty() {
        assert!(validate_db_path("").is_err());
    }

    #[test]
    fn test_validate_db_path_valid() {
        assert!(validate_db_path("session.db").is_ok());
        assert!(validate_db_path("/tmp/session.db").is_ok());
    }

    #[test]
    fn test_validate_url_empty() {
        assert!(validate_url("").is_ok());
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com").is_ok());
    }

    #[test]
    fn test_validate_url_null_char() {
        assert!(validate_url("https://example.com\0").is_err());
    }

    #[test]
    fn test_validate_session_id_positive() {
        assert!(validate_session_id(1).is_ok());
        assert!(validate_session_id(100).is_ok());
    }

    #[test]
    fn test_validate_session_id_invalid() {
        assert!(validate_session_id(0).is_err());
        assert!(validate_session_id(-1).is_err());
    }
}
