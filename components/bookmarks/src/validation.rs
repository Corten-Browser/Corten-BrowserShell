use anyhow::{bail, Result};

/// Validate URL format
pub fn validate_url(url: &str) -> Result<()> {
    if url.is_empty() {
        bail!("URL cannot be empty");
    }

    // Check for dangerous schemes
    let url_lower = url.to_lowercase();
    if url_lower.starts_with("javascript:") || url_lower.starts_with("data:") {
        bail!("Dangerous URL scheme not allowed");
    }

    // Must start with http:// or https://
    if !url_lower.starts_with("http://") && !url_lower.starts_with("https://") {
        bail!("URL must start with http:// or https://");
    }

    Ok(())
}

/// Validate folder path
pub fn validate_folder_path(path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("Folder path cannot be empty");
    }

    // No absolute paths
    if path.starts_with('/') {
        bail!("Folder path cannot be absolute");
    }

    // No parent directory references
    if path.contains("..") {
        bail!("Folder path cannot contain '..'");
    }

    // No double slashes
    if path.contains("//") {
        bail!("Folder path cannot contain '//'");
    }

    Ok(())
}

/// Validate tag
pub fn validate_tag(tag: &str) -> Result<()> {
    if tag.is_empty() {
        bail!("Tag cannot be empty");
    }

    if tag.len() > 100 {
        bail!("Tag cannot be longer than 100 characters");
    }

    // No whitespace allowed
    if tag.contains(char::is_whitespace) {
        bail!("Tag cannot contain whitespace");
    }

    Ok(())
}

/// Sanitize title
pub fn sanitize_title(title: &str) -> String {
    // Replace newlines and tabs with spaces
    let title = title.replace(['\n', '\r', '\t'], " ");

    // Collapse multiple spaces into one
    let title = title.split_whitespace().collect::<Vec<_>>().join(" ");

    // Trim whitespace
    title.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // URL validation tests
    #[test]
    fn test_validate_url_valid_http() {
        assert!(validate_url("http://example.com").is_ok());
    }

    #[test]
    fn test_validate_url_valid_https() {
        assert!(validate_url("https://example.com").is_ok());
    }

    #[test]
    fn test_validate_url_with_path() {
        assert!(validate_url("https://example.com/path/to/page").is_ok());
    }

    #[test]
    fn test_validate_url_with_query() {
        assert!(validate_url("https://example.com?key=value").is_ok());
    }

    #[test]
    fn test_validate_url_invalid_no_scheme() {
        assert!(validate_url("example.com").is_err());
    }

    #[test]
    fn test_validate_url_invalid_empty() {
        assert!(validate_url("").is_err());
    }

    #[test]
    fn test_validate_url_invalid_javascript() {
        assert!(validate_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn test_validate_url_invalid_data() {
        assert!(validate_url("data:text/html,<script>alert(1)</script>").is_err());
    }

    // Folder path validation tests
    #[test]
    fn test_validate_folder_path_valid_simple() {
        assert!(validate_folder_path("Programming").is_ok());
    }

    #[test]
    fn test_validate_folder_path_valid_nested() {
        assert!(validate_folder_path("Programming/Rust").is_ok());
    }

    #[test]
    fn test_validate_folder_path_valid_deep() {
        assert!(validate_folder_path("Programming/Rust/Async/Tokio").is_ok());
    }

    #[test]
    fn test_validate_folder_path_invalid_empty() {
        assert!(validate_folder_path("").is_err());
    }

    #[test]
    fn test_validate_folder_path_invalid_parent_reference() {
        assert!(validate_folder_path("../etc/passwd").is_err());
    }

    #[test]
    fn test_validate_folder_path_invalid_absolute() {
        assert!(validate_folder_path("/etc/passwd").is_err());
    }

    #[test]
    fn test_validate_folder_path_invalid_dot_dot() {
        assert!(validate_folder_path("Programming/../etc").is_err());
    }

    #[test]
    fn test_validate_folder_path_invalid_double_slash() {
        assert!(validate_folder_path("Programming//Rust").is_err());
    }

    // Tag validation tests
    #[test]
    fn test_validate_tag_valid() {
        assert!(validate_tag("rust").is_ok());
        assert!(validate_tag("programming").is_ok());
        assert!(validate_tag("web-dev").is_ok());
    }

    #[test]
    fn test_validate_tag_invalid_empty() {
        assert!(validate_tag("").is_err());
    }

    #[test]
    fn test_validate_tag_invalid_too_long() {
        let long_tag = "a".repeat(101);
        assert!(validate_tag(&long_tag).is_err());
    }

    #[test]
    fn test_validate_tag_invalid_whitespace() {
        assert!(validate_tag("rust programming").is_err());
    }

    // Title sanitization tests
    #[test]
    fn test_sanitize_title_normal() {
        assert_eq!(sanitize_title("Normal Title"), "Normal Title");
    }

    #[test]
    fn test_sanitize_title_trim_whitespace() {
        assert_eq!(sanitize_title("  Title  "), "Title");
    }

    #[test]
    fn test_sanitize_title_remove_newlines() {
        assert_eq!(sanitize_title("Title\nWith\nNewlines"), "Title With Newlines");
    }

    #[test]
    fn test_sanitize_title_remove_tabs() {
        assert_eq!(sanitize_title("Title\tWith\tTabs"), "Title With Tabs");
    }

    #[test]
    fn test_sanitize_title_collapse_spaces() {
        assert_eq!(sanitize_title("Title   With   Spaces"), "Title With Spaces");
    }
}
