//! Privacy management module for CortenBrowser.
//!
//! Provides privacy protection features including:
//! - Do Not Track (DNT) header management
//! - Third-party cookie blocking
//! - Tracking protection
//! - Privacy settings management

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

/// Cookie policy for privacy settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CookiePolicy {
    /// Accept all cookies.
    AcceptAll,
    /// Block third-party cookies only.
    BlockThirdParty,
    /// Block all cookies.
    BlockAll,
    /// Custom cookie rules.
    Custom(CookieRules),
}

impl Default for CookiePolicy {
    fn default() -> Self {
        CookiePolicy::BlockThirdParty
    }
}

/// Custom cookie rules for fine-grained control.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CookieRules {
    /// Domains that are always allowed to set cookies.
    pub allowed_domains: HashSet<String>,
    /// Domains that are always blocked from setting cookies.
    pub blocked_domains: HashSet<String>,
    /// Whether to block third-party cookies by default.
    pub block_third_party_default: bool,
    /// Whether to block session cookies.
    pub block_session_cookies: bool,
    /// Whether to block persistent cookies.
    pub block_persistent_cookies: bool,
}

impl Default for CookieRules {
    fn default() -> Self {
        Self {
            allowed_domains: HashSet::new(),
            blocked_domains: HashSet::new(),
            block_third_party_default: true,
            block_session_cookies: false,
            block_persistent_cookies: false,
        }
    }
}

/// Privacy settings configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Whether Do Not Track header is enabled.
    pub dnt_enabled: bool,
    /// Cookie policy.
    pub cookie_policy: CookiePolicy,
    /// Whether tracking protection is enabled.
    pub tracking_protection: bool,
    /// Whether to send Global Privacy Control header.
    pub gpc_enabled: bool,
    /// Whether to block fingerprinting attempts.
    pub fingerprint_protection: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            dnt_enabled: true,
            cookie_policy: CookiePolicy::BlockThirdParty,
            tracking_protection: true,
            gpc_enabled: true,
            fingerprint_protection: false,
        }
    }
}

/// A cookie representation for privacy checking.
#[derive(Debug, Clone)]
pub struct Cookie {
    /// Cookie name.
    pub name: String,
    /// Cookie value.
    pub value: String,
    /// Domain the cookie applies to.
    pub domain: Option<String>,
    /// Path the cookie applies to.
    pub path: Option<String>,
    /// Whether this is a session cookie (no expiry).
    pub is_session: bool,
    /// Whether the cookie is secure-only.
    pub secure: bool,
    /// Whether the cookie is HTTP-only.
    pub http_only: bool,
}

impl Cookie {
    /// Create a new cookie.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            domain: None,
            path: None,
            is_session: true,
            secure: false,
            http_only: false,
        }
    }

    /// Set the domain.
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Set the path.
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Mark as persistent (non-session) cookie.
    pub fn persistent(mut self) -> Self {
        self.is_session = false;
        self
    }

    /// Mark as secure-only.
    pub fn secure(mut self) -> Self {
        self.secure = true;
        self
    }

    /// Mark as HTTP-only.
    pub fn http_only(mut self) -> Self {
        self.http_only = true;
        self
    }

    /// Get the effective domain for this cookie.
    pub fn effective_domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }
}

/// Result of a cookie blocking decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CookieDecision {
    /// Allow the cookie.
    Allow,
    /// Block the cookie.
    Block(BlockReason),
}

/// Reason why a cookie was blocked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockReason {
    /// Cookie is from a third-party domain.
    ThirdParty,
    /// Cookie domain is in the blocklist.
    BlockedDomain,
    /// All cookies are blocked by policy.
    PolicyBlockAll,
    /// Session cookies are blocked.
    SessionCookieBlocked,
    /// Persistent cookies are blocked.
    PersistentCookieBlocked,
    /// Cookie is from a known tracker.
    KnownTracker,
}

/// Known tracking domains list.
#[derive(Debug, Clone, Default)]
pub struct TrackingProtectionList {
    /// Known tracker domains.
    tracker_domains: HashSet<String>,
    /// Known fingerprinting domains.
    fingerprint_domains: HashSet<String>,
    /// Known cryptominer domains.
    cryptominer_domains: HashSet<String>,
}

impl TrackingProtectionList {
    /// Create a new tracking protection list with default entries.
    pub fn new() -> Self {
        let mut list = Self::default();
        // Add common tracking domains
        list.add_tracker("doubleclick.net");
        list.add_tracker("googlesyndication.com");
        list.add_tracker("googleadservices.com");
        list.add_tracker("facebook.net");
        list.add_tracker("fbcdn.net");
        list.add_tracker("analytics.google.com");
        list.add_tracker("google-analytics.com");
        list.add_tracker("adnxs.com");
        list.add_tracker("adsrvr.org");
        list.add_tracker("criteo.com");
        list.add_tracker("scorecardresearch.com");
        list.add_tracker("quantserve.com");
        list.add_tracker("outbrain.com");
        list.add_tracker("taboola.com");
        list
    }

    /// Add a tracker domain.
    pub fn add_tracker(&mut self, domain: impl Into<String>) {
        self.tracker_domains.insert(domain.into());
    }

    /// Add a fingerprinting domain.
    pub fn add_fingerprinter(&mut self, domain: impl Into<String>) {
        self.fingerprint_domains.insert(domain.into());
    }

    /// Add a cryptominer domain.
    pub fn add_cryptominer(&mut self, domain: impl Into<String>) {
        self.cryptominer_domains.insert(domain.into());
    }

    /// Check if a domain is a known tracker.
    pub fn is_tracker(&self, domain: &str) -> bool {
        let domain_lower = domain.to_lowercase();
        self.tracker_domains
            .iter()
            .any(|t| domain_lower == *t || domain_lower.ends_with(&format!(".{}", t)))
    }

    /// Check if a domain is a known fingerprinter.
    pub fn is_fingerprinter(&self, domain: &str) -> bool {
        let domain_lower = domain.to_lowercase();
        self.fingerprint_domains
            .iter()
            .any(|f| domain_lower == *f || domain_lower.ends_with(&format!(".{}", f)))
    }

    /// Check if a domain is a known cryptominer.
    pub fn is_cryptominer(&self, domain: &str) -> bool {
        let domain_lower = domain.to_lowercase();
        self.cryptominer_domains
            .iter()
            .any(|c| domain_lower == *c || domain_lower.ends_with(&format!(".{}", c)))
    }

    /// Get total number of entries in the list.
    pub fn len(&self) -> usize {
        self.tracker_domains.len()
            + self.fingerprint_domains.len()
            + self.cryptominer_domains.len()
    }

    /// Check if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Privacy manager for handling privacy-related decisions.
pub struct PrivacyManager {
    /// Privacy settings.
    settings: Arc<RwLock<PrivacySettings>>,
    /// Tracking protection list.
    tracking_list: Arc<RwLock<TrackingProtectionList>>,
}

impl PrivacyManager {
    /// Create a new privacy manager with default settings.
    pub fn new() -> Self {
        Self {
            settings: Arc::new(RwLock::new(PrivacySettings::default())),
            tracking_list: Arc::new(RwLock::new(TrackingProtectionList::new())),
        }
    }

    /// Create a new privacy manager with custom settings.
    pub fn with_settings(settings: PrivacySettings) -> Self {
        Self {
            settings: Arc::new(RwLock::new(settings)),
            tracking_list: Arc::new(RwLock::new(TrackingProtectionList::new())),
        }
    }

    /// Get the current privacy settings.
    pub async fn get_settings(&self) -> PrivacySettings {
        self.settings.read().await.clone()
    }

    /// Update privacy settings.
    pub async fn update_settings(&self, settings: PrivacySettings) {
        let mut current = self.settings.write().await;
        *current = settings;
    }

    /// Check if DNT header should be sent.
    pub async fn should_send_dnt(&self) -> bool {
        self.settings.read().await.dnt_enabled
    }

    /// Enable or disable DNT.
    pub async fn set_dnt_enabled(&self, enabled: bool) {
        self.settings.write().await.dnt_enabled = enabled;
    }

    /// Check if GPC header should be sent.
    pub async fn should_send_gpc(&self) -> bool {
        self.settings.read().await.gpc_enabled
    }

    /// Enable or disable GPC.
    pub async fn set_gpc_enabled(&self, enabled: bool) {
        self.settings.write().await.gpc_enabled = enabled;
    }

    /// Get the current cookie policy.
    pub async fn get_cookie_policy(&self) -> CookiePolicy {
        self.settings.read().await.cookie_policy.clone()
    }

    /// Set the cookie policy.
    pub async fn set_cookie_policy(&self, policy: CookiePolicy) {
        self.settings.write().await.cookie_policy = policy;
    }

    /// Check if a cookie should be blocked.
    pub async fn should_block_cookie(&self, cookie: &Cookie, request_url: &Url) -> CookieDecision {
        let settings = self.settings.read().await;

        match &settings.cookie_policy {
            CookiePolicy::AcceptAll => CookieDecision::Allow,
            CookiePolicy::BlockAll => CookieDecision::Block(BlockReason::PolicyBlockAll),
            CookiePolicy::BlockThirdParty => {
                let cookie_domain = cookie.effective_domain().unwrap_or("");
                let page_domain = request_url.host_str().unwrap_or("");

                if self.is_third_party(cookie_domain, page_domain) {
                    // Check if it's a known tracker
                    let tracking_list = self.tracking_list.read().await;
                    if tracking_list.is_tracker(cookie_domain) {
                        return CookieDecision::Block(BlockReason::KnownTracker);
                    }
                    CookieDecision::Block(BlockReason::ThirdParty)
                } else {
                    CookieDecision::Allow
                }
            }
            CookiePolicy::Custom(rules) => {
                let cookie_domain = cookie.effective_domain().unwrap_or("");
                let page_domain = request_url.host_str().unwrap_or("");

                // Check if domain is explicitly allowed
                if rules
                    .allowed_domains
                    .iter()
                    .any(|d| cookie_domain == d || cookie_domain.ends_with(&format!(".{}", d)))
                {
                    return CookieDecision::Allow;
                }

                // Check if domain is explicitly blocked
                if rules
                    .blocked_domains
                    .iter()
                    .any(|d| cookie_domain == d || cookie_domain.ends_with(&format!(".{}", d)))
                {
                    return CookieDecision::Block(BlockReason::BlockedDomain);
                }

                // Check session/persistent cookie rules
                if cookie.is_session && rules.block_session_cookies {
                    return CookieDecision::Block(BlockReason::SessionCookieBlocked);
                }

                if !cookie.is_session && rules.block_persistent_cookies {
                    return CookieDecision::Block(BlockReason::PersistentCookieBlocked);
                }

                // Check third-party rules
                if rules.block_third_party_default
                    && self.is_third_party(cookie_domain, page_domain)
                {
                    return CookieDecision::Block(BlockReason::ThirdParty);
                }

                CookieDecision::Allow
            }
        }
    }

    /// Check if a cookie domain is third-party relative to a page domain.
    pub fn is_third_party(&self, cookie_domain: &str, page_domain: &str) -> bool {
        if cookie_domain.is_empty() || page_domain.is_empty() {
            return true;
        }

        let cookie_domain = cookie_domain.to_lowercase();
        let page_domain = page_domain.to_lowercase();

        // Remove leading dot from cookie domain if present
        let cookie_domain = cookie_domain.strip_prefix('.').unwrap_or(&cookie_domain);

        // Extract registrable domain (simplified - in production use publicsuffix list)
        let cookie_base = Self::get_base_domain(cookie_domain);
        let page_base = Self::get_base_domain(&page_domain);

        cookie_base != page_base
    }

    /// Get the base (registrable) domain from a full domain.
    /// This is a simplified implementation - production should use publicsuffix list.
    fn get_base_domain(domain: &str) -> &str {
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() <= 2 {
            return domain;
        }

        // Handle common TLDs like .co.uk, .com.au, etc.
        let last = parts.last().unwrap_or(&"");
        let second_last = parts.get(parts.len() - 2).unwrap_or(&"");

        // Common second-level TLDs
        let compound_tlds = ["co", "com", "org", "net", "gov", "edu", "ac"];

        if parts.len() > 2 && compound_tlds.contains(second_last) && last.len() == 2 {
            // e.g., example.co.uk -> take last 3 parts
            if parts.len() >= 3 {
                let suffix_len = parts[parts.len() - 3..].join(".").len();
                return &domain[domain.len() - suffix_len..];
            }
        }

        // Take last 2 parts for standard TLDs
        let suffix_len = parts[parts.len() - 2..].join(".").len();
        &domain[domain.len() - suffix_len..]
    }

    /// Check if tracking protection should block a request.
    pub async fn should_block_tracker(&self, url: &Url) -> bool {
        let settings = self.settings.read().await;
        if !settings.tracking_protection {
            return false;
        }

        if let Some(host) = url.host_str() {
            let tracking_list = self.tracking_list.read().await;
            tracking_list.is_tracker(host)
        } else {
            false
        }
    }

    /// Add a domain to the tracking protection list.
    pub async fn add_tracker(&self, domain: impl Into<String>) {
        self.tracking_list.write().await.add_tracker(domain);
    }

    /// Get the number of trackers in the protection list.
    pub async fn tracker_count(&self) -> usize {
        self.tracking_list.read().await.len()
    }

    /// Check if fingerprint protection is enabled.
    pub async fn is_fingerprint_protection_enabled(&self) -> bool {
        self.settings.read().await.fingerprint_protection
    }

    /// Enable or disable fingerprint protection.
    pub async fn set_fingerprint_protection(&self, enabled: bool) {
        self.settings.write().await.fingerprint_protection = enabled;
    }

    /// Get the headers that should be added for privacy.
    pub async fn get_privacy_headers(&self) -> Vec<(String, String)> {
        let settings = self.settings.read().await;
        let mut headers = Vec::new();

        if settings.dnt_enabled {
            headers.push(("DNT".to_string(), "1".to_string()));
        }

        if settings.gpc_enabled {
            headers.push(("Sec-GPC".to_string(), "1".to_string()));
        }

        headers
    }
}

impl Default for PrivacyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PrivacyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrivacyManager").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_settings() {
        let manager = PrivacyManager::new();
        let settings = manager.get_settings().await;

        assert!(settings.dnt_enabled);
        assert!(settings.tracking_protection);
        assert!(settings.gpc_enabled);
        assert_eq!(settings.cookie_policy, CookiePolicy::BlockThirdParty);
    }

    #[tokio::test]
    async fn test_dnt_enabled() {
        let manager = PrivacyManager::new();

        assert!(manager.should_send_dnt().await);

        manager.set_dnt_enabled(false).await;
        assert!(!manager.should_send_dnt().await);

        manager.set_dnt_enabled(true).await;
        assert!(manager.should_send_dnt().await);
    }

    #[tokio::test]
    async fn test_gpc_enabled() {
        let manager = PrivacyManager::new();

        assert!(manager.should_send_gpc().await);

        manager.set_gpc_enabled(false).await;
        assert!(!manager.should_send_gpc().await);
    }

    #[tokio::test]
    async fn test_third_party_detection() {
        let manager = PrivacyManager::new();

        // Same domain - not third party
        assert!(!manager.is_third_party("example.com", "example.com"));
        assert!(!manager.is_third_party("www.example.com", "example.com"));
        assert!(!manager.is_third_party("sub.example.com", "example.com"));
        assert!(!manager.is_third_party(".example.com", "www.example.com"));

        // Different domains - third party
        assert!(manager.is_third_party("tracker.com", "example.com"));
        assert!(manager.is_third_party("ad.tracker.com", "example.com"));
        assert!(manager.is_third_party("cdn.different.net", "example.com"));
    }

    #[tokio::test]
    async fn test_cookie_blocking_accept_all() {
        let settings = PrivacySettings {
            cookie_policy: CookiePolicy::AcceptAll,
            ..Default::default()
        };
        let manager = PrivacyManager::with_settings(settings);

        let cookie = Cookie::new("test", "value").domain("tracker.com");
        let url = Url::parse("https://example.com/page").unwrap();

        let decision = manager.should_block_cookie(&cookie, &url).await;
        assert_eq!(decision, CookieDecision::Allow);
    }

    #[tokio::test]
    async fn test_cookie_blocking_block_all() {
        let settings = PrivacySettings {
            cookie_policy: CookiePolicy::BlockAll,
            ..Default::default()
        };
        let manager = PrivacyManager::with_settings(settings);

        let cookie = Cookie::new("test", "value").domain("example.com");
        let url = Url::parse("https://example.com/page").unwrap();

        let decision = manager.should_block_cookie(&cookie, &url).await;
        assert_eq!(decision, CookieDecision::Block(BlockReason::PolicyBlockAll));
    }

    #[tokio::test]
    async fn test_cookie_blocking_third_party() {
        let manager = PrivacyManager::new(); // Default is BlockThirdParty

        // First-party cookie should be allowed
        let first_party = Cookie::new("session", "abc").domain("example.com");
        let url = Url::parse("https://example.com/page").unwrap();
        let decision = manager.should_block_cookie(&first_party, &url).await;
        assert_eq!(decision, CookieDecision::Allow);

        // Third-party cookie should be blocked
        let third_party = Cookie::new("track", "123").domain("tracker.com");
        let decision = manager.should_block_cookie(&third_party, &url).await;
        assert_eq!(decision, CookieDecision::Block(BlockReason::ThirdParty));
    }

    #[tokio::test]
    async fn test_cookie_blocking_known_tracker() {
        let manager = PrivacyManager::new();

        let cookie = Cookie::new("_ga", "tracking").domain("google-analytics.com");
        let url = Url::parse("https://example.com/page").unwrap();

        let decision = manager.should_block_cookie(&cookie, &url).await;
        assert_eq!(decision, CookieDecision::Block(BlockReason::KnownTracker));
    }

    #[tokio::test]
    async fn test_custom_cookie_rules_allowed() {
        let mut rules = CookieRules::default();
        rules.allowed_domains.insert("cdn.trusted.com".to_string());

        let settings = PrivacySettings {
            cookie_policy: CookiePolicy::Custom(rules),
            ..Default::default()
        };
        let manager = PrivacyManager::with_settings(settings);

        let cookie = Cookie::new("data", "value").domain("cdn.trusted.com");
        let url = Url::parse("https://example.com/page").unwrap();

        let decision = manager.should_block_cookie(&cookie, &url).await;
        assert_eq!(decision, CookieDecision::Allow);
    }

    #[tokio::test]
    async fn test_custom_cookie_rules_blocked() {
        let mut rules = CookieRules::default();
        rules.blocked_domains.insert("bad-tracker.com".to_string());

        let settings = PrivacySettings {
            cookie_policy: CookiePolicy::Custom(rules),
            ..Default::default()
        };
        let manager = PrivacyManager::with_settings(settings);

        let cookie = Cookie::new("evil", "track").domain("bad-tracker.com");
        let url = Url::parse("https://example.com/page").unwrap();

        let decision = manager.should_block_cookie(&cookie, &url).await;
        assert_eq!(decision, CookieDecision::Block(BlockReason::BlockedDomain));
    }

    #[tokio::test]
    async fn test_tracking_protection() {
        let manager = PrivacyManager::new();

        let tracker_url = Url::parse("https://google-analytics.com/collect").unwrap();
        assert!(manager.should_block_tracker(&tracker_url).await);

        let normal_url = Url::parse("https://example.com/page").unwrap();
        assert!(!manager.should_block_tracker(&normal_url).await);
    }

    #[tokio::test]
    async fn test_tracking_list() {
        let list = TrackingProtectionList::new();

        assert!(list.is_tracker("doubleclick.net"));
        assert!(list.is_tracker("sub.doubleclick.net"));
        assert!(list.is_tracker("google-analytics.com"));
        assert!(!list.is_tracker("example.com"));
    }

    #[tokio::test]
    async fn test_privacy_headers() {
        let manager = PrivacyManager::new();

        let headers = manager.get_privacy_headers().await;
        assert!(headers.iter().any(|(k, v)| k == "DNT" && v == "1"));
        assert!(headers.iter().any(|(k, v)| k == "Sec-GPC" && v == "1"));

        manager.set_dnt_enabled(false).await;
        manager.set_gpc_enabled(false).await;

        let headers = manager.get_privacy_headers().await;
        assert!(headers.is_empty());
    }

    #[test]
    fn test_cookie_builder() {
        let cookie = Cookie::new("session", "abc123")
            .domain("example.com")
            .path("/app")
            .secure()
            .http_only()
            .persistent();

        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.domain, Some("example.com".to_string()));
        assert_eq!(cookie.path, Some("/app".to_string()));
        assert!(cookie.secure);
        assert!(cookie.http_only);
        assert!(!cookie.is_session);
    }

    #[test]
    fn test_cookie_policy_default() {
        let policy = CookiePolicy::default();
        assert_eq!(policy, CookiePolicy::BlockThirdParty);
    }

    #[test]
    fn test_base_domain_extraction() {
        // Simple domains
        assert_eq!(PrivacyManager::get_base_domain("example.com"), "example.com");
        assert_eq!(
            PrivacyManager::get_base_domain("www.example.com"),
            "example.com"
        );
        assert_eq!(
            PrivacyManager::get_base_domain("sub.domain.example.com"),
            "example.com"
        );

        // Compound TLDs
        assert_eq!(
            PrivacyManager::get_base_domain("example.co.uk"),
            "example.co.uk"
        );
    }

    #[tokio::test]
    async fn test_update_settings() {
        let manager = PrivacyManager::new();

        let new_settings = PrivacySettings {
            dnt_enabled: false,
            cookie_policy: CookiePolicy::BlockAll,
            tracking_protection: false,
            gpc_enabled: false,
            fingerprint_protection: true,
        };

        manager.update_settings(new_settings.clone()).await;

        let settings = manager.get_settings().await;
        assert!(!settings.dnt_enabled);
        assert_eq!(settings.cookie_policy, CookiePolicy::BlockAll);
        assert!(!settings.tracking_protection);
        assert!(settings.fingerprint_protection);
    }

    #[tokio::test]
    async fn test_add_tracker() {
        let manager = PrivacyManager::new();
        let initial_count = manager.tracker_count().await;

        manager.add_tracker("new-tracker.com").await;

        assert_eq!(manager.tracker_count().await, initial_count + 1);

        let url = Url::parse("https://new-tracker.com/track").unwrap();
        assert!(manager.should_block_tracker(&url).await);
    }

    #[tokio::test]
    async fn test_fingerprint_protection() {
        let manager = PrivacyManager::new();

        assert!(!manager.is_fingerprint_protection_enabled().await);

        manager.set_fingerprint_protection(true).await;
        assert!(manager.is_fingerprint_protection_enabled().await);
    }

    #[test]
    fn test_block_reason_variants() {
        // Ensure all block reasons are distinct
        let reasons = vec![
            BlockReason::ThirdParty,
            BlockReason::BlockedDomain,
            BlockReason::PolicyBlockAll,
            BlockReason::SessionCookieBlocked,
            BlockReason::PersistentCookieBlocked,
            BlockReason::KnownTracker,
        ];

        for (i, r1) in reasons.iter().enumerate() {
            for (j, r2) in reasons.iter().enumerate() {
                if i == j {
                    assert_eq!(r1, r2);
                } else {
                    assert_ne!(r1, r2);
                }
            }
        }
    }
}
