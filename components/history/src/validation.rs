use anyhow::{anyhow, Result};

/// Validate URL format
pub fn validate_url(url: &str) -> Result<()> {
    if url.is_empty() {
        return Err(anyhow!("URL cannot be empty"));
    }

    // Must start with http:// or https://
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(anyhow!("URL must start with http:// or https://"));
    }

    // Basic validation that it contains a domain
    if url.len() < 10 {
        return Err(anyhow!("URL is too short"));
    }

    Ok(())
}

/// Validate and sanitize title
pub fn validate_title(title: &str) -> String {
    let trimmed = title.trim();

    if trimmed.is_empty() {
        return "Untitled".to_string();
    }

    // Limit to 255 characters
    if trimmed.len() > 255 {
        trimmed.chars().take(255).collect()
    } else {
        trimmed.to_string()
    }
}

/// Validate timestamp
pub fn validate_timestamp(timestamp: i64) -> Result<()> {
    // Timestamp should not be negative
    if timestamp < 0 {
        return Err(anyhow!("Timestamp cannot be negative"));
    }

    // Timestamp should not be before year 2000 (946684800)
    if timestamp < 946684800 {
        return Err(anyhow!("Timestamp is too old (before year 2000)"));
    }

    // Timestamp should not be more than 1 day in the future
    let now = chrono::Utc::now().timestamp();
    if timestamp > now + 86400 {
        return Err(anyhow!("Timestamp cannot be more than 1 day in the future"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_edge_cases() {
        assert!(validate_url("https://a.b").is_ok());
        assert!(validate_url("http://localhost").is_ok());
    }
}
