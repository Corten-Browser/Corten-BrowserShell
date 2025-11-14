use anyhow::{anyhow, Result};

/// Validate URL format and scheme
pub fn validate_url(url: &str) -> Result<()> {
    if url.is_empty() {
        return Err(anyhow!("URL cannot be empty"));
    }

    // Basic URL parsing
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(anyhow!("Invalid URL scheme. Only http:// and https:// are supported"));
    }

    // Check if it has a valid domain
    let without_scheme = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .ok_or_else(|| anyhow!("Invalid URL format"))?;

    if without_scheme.is_empty() {
        return Err(anyhow!("Invalid URL format"));
    }

    Ok(())
}

/// Sanitize file name to prevent path traversal
pub fn sanitize_file_name(file_name: &str) -> String {
    if file_name.is_empty() {
        return "download".to_string();
    }

    // Normalize path separators and remove leading/trailing slashes
    let normalized = file_name
        .replace('\\', "/")
        .trim_matches('/')
        .to_string();

    // Remove all ".." and "." segments and filter out empty segments
    let cleaned_segments: Vec<&str> = normalized
        .split('/')
        .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        .collect();

    // Join remaining segments with underscores
    let joined = cleaned_segments.join("_");

    if joined.is_empty() {
        return "download".to_string();
    }

    // Replace invalid characters
    let mut result = String::new();
    for c in joined.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => result.push(c),
            _ => result.push('_'),
        }
    }

    if result.is_empty() {
        "download".to_string()
    } else {
        result
    }
}

/// Validate save path
pub fn validate_save_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(anyhow!("Save path cannot be empty"));
    }
    Ok(())
}

/// Extract file name from URL
pub fn extract_file_name_from_url(url: &str) -> String {
    // Remove query parameters
    let without_query = url.split('?').next().unwrap_or(url);

    // Get the last path segment
    let segments: Vec<&str> = without_query.split('/').collect();
    let last_segment = segments.last().copied().unwrap_or("");

    // If last segment is empty (trailing slash) or no valid segment, return default
    if last_segment.is_empty() {
        return "download".to_string();
    }

    last_segment.to_string()
}
