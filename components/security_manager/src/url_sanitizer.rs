//! Advanced URL Sanitization
//!
//! Enhanced URL sanitization with protection against:
//! - Dangerous URL schemes (javascript:, data:, vbscript:, etc.)
//! - IDN homograph attacks (punycode abuse)
//! - URL obfuscation techniques
//! - Protocol confusion attacks

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;
use url::Url;

/// Errors that can occur during URL sanitization
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UrlSanitizationError {
    /// URL parsing failed
    #[error("Invalid URL format: {0}")]
    InvalidFormat(String),

    /// Dangerous URL scheme detected
    #[error("Dangerous URL scheme '{scheme}': {reason}")]
    DangerousScheme { scheme: String, reason: String },

    /// IDN homograph attack detected
    #[error("Potential IDN homograph attack: {details}")]
    HomographAttack { details: String },

    /// Domain blocked by policy
    #[error("Domain blocked: {domain}")]
    BlockedDomain { domain: String },

    /// URL too long
    #[error("URL exceeds maximum length: {actual} > {max}")]
    TooLong { actual: usize, max: usize },

    /// Suspicious URL pattern
    #[error("Suspicious URL pattern: {reason}")]
    SuspiciousPattern { reason: String },
}

/// URL sanitization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlSanitizerConfig {
    /// Maximum URL length
    pub max_length: usize,

    /// Allowed URL schemes
    pub allowed_schemes: HashSet<String>,

    /// Blocked domains
    pub blocked_domains: HashSet<String>,

    /// Enable IDN homograph attack detection
    pub check_homograph: bool,

    /// Enable data URL content type validation
    pub validate_data_urls: bool,

    /// Maximum data URL size (in bytes)
    pub max_data_url_size: usize,

    /// Allowed data URL MIME types
    pub allowed_data_types: HashSet<String>,

    /// Block IP addresses as hostnames
    pub block_ip_addresses: bool,

    /// Enable URL obfuscation detection
    pub detect_obfuscation: bool,
}

impl Default for UrlSanitizerConfig {
    fn default() -> Self {
        let mut allowed_schemes = HashSet::new();
        allowed_schemes.insert("https".to_string());
        allowed_schemes.insert("http".to_string());
        allowed_schemes.insert("file".to_string());
        allowed_schemes.insert("about".to_string());
        allowed_schemes.insert("blob".to_string());
        allowed_schemes.insert("data".to_string()); // Allowed but validated

        let mut allowed_data_types = HashSet::new();
        allowed_data_types.insert("text/plain".to_string());
        allowed_data_types.insert("image/png".to_string());
        allowed_data_types.insert("image/jpeg".to_string());
        allowed_data_types.insert("image/gif".to_string());
        allowed_data_types.insert("image/svg+xml".to_string());

        Self {
            max_length: 2048,
            allowed_schemes,
            blocked_domains: HashSet::new(),
            check_homograph: true,
            validate_data_urls: true,
            max_data_url_size: 1024 * 1024, // 1 MB
            allowed_data_types,
            block_ip_addresses: false,
            detect_obfuscation: true,
        }
    }
}

/// Advanced URL sanitizer
pub struct UrlSanitizer {
    /// Configuration
    config: UrlSanitizerConfig,

    /// Dangerous schemes that should always be blocked
    dangerous_schemes: HashSet<String>,

    /// Characters commonly used in homograph attacks
    confusable_chars: HashSet<char>,
}

impl UrlSanitizer {
    /// Create a new URL sanitizer with default configuration
    pub fn new() -> Self {
        Self::with_config(UrlSanitizerConfig::default())
    }

    /// Create a URL sanitizer with custom configuration
    pub fn with_config(config: UrlSanitizerConfig) -> Self {
        let mut dangerous_schemes = HashSet::new();
        dangerous_schemes.insert("javascript".to_string());
        dangerous_schemes.insert("vbscript".to_string());
        dangerous_schemes.insert("data".to_string()); // Requires special handling
        dangerous_schemes.insert("file".to_string());  // Requires special handling
        dangerous_schemes.insert("jar".to_string());
        dangerous_schemes.insert("ms-its".to_string());
        dangerous_schemes.insert("mk".to_string());
        dangerous_schemes.insert("shell".to_string());
        dangerous_schemes.insert("wyciwyg".to_string());

        // Common confusable characters for IDN homograph attacks
        let mut confusable_chars = HashSet::new();
        // Cyrillic 'a' (U+0430)
        confusable_chars.insert('\u{0430}');
        // Cyrillic 'o' (U+043E)
        confusable_chars.insert('\u{043E}');
        // Cyrillic 'e' (U+0435)
        confusable_chars.insert('\u{0435}');
        // Cyrillic 'p' (U+0440)
        confusable_chars.insert('\u{0440}');
        // Cyrillic 'c' (U+0441)
        confusable_chars.insert('\u{0441}');
        // Cyrillic 'y' (U+0443)
        confusable_chars.insert('\u{0443}');
        // Cyrillic 'x' (U+0445)
        confusable_chars.insert('\u{0445}');

        Self {
            config,
            dangerous_schemes,
            confusable_chars,
        }
    }

    /// Sanitize and validate a URL
    pub fn sanitize(&self, url_str: &str) -> Result<String, UrlSanitizationError> {
        // Check length
        if url_str.len() > self.config.max_length {
            return Err(UrlSanitizationError::TooLong {
                actual: url_str.len(),
                max: self.config.max_length,
            });
        }

        // Detect obfuscation
        if self.config.detect_obfuscation {
            self.detect_obfuscation(url_str)?;
        }

        // Check for homograph attacks in the original URL string
        // (before URL parsing converts to punycode)
        if self.config.check_homograph {
            self.check_homograph_attack_in_url(url_str)?;
        }

        // Parse URL
        let url = Url::parse(url_str).map_err(|e| {
            UrlSanitizationError::InvalidFormat(format!("Parse error: {}", e))
        })?;

        // Check scheme
        let scheme = url.scheme().to_lowercase();
        self.validate_scheme(&scheme, url_str)?;

        // Check domain
        if let Some(host) = url.host_str() {
            self.validate_host(host)?;

            // Check blocked domains
            self.check_blocked_domain(host)?;
        }

        // For data URLs, validate content type and size
        if scheme == "data" && self.config.validate_data_urls {
            self.validate_data_url(url_str)?;
        }

        // Return sanitized URL (remove fragments for security)
        let mut sanitized = url.clone();
        sanitized.set_fragment(None);

        Ok(sanitized.to_string())
    }

    /// Validate URL scheme
    fn validate_scheme(&self, scheme: &str, _url_str: &str) -> Result<(), UrlSanitizationError> {
        // Check if it's a dangerous scheme
        if self.dangerous_schemes.contains(scheme) {
            // Special handling for data: and file: schemes
            match scheme {
                "data" => {
                    if !self.config.validate_data_urls {
                        return Err(UrlSanitizationError::DangerousScheme {
                            scheme: scheme.to_string(),
                            reason: "data: URLs are disabled".to_string(),
                        });
                    }
                    // Will be validated separately
                }
                "file" => {
                    if !self.config.allowed_schemes.contains("file") {
                        return Err(UrlSanitizationError::DangerousScheme {
                            scheme: scheme.to_string(),
                            reason: "file: URLs are not allowed".to_string(),
                        });
                    }
                }
                _ => {
                    return Err(UrlSanitizationError::DangerousScheme {
                        scheme: scheme.to_string(),
                        reason: format!("Scheme '{}' is inherently dangerous", scheme),
                    });
                }
            }
        }

        // Check if scheme is in allowed list
        if !self.config.allowed_schemes.contains(scheme) {
            return Err(UrlSanitizationError::DangerousScheme {
                scheme: scheme.to_string(),
                reason: "Scheme not in allowed list".to_string(),
            });
        }

        Ok(())
    }

    /// Validate hostname
    fn validate_host(&self, host: &str) -> Result<(), UrlSanitizationError> {
        // Check if it's an IP address and blocking is enabled
        if self.config.block_ip_addresses {
            if host.parse::<std::net::IpAddr>().is_ok() {
                return Err(UrlSanitizationError::SuspiciousPattern {
                    reason: "IP addresses as hostnames are blocked".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Check for IDN homograph attacks in the original URL string
    fn check_homograph_attack_in_url(&self, url_str: &str) -> Result<(), UrlSanitizationError> {
        // Extract the domain part from the URL string
        // Look for "://" and then extract until the next '/', ':', '?', or '#'
        if let Some(scheme_end) = url_str.find("://") {
            let after_scheme = &url_str[scheme_end + 3..];
            let domain_end = after_scheme
                .find(|c: char| c == '/' || c == ':' || c == '?' || c == '#')
                .unwrap_or(after_scheme.len());
            let domain = &after_scheme[..domain_end];

            self.check_homograph_attack(domain)?;
        }

        Ok(())
    }

    /// Check for IDN homograph attacks
    fn check_homograph_attack(&self, host: &str) -> Result<(), UrlSanitizationError> {
        // Check if domain contains confusable characters
        for ch in host.chars() {
            if self.confusable_chars.contains(&ch) {
                return Err(UrlSanitizationError::HomographAttack {
                    details: format!(
                        "Domain contains confusable character '{}' (U+{:04X})",
                        ch, ch as u32
                    ),
                });
            }
        }

        // Check for mixed scripts (potential homograph attack indicator)
        if self.has_mixed_scripts(host) {
            return Err(UrlSanitizationError::HomographAttack {
                details: "Domain mixes different character scripts".to_string(),
            });
        }

        Ok(())
    }

    /// Check if domain has mixed character scripts
    fn has_mixed_scripts(&self, domain: &str) -> bool {
        let mut has_latin = false;
        let mut has_cyrillic = false;
        let mut has_greek = false;

        for ch in domain.chars() {
            if ch.is_ascii_alphabetic() {
                has_latin = true;
            } else if ('\u{0400}'..='\u{04FF}').contains(&ch) {
                // Cyrillic
                has_cyrillic = true;
            } else if ('\u{0370}'..='\u{03FF}').contains(&ch) {
                // Greek
                has_greek = true;
            }
        }

        // If multiple scripts are present, it's suspicious
        let script_count = [has_latin, has_cyrillic, has_greek]
            .iter()
            .filter(|&&x| x)
            .count();

        script_count > 1
    }

    /// Check if domain is blocked
    fn check_blocked_domain(&self, host: &str) -> Result<(), UrlSanitizationError> {
        let host_lower = host.to_lowercase();

        for blocked in &self.config.blocked_domains {
            if host_lower.ends_with(blocked) || host_lower == *blocked {
                return Err(UrlSanitizationError::BlockedDomain {
                    domain: host.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate data URL
    fn validate_data_url(&self, url_str: &str) -> Result<(), UrlSanitizationError> {
        // Check size
        if url_str.len() > self.config.max_data_url_size {
            return Err(UrlSanitizationError::TooLong {
                actual: url_str.len(),
                max: self.config.max_data_url_size,
            });
        }

        // Parse data URL format: data:[<mediatype>][;base64],<data>
        if let Some(content) = url_str.strip_prefix("data:") {
            let parts: Vec<&str> = content.splitn(2, ',').collect();
            if parts.len() == 2 {
                let metadata = parts[0];

                // Extract MIME type
                let mime_type = if let Some(semicolon_pos) = metadata.find(';') {
                    &metadata[..semicolon_pos]
                } else {
                    metadata
                };

                // Default to text/plain if empty
                let mime_type = if mime_type.is_empty() {
                    "text/plain"
                } else {
                    mime_type
                };

                // Check if MIME type is allowed
                if !self.config.allowed_data_types.contains(mime_type) {
                    return Err(UrlSanitizationError::DangerousScheme {
                        scheme: "data".to_string(),
                        reason: format!("MIME type '{}' not allowed for data URLs", mime_type),
                    });
                }

                // Check for dangerous content in data URLs
                if mime_type.starts_with("text/html") || mime_type.starts_with("application/javascript") {
                    return Err(UrlSanitizationError::DangerousScheme {
                        scheme: "data".to_string(),
                        reason: "HTML and JavaScript data URLs are not allowed".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Detect URL obfuscation techniques
    fn detect_obfuscation(&self, url_str: &str) -> Result<(), UrlSanitizationError> {
        // Check for excessive URL encoding
        let percent_count = url_str.chars().filter(|&c| c == '%').count();
        if percent_count > 10 {
            return Err(UrlSanitizationError::SuspiciousPattern {
                reason: "Excessive URL encoding detected".to_string(),
            });
        }

        // Check for null bytes
        if url_str.contains('\0') {
            return Err(UrlSanitizationError::SuspiciousPattern {
                reason: "Null byte in URL".to_string(),
            });
        }

        // Check for unusual characters
        if url_str.contains('\r') || url_str.contains('\n') {
            return Err(UrlSanitizationError::SuspiciousPattern {
                reason: "Line breaks in URL".to_string(),
            });
        }

        Ok(())
    }

    /// Add a domain to the blocklist
    pub fn block_domain(&mut self, domain: String) {
        self.config.blocked_domains.insert(domain);
    }

    /// Remove a domain from the blocklist
    pub fn unblock_domain(&mut self, domain: &str) {
        self.config.blocked_domains.remove(domain);
    }

    /// Get the current configuration
    pub fn config(&self) -> &UrlSanitizerConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: UrlSanitizerConfig) {
        self.config = config;
    }
}

impl Default for UrlSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_https_url() {
        let sanitizer = UrlSanitizer::new();
        let result = sanitizer.sanitize("https://example.com/path");
        assert!(result.is_ok());
    }

    #[test]
    fn test_javascript_url_blocked() {
        let sanitizer = UrlSanitizer::new();
        let result = sanitizer.sanitize("javascript:alert(1)");
        assert!(matches!(
            result,
            Err(UrlSanitizationError::DangerousScheme { .. })
        ));
    }

    #[test]
    fn test_vbscript_url_blocked() {
        let sanitizer = UrlSanitizer::new();
        let result = sanitizer.sanitize("vbscript:msgbox(1)");
        assert!(matches!(
            result,
            Err(UrlSanitizationError::DangerousScheme { .. })
        ));
    }

    #[test]
    fn test_data_url_with_allowed_type() {
        let sanitizer = UrlSanitizer::new();
        let result = sanitizer.sanitize("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUA");
        assert!(result.is_ok());
    }

    #[test]
    fn test_data_url_with_dangerous_type() {
        let sanitizer = UrlSanitizer::new();
        let result = sanitizer.sanitize("data:text/html,<script>alert(1)</script>");
        assert!(matches!(
            result,
            Err(UrlSanitizationError::DangerousScheme { .. })
        ));
    }

    #[test]
    fn test_url_too_long() {
        let sanitizer = UrlSanitizer::new();
        let long_url = format!("https://example.com/{}", "a".repeat(3000));
        let result = sanitizer.sanitize(&long_url);
        assert!(matches!(result, Err(UrlSanitizationError::TooLong { .. })));
    }

    #[test]
    fn test_blocked_domain() {
        let mut sanitizer = UrlSanitizer::new();
        sanitizer.block_domain("malicious.com".to_string());

        let result = sanitizer.sanitize("https://malicious.com/page");
        assert!(matches!(
            result,
            Err(UrlSanitizationError::BlockedDomain { .. })
        ));
    }

    #[test]
    fn test_homograph_attack_cyrillic() {
        let sanitizer = UrlSanitizer::new();
        // URL with Cyrillic 'a' (U+0430) that looks like Latin 'a'
        let result = sanitizer.sanitize("https://ex\u{0430}mple.com");
        assert!(matches!(
            result,
            Err(UrlSanitizationError::HomographAttack { .. })
        ));
    }

    #[test]
    fn test_mixed_script_detection() {
        let sanitizer = UrlSanitizer::new();
        assert!(sanitizer.has_mixed_scripts("exam\u{0430}ple")); // Latin + Cyrillic
        assert!(!sanitizer.has_mixed_scripts("example")); // All Latin
    }

    #[test]
    fn test_excessive_url_encoding() {
        let sanitizer = UrlSanitizer::new();
        let encoded_url = "https://example.com/%00%01%02%03%04%05%06%07%08%09%0A%0B";
        let result = sanitizer.sanitize(encoded_url);
        assert!(matches!(
            result,
            Err(UrlSanitizationError::SuspiciousPattern { .. })
        ));
    }

    #[test]
    fn test_null_byte_in_url() {
        let sanitizer = UrlSanitizer::new();
        let result = sanitizer.sanitize("https://example.com/path\0malicious");
        assert!(matches!(
            result,
            Err(UrlSanitizationError::SuspiciousPattern { .. })
        ));
    }

    #[test]
    fn test_line_breaks_in_url() {
        let sanitizer = UrlSanitizer::new();
        let result = sanitizer.sanitize("https://example.com/path\nmalicious");
        assert!(matches!(
            result,
            Err(UrlSanitizationError::SuspiciousPattern { .. })
        ));
    }

    #[test]
    fn test_fragment_removed() {
        let sanitizer = UrlSanitizer::new();
        let result = sanitizer.sanitize("https://example.com/path#fragment");
        assert!(result.is_ok());
        assert!(!result.unwrap().contains("#fragment"));
    }

    #[test]
    fn test_ip_address_blocking() {
        let mut config = UrlSanitizerConfig::default();
        config.block_ip_addresses = true;

        let sanitizer = UrlSanitizer::with_config(config);
        let result = sanitizer.sanitize("https://192.168.1.1/path");
        assert!(matches!(
            result,
            Err(UrlSanitizationError::SuspiciousPattern { .. })
        ));
    }

    #[test]
    fn test_unblock_domain() {
        let mut sanitizer = UrlSanitizer::new();
        sanitizer.block_domain("test.com".to_string());

        let result = sanitizer.sanitize("https://test.com");
        assert!(result.is_err());

        sanitizer.unblock_domain("test.com");
        let result = sanitizer.sanitize("https://test.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_data_url_size_limit() {
        let mut config = UrlSanitizerConfig::default();
        config.max_data_url_size = 50;

        let sanitizer = UrlSanitizer::with_config(config);
        let large_data_url = format!(
            "data:text/plain,{}",
            "a".repeat(100)
        );

        let result = sanitizer.sanitize(&large_data_url);
        assert!(matches!(result, Err(UrlSanitizationError::TooLong { .. })));
    }

    #[test]
    fn test_custom_allowed_schemes() {
        let mut config = UrlSanitizerConfig::default();
        config.allowed_schemes.clear();
        config.allowed_schemes.insert("https".to_string());

        let sanitizer = UrlSanitizer::with_config(config);

        // HTTPS should work
        assert!(sanitizer.sanitize("https://example.com").is_ok());

        // HTTP should not work
        assert!(sanitizer.sanitize("http://example.com").is_err());
    }

    #[test]
    fn test_confusable_characters() {
        let sanitizer = UrlSanitizer::new();

        // Test various confusable Cyrillic characters
        assert!(sanitizer.sanitize("https://ex\u{0430}mple.com").is_err()); // Cyrillic 'a'
        assert!(sanitizer.sanitize("https://g\u{043E}ogle.com").is_err()); // Cyrillic 'o'
        assert!(sanitizer.sanitize("https://\u{0441}at.com").is_err()); // Cyrillic 'c'
    }

    #[test]
    fn test_data_url_javascript_blocked() {
        let sanitizer = UrlSanitizer::new();
        let result = sanitizer.sanitize("data:application/javascript,alert(1)");
        assert!(matches!(
            result,
            Err(UrlSanitizationError::DangerousScheme { .. })
        ));
    }
}
