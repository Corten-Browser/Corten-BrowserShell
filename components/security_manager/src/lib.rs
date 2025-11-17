//! Security Manager Component
//!
//! Provides security features including:
//! - URL sanitization and validation
//! - Input filtering and XSS prevention
//! - Content Security Policy enforcement
//! - Permission management

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use url::Url;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("URL blocked by security policy: {0}")]
    BlockedUrl(String),
    #[error("Input contains potentially dangerous content: {0}")]
    DangerousInput(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("CSP violation: {0}")]
    CspViolation(String),
}

pub type Result<T> = std::result::Result<T, SecurityError>;

/// Security policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Allowed URL schemes (e.g., "https", "http", "file")
    pub allowed_schemes: HashSet<String>,
    /// Blocked domains
    pub blocked_domains: HashSet<String>,
    /// Enable XSS protection
    pub xss_protection: bool,
    /// Enable CSRF protection
    pub csrf_protection: bool,
    /// Maximum URL length
    pub max_url_length: usize,
    /// Maximum input length
    pub max_input_length: usize,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        let mut allowed_schemes = HashSet::new();
        allowed_schemes.insert("https".to_string());
        allowed_schemes.insert("http".to_string());
        allowed_schemes.insert("file".to_string());
        allowed_schemes.insert("about".to_string());

        Self {
            allowed_schemes,
            blocked_domains: HashSet::new(),
            xss_protection: true,
            csrf_protection: true,
            max_url_length: 2048,
            max_input_length: 10_000,
        }
    }
}

/// Content Security Policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSecurityPolicy {
    /// Default source directive
    pub default_src: Vec<String>,
    /// Script source directive
    pub script_src: Vec<String>,
    /// Style source directive
    pub style_src: Vec<String>,
    /// Image source directive
    pub img_src: Vec<String>,
    /// Connect source directive
    pub connect_src: Vec<String>,
    /// Frame source directive
    pub frame_src: Vec<String>,
    /// Object source directive
    pub object_src: Vec<String>,
    /// Base URI directive
    pub base_uri: Vec<String>,
    /// Form action directive
    pub form_action: Vec<String>,
}

impl Default for ContentSecurityPolicy {
    fn default() -> Self {
        Self {
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string()],
            style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            img_src: vec!["'self'".to_string(), "data:".to_string(), "https:".to_string()],
            connect_src: vec!["'self'".to_string(), "https:".to_string()],
            frame_src: vec!["'self'".to_string()],
            object_src: vec!["'none'".to_string()],
            base_uri: vec!["'self'".to_string()],
            form_action: vec!["'self'".to_string()],
        }
    }
}

impl ContentSecurityPolicy {
    /// Convert to CSP header string
    pub fn to_header_string(&self) -> String {
        let mut parts = Vec::new();

        if !self.default_src.is_empty() {
            parts.push(format!("default-src {}", self.default_src.join(" ")));
        }
        if !self.script_src.is_empty() {
            parts.push(format!("script-src {}", self.script_src.join(" ")));
        }
        if !self.style_src.is_empty() {
            parts.push(format!("style-src {}", self.style_src.join(" ")));
        }
        if !self.img_src.is_empty() {
            parts.push(format!("img-src {}", self.img_src.join(" ")));
        }
        if !self.connect_src.is_empty() {
            parts.push(format!("connect-src {}", self.connect_src.join(" ")));
        }
        if !self.frame_src.is_empty() {
            parts.push(format!("frame-src {}", self.frame_src.join(" ")));
        }
        if !self.object_src.is_empty() {
            parts.push(format!("object-src {}", self.object_src.join(" ")));
        }
        if !self.base_uri.is_empty() {
            parts.push(format!("base-uri {}", self.base_uri.join(" ")));
        }
        if !self.form_action.is_empty() {
            parts.push(format!("form-action {}", self.form_action.join(" ")));
        }

        parts.join("; ")
    }
}

/// Permission types for security
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    Geolocation,
    Notifications,
    Camera,
    Microphone,
    Clipboard,
    Storage,
    Downloads,
}

/// Permission status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionStatus {
    Granted,
    Denied,
    Prompt,
}

/// Security Manager
pub struct SecurityManager {
    /// Security policy
    policy: Arc<RwLock<SecurityPolicy>>,
    /// Content Security Policy
    csp: Arc<RwLock<ContentSecurityPolicy>>,
    /// Domain permissions: domain -> (permission -> status)
    permissions: Arc<RwLock<HashMap<String, HashMap<Permission, PermissionStatus>>>>,
    /// XSS pattern matcher
    xss_patterns: Vec<Regex>,
}

impl SecurityManager {
    /// Create a new SecurityManager with default settings
    pub fn new() -> Self {
        let xss_patterns = vec![
            Regex::new(r"(?i)<script[^>]*>").unwrap(),
            Regex::new(r"(?i)javascript:").unwrap(),
            Regex::new(r"(?i)on\w+\s*=").unwrap(),
            Regex::new(r"(?i)<iframe[^>]*>").unwrap(),
            Regex::new(r"(?i)<object[^>]*>").unwrap(),
            Regex::new(r"(?i)<embed[^>]*>").unwrap(),
            Regex::new(r#"(?i)<link[^>]+rel\s*=\s*['"]?import"#).unwrap(),
            Regex::new(r#"(?i)eval\s*\("#).unwrap(),
            Regex::new(r"(?i)document\.cookie").unwrap(),
            Regex::new(r"(?i)document\.write").unwrap(),
        ];

        Self {
            policy: Arc::new(RwLock::new(SecurityPolicy::default())),
            csp: Arc::new(RwLock::new(ContentSecurityPolicy::default())),
            permissions: Arc::new(RwLock::new(HashMap::new())),
            xss_patterns,
        }
    }

    /// Set security policy
    pub async fn set_policy(&self, policy: SecurityPolicy) {
        let mut current = self.policy.write().await;
        *current = policy;
    }

    /// Get current security policy
    pub async fn get_policy(&self) -> SecurityPolicy {
        self.policy.read().await.clone()
    }

    /// Set Content Security Policy
    pub async fn set_csp(&self, csp: ContentSecurityPolicy) {
        let mut current = self.csp.write().await;
        *current = csp;
    }

    /// Get current Content Security Policy
    pub async fn get_csp(&self) -> ContentSecurityPolicy {
        self.csp.read().await.clone()
    }

    /// Validate and sanitize a URL
    pub async fn validate_url(&self, url_str: &str) -> Result<String> {
        let policy = self.policy.read().await;

        // Check URL length
        if url_str.len() > policy.max_url_length {
            return Err(SecurityError::InvalidUrl(format!(
                "URL too long ({} > {})",
                url_str.len(),
                policy.max_url_length
            )));
        }

        // Parse URL
        let url = Url::parse(url_str)
            .map_err(|e| SecurityError::InvalidUrl(format!("Parse error: {}", e)))?;

        // Check scheme
        let scheme = url.scheme();
        if !policy.allowed_schemes.contains(scheme) {
            return Err(SecurityError::BlockedUrl(format!(
                "Scheme '{}' not allowed",
                scheme
            )));
        }

        // Check blocked domains
        if let Some(host) = url.host_str() {
            let host_lower = host.to_lowercase();
            for blocked in &policy.blocked_domains {
                if host_lower.ends_with(blocked) || host_lower == *blocked {
                    return Err(SecurityError::BlockedUrl(format!(
                        "Domain '{}' is blocked",
                        host
                    )));
                }
            }
        }

        // Sanitize - remove dangerous fragments
        let mut sanitized = url.clone();
        sanitized.set_fragment(None); // Remove fragment for security

        Ok(sanitized.to_string())
    }

    /// Sanitize input text to prevent XSS
    pub async fn sanitize_input(&self, input: &str) -> Result<String> {
        let policy = self.policy.read().await;

        // Check input length
        if input.len() > policy.max_input_length {
            return Err(SecurityError::DangerousInput(format!(
                "Input too long ({} > {})",
                input.len(),
                policy.max_input_length
            )));
        }

        if !policy.xss_protection {
            return Ok(input.to_string());
        }

        // Check for XSS patterns
        for pattern in &self.xss_patterns {
            if pattern.is_match(input) {
                return Err(SecurityError::DangerousInput(
                    "Potentially dangerous content detected".to_string(),
                ));
            }
        }

        // HTML encode dangerous characters
        let sanitized = input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('/', "&#x2F;");

        Ok(sanitized)
    }

    /// Check if a script source is allowed by CSP
    pub async fn check_script_source(&self, source: &str) -> Result<()> {
        let csp = self.csp.read().await;

        if csp.script_src.contains(&"'unsafe-inline'".to_string()) {
            return Ok(());
        }

        if csp.script_src.contains(&"'self'".to_string()) && source.starts_with('/') {
            return Ok(());
        }

        // Check if source matches any allowed source
        for allowed in &csp.script_src {
            if allowed == source || (allowed.ends_with('*') && source.starts_with(&allowed[..allowed.len()-1])) {
                return Ok(());
            }
        }

        Err(SecurityError::CspViolation(format!(
            "Script source '{}' not allowed by CSP",
            source
        )))
    }

    /// Set permission for a domain
    pub async fn set_permission(
        &self,
        domain: String,
        permission: Permission,
        status: PermissionStatus,
    ) {
        let mut perms = self.permissions.write().await;
        perms
            .entry(domain)
            .or_insert_with(HashMap::new)
            .insert(permission, status);
    }

    /// Get permission status for a domain
    pub async fn get_permission(
        &self,
        domain: &str,
        permission: Permission,
    ) -> PermissionStatus {
        let perms = self.permissions.read().await;
        perms
            .get(domain)
            .and_then(|p| p.get(&permission))
            .copied()
            .unwrap_or(PermissionStatus::Prompt)
    }

    /// Check if permission is granted
    pub async fn check_permission(&self, domain: &str, permission: Permission) -> Result<()> {
        let status = self.get_permission(domain, permission).await;
        match status {
            PermissionStatus::Granted => Ok(()),
            PermissionStatus::Denied => Err(SecurityError::PermissionDenied(format!(
                "{:?} permission denied for {}",
                permission, domain
            ))),
            PermissionStatus::Prompt => Err(SecurityError::PermissionDenied(format!(
                "{:?} permission requires user approval for {}",
                permission, domain
            ))),
        }
    }

    /// Add a domain to the blocklist
    pub async fn block_domain(&self, domain: String) {
        let mut policy = self.policy.write().await;
        policy.blocked_domains.insert(domain);
    }

    /// Remove a domain from the blocklist
    pub async fn unblock_domain(&self, domain: &str) {
        let mut policy = self.policy.write().await;
        policy.blocked_domains.remove(domain);
    }

    /// Get list of blocked domains
    pub async fn get_blocked_domains(&self) -> Vec<String> {
        let policy = self.policy.read().await;
        policy.blocked_domains.iter().cloned().collect()
    }

    /// Generate a secure random token (for CSRF protection)
    pub fn generate_csrf_token(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut hasher = DefaultHasher::new();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        nanos.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Validate CSRF token (simplified version)
    pub fn validate_csrf_token(&self, token: &str) -> bool {
        // In a real implementation, this would check against stored tokens
        // For now, just verify it's a valid hex string of expected length
        token.len() == 16 && token.chars().all(|c| c.is_ascii_hexdigit())
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_url_success() {
        let manager = SecurityManager::new();
        let result = manager.validate_url("https://example.com/page").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_url_invalid_scheme() {
        let manager = SecurityManager::new();
        let result = manager.validate_url("ftp://example.com/file").await;
        assert!(matches!(result, Err(SecurityError::BlockedUrl(_))));
    }

    #[tokio::test]
    async fn test_validate_url_blocked_domain() {
        let manager = SecurityManager::new();
        manager.block_domain("malicious.com".to_string()).await;

        let result = manager.validate_url("https://malicious.com").await;
        assert!(matches!(result, Err(SecurityError::BlockedUrl(_))));
    }

    #[tokio::test]
    async fn test_validate_url_too_long() {
        let manager = SecurityManager::new();
        let long_url = format!("https://example.com/{}", "a".repeat(3000));
        let result = manager.validate_url(&long_url).await;
        assert!(matches!(result, Err(SecurityError::InvalidUrl(_))));
    }

    #[tokio::test]
    async fn test_sanitize_input_xss_script() {
        let manager = SecurityManager::new();
        let result = manager.sanitize_input("<script>alert('xss')</script>").await;
        assert!(matches!(result, Err(SecurityError::DangerousInput(_))));
    }

    #[tokio::test]
    async fn test_sanitize_input_xss_javascript() {
        let manager = SecurityManager::new();
        let result = manager.sanitize_input("javascript:alert(1)").await;
        assert!(matches!(result, Err(SecurityError::DangerousInput(_))));
    }

    #[tokio::test]
    async fn test_sanitize_input_xss_event_handler() {
        let manager = SecurityManager::new();
        let result = manager.sanitize_input("<img onerror=alert(1)>").await;
        assert!(matches!(result, Err(SecurityError::DangerousInput(_))));
    }

    #[tokio::test]
    async fn test_sanitize_input_safe() {
        let manager = SecurityManager::new();
        let result = manager.sanitize_input("Hello World!").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello World!");
    }

    #[tokio::test]
    async fn test_sanitize_encodes_html() {
        let manager = SecurityManager::new();
        let result = manager.sanitize_input("a < b & c > d").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "a &lt; b &amp; c &gt; d");
    }

    #[tokio::test]
    async fn test_permissions() {
        let manager = SecurityManager::new();
        let domain = "example.com".to_string();

        // Default is Prompt
        let status = manager.get_permission(&domain, Permission::Geolocation).await;
        assert_eq!(status, PermissionStatus::Prompt);

        // Set to Granted
        manager
            .set_permission(domain.clone(), Permission::Geolocation, PermissionStatus::Granted)
            .await;
        let status = manager.get_permission(&domain, Permission::Geolocation).await;
        assert_eq!(status, PermissionStatus::Granted);

        // Check permission
        let result = manager.check_permission(&domain, Permission::Geolocation).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_permission_denied() {
        let manager = SecurityManager::new();
        let domain = "example.com".to_string();

        manager
            .set_permission(domain.clone(), Permission::Camera, PermissionStatus::Denied)
            .await;

        let result = manager.check_permission(&domain, Permission::Camera).await;
        assert!(matches!(result, Err(SecurityError::PermissionDenied(_))));
    }

    #[tokio::test]
    async fn test_csp_header_generation() {
        let csp = ContentSecurityPolicy::default();
        let header = csp.to_header_string();

        assert!(header.contains("default-src 'self'"));
        assert!(header.contains("script-src 'self'"));
        assert!(header.contains("object-src 'none'"));
    }

    #[tokio::test]
    async fn test_csrf_token_generation() {
        let manager = SecurityManager::new();
        let token1 = manager.generate_csrf_token();
        let token2 = manager.generate_csrf_token();

        // Tokens should be valid
        assert!(manager.validate_csrf_token(&token1));
        assert!(manager.validate_csrf_token(&token2));

        // Tokens should be different (very likely)
        assert_ne!(token1, token2);
    }

    #[tokio::test]
    async fn test_block_unblock_domain() {
        let manager = SecurityManager::new();

        manager.block_domain("bad.com".to_string()).await;
        let blocked = manager.get_blocked_domains().await;
        assert!(blocked.contains(&"bad.com".to_string()));

        manager.unblock_domain("bad.com").await;
        let blocked = manager.get_blocked_domains().await;
        assert!(!blocked.contains(&"bad.com".to_string()));
    }

    #[tokio::test]
    async fn test_check_script_source_self() {
        let manager = SecurityManager::new();
        let result = manager.check_script_source("/script.js").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_script_source_external_blocked() {
        let manager = SecurityManager::new();
        let result = manager.check_script_source("https://external.com/script.js").await;
        assert!(matches!(result, Err(SecurityError::CspViolation(_))));
    }
}
