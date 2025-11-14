// @implements: REQ-UI-001
//! Address Bar Widget
//!
//! URL input field with validation and security indicators.

/// Result of URL validation
#[derive(Debug, Clone, PartialEq)]
pub enum UrlValidationResult {
    /// Valid URL with normalized form
    Valid(String),
    /// Invalid URL with error message
    Invalid(String),
    /// Search query to be sent to default search engine
    SearchQuery(String),
}

/// Address Bar widget state
#[derive(Debug, Clone)]
pub struct AddressBar {
    text: String,
    is_loading: bool,
    is_focused: bool,
}

impl AddressBar {
    /// Create a new Address Bar widget
    pub fn new() -> Self {
        Self {
            text: String::new(),
            is_loading: false,
            is_focused: false,
        }
    }

    /// Get the current text in the address bar
    pub fn get_text(&self) -> &str {
        &self.text
    }

    /// Set the text in the address bar
    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    /// Check if page is currently loading
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    /// Check if address bar is focused
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Check if current URL is secure (HTTPS)
    pub fn is_secure(&self) -> bool {
        self.text.starts_with("https://")
    }

    /// Validate a URL string
    pub fn validate_url(&self, input: &str) -> UrlValidationResult {
        let trimmed = input.trim();

        // Empty input
        if trimmed.is_empty() {
            return UrlValidationResult::Invalid("Empty URL".to_string());
        }

        // Check if it's a valid URL scheme
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            // Basic validation - has a domain after the scheme
            let after_scheme = if trimmed.starts_with("https://") {
                &trimmed[8..]
            } else {
                &trimmed[7..]
            };

            if after_scheme.is_empty() || !after_scheme.contains('.') {
                return UrlValidationResult::Invalid("Invalid domain".to_string());
            }

            return UrlValidationResult::Valid(trimmed.to_string());
        }

        // Check if it looks like a domain (has a dot and no spaces)
        if trimmed.contains('.') && !trimmed.contains(' ') {
            // Additional validation - check for valid characters
            let has_valid_chars = trimmed.chars().all(|c| {
                c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '/' || c == ':'
            });

            if has_valid_chars {
                // Assume HTTPS for security
                return UrlValidationResult::Valid(format!("https://{}", trimmed));
            }
        }

        // Check if it might be an invalid URL attempt (contains invalid URL characters)
        if trimmed.contains("://") || trimmed.starts_with("www.") {
            return UrlValidationResult::Invalid("Malformed URL".to_string());
        }

        // Otherwise, treat as search query
        UrlValidationResult::SearchQuery(trimmed.to_string())
    }
}

impl Default for AddressBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_bar_new_creates_empty() {
        let bar = AddressBar::new();
        assert_eq!(bar.get_text(), "");
        assert!(!bar.is_loading());
        assert!(!bar.is_focused());
    }

    #[test]
    fn address_bar_default_creates_empty() {
        let bar = AddressBar::default();
        assert_eq!(bar.get_text(), "");
    }

    #[test]
    fn validate_url_handles_https() {
        let bar = AddressBar::new();
        let result = bar.validate_url("https://example.com");
        assert_eq!(result, UrlValidationResult::Valid("https://example.com".to_string()));
    }

    #[test]
    fn validate_url_handles_http() {
        let bar = AddressBar::new();
        let result = bar.validate_url("http://example.com");
        assert_eq!(result, UrlValidationResult::Valid("http://example.com".to_string()));
    }

    #[test]
    fn validate_url_handles_domain_only() {
        let bar = AddressBar::new();
        let result = bar.validate_url("example.com");
        assert_eq!(result, UrlValidationResult::Valid("https://example.com".to_string()));
    }

    #[test]
    fn validate_url_handles_search_query() {
        let bar = AddressBar::new();
        let result = bar.validate_url("hello world");
        assert_eq!(result, UrlValidationResult::SearchQuery("hello world".to_string()));
    }

    #[test]
    fn validate_url_rejects_empty() {
        let bar = AddressBar::new();
        let result = bar.validate_url("");
        assert!(matches!(result, UrlValidationResult::Invalid(_)));
    }

    #[test]
    fn validate_url_rejects_invalid_domain() {
        let bar = AddressBar::new();
        let result = bar.validate_url("https://");
        assert!(matches!(result, UrlValidationResult::Invalid(_)));
    }
}
