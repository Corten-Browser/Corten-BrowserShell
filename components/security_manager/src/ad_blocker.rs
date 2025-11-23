//! Ad Blocker module for CortenBrowser.
//!
//! Provides ad blocking and content filtering capabilities including:
//! - URL-based ad blocking with pattern matching
//! - Multiple filter list support (EasyList, etc.)
//! - Custom rule creation
//! - Site whitelist/allowlist management
//! - Blocked content statistics

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

/// Action to take when a rule matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterAction {
    /// Block the request.
    Block,
    /// Allow the request (whitelist).
    Allow,
}

impl Default for FilterAction {
    fn default() -> Self {
        FilterAction::Block
    }
}

/// Type of content being filtered.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentType {
    /// Scripts (JavaScript, etc.)
    Script,
    /// Images
    Image,
    /// Stylesheets (CSS)
    Stylesheet,
    /// XMLHttpRequest / Fetch
    XmlHttpRequest,
    /// Subdocuments (iframes)
    Subdocument,
    /// Fonts
    Font,
    /// Media (audio/video)
    Media,
    /// WebSocket connections
    WebSocket,
    /// Other/unknown content
    Other,
}

/// A single filter rule for ad blocking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    /// Unique identifier for the rule.
    pub id: String,
    /// The pattern to match against URLs.
    pub pattern: String,
    /// Compiled regex pattern (not serialized).
    #[serde(skip)]
    regex: Option<Regex>,
    /// Action to take when matched.
    pub action: FilterAction,
    /// Domains where this rule applies (empty = all domains).
    pub domains: HashSet<String>,
    /// Domains where this rule is excluded.
    pub excluded_domains: HashSet<String>,
    /// Content types this rule applies to (empty = all types).
    pub content_types: HashSet<ContentType>,
    /// Whether this rule is currently enabled.
    pub enabled: bool,
}

impl FilterRule {
    /// Create a new blocking filter rule.
    pub fn new(id: impl Into<String>, pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        let regex = Self::compile_pattern(&pattern);
        Self {
            id: id.into(),
            pattern,
            regex,
            action: FilterAction::Block,
            domains: HashSet::new(),
            excluded_domains: HashSet::new(),
            content_types: HashSet::new(),
            enabled: true,
        }
    }

    /// Create a new whitelist (allow) rule.
    pub fn whitelist(id: impl Into<String>, pattern: impl Into<String>) -> Self {
        let mut rule = Self::new(id, pattern);
        rule.action = FilterAction::Allow;
        rule
    }

    /// Set the action for this rule.
    pub fn with_action(mut self, action: FilterAction) -> Self {
        self.action = action;
        self
    }

    /// Add a domain where this rule applies.
    pub fn for_domain(mut self, domain: impl Into<String>) -> Self {
        self.domains.insert(domain.into());
        self
    }

    /// Add a domain where this rule is excluded.
    pub fn except_domain(mut self, domain: impl Into<String>) -> Self {
        self.excluded_domains.insert(domain.into());
        self
    }

    /// Add a content type this rule applies to.
    pub fn for_content_type(mut self, content_type: ContentType) -> Self {
        self.content_types.insert(content_type);
        self
    }

    /// Enable or disable this rule.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Compile a filter pattern to regex.
    /// Supports:
    /// - * for wildcard matching
    /// - ^ for separator characters
    /// - || for domain anchor
    /// - | for start/end anchor
    fn compile_pattern(pattern: &str) -> Option<Regex> {
        let mut regex_str = String::new();

        // Handle domain anchor (||)
        let pattern = if let Some(stripped) = pattern.strip_prefix("||") {
            regex_str.push_str(r"^https?://([a-z0-9-]+\.)*");
            stripped
        } else if let Some(stripped) = pattern.strip_prefix('|') {
            regex_str.push('^');
            stripped
        } else {
            pattern
        };

        // Handle end anchor
        let (pattern, end_anchor) = if let Some(stripped) = pattern.strip_suffix('|') {
            (stripped, true)
        } else {
            (pattern, false)
        };

        // Convert pattern to regex
        for ch in pattern.chars() {
            match ch {
                '*' => regex_str.push_str(".*"),
                '^' => regex_str.push_str(r"[/:?#\[\]@!$&'()*+,;=\-._~%]"),
                '.' => regex_str.push_str(r"\."),
                '?' => regex_str.push_str(r"\?"),
                '+' => regex_str.push_str(r"\+"),
                '[' => regex_str.push_str(r"\["),
                ']' => regex_str.push_str(r"\]"),
                '(' => regex_str.push_str(r"\("),
                ')' => regex_str.push_str(r"\)"),
                '{' => regex_str.push_str(r"\{"),
                '}' => regex_str.push_str(r"\}"),
                '|' => regex_str.push_str(r"\|"),
                '\\' => regex_str.push_str(r"\\"),
                '$' => regex_str.push_str(r"\$"),
                _ => regex_str.push(ch),
            }
        }

        if end_anchor {
            regex_str.push('$');
        }

        Regex::new(&regex_str).ok()
    }

    /// Check if this rule matches a URL.
    pub fn matches(&self, url: &str, page_domain: Option<&str>, content_type: Option<&ContentType>) -> bool {
        if !self.enabled {
            return false;
        }

        // Check domain restrictions
        if let Some(domain) = page_domain {
            // If domains list is not empty, URL must match one of them
            if !self.domains.is_empty() && !self.domain_matches(domain, &self.domains) {
                return false;
            }
            // Check excluded domains
            if self.domain_matches(domain, &self.excluded_domains) {
                return false;
            }
        }

        // Check content type restrictions
        if let Some(ct) = content_type {
            if !self.content_types.is_empty() && !self.content_types.contains(ct) {
                return false;
            }
        }

        // Check URL pattern
        if let Some(ref regex) = self.regex {
            regex.is_match(url)
        } else {
            // Fallback to simple contains check
            url.contains(&self.pattern)
        }
    }

    /// Check if a domain matches any in a set (with subdomain support).
    fn domain_matches(&self, domain: &str, domain_set: &HashSet<String>) -> bool {
        let domain_lower = domain.to_lowercase();
        domain_set.iter().any(|d| {
            let d_lower = d.to_lowercase();
            domain_lower == d_lower || domain_lower.ends_with(&format!(".{}", d_lower))
        })
    }

    /// Reinitialize the compiled regex pattern.
    pub fn recompile(&mut self) {
        self.regex = Self::compile_pattern(&self.pattern);
    }
}

/// A collection of filter rules (e.g., EasyList).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterList {
    /// Unique identifier for this list.
    pub id: String,
    /// Display name for this list.
    pub name: String,
    /// Description of this list.
    pub description: String,
    /// URL where updates can be fetched (if applicable).
    pub update_url: Option<String>,
    /// List of filter rules.
    pub rules: Vec<FilterRule>,
    /// Whether this list is enabled.
    pub enabled: bool,
    /// Last update timestamp (Unix epoch).
    pub last_updated: u64,
}

impl FilterList {
    /// Create a new empty filter list.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            update_url: None,
            rules: Vec::new(),
            enabled: true,
            last_updated: 0,
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the update URL.
    pub fn with_update_url(mut self, url: impl Into<String>) -> Self {
        self.update_url = Some(url.into());
        self
    }

    /// Add a rule to this list.
    pub fn add_rule(&mut self, rule: FilterRule) {
        self.rules.push(rule);
    }

    /// Remove a rule by ID.
    pub fn remove_rule(&mut self, rule_id: &str) -> bool {
        let initial_len = self.rules.len();
        self.rules.retain(|r| r.id != rule_id);
        self.rules.len() < initial_len
    }

    /// Get the number of rules in this list.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Enable or disable this list.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if any rule in this list matches.
    pub fn matches(&self, url: &str, page_domain: Option<&str>, content_type: Option<&ContentType>) -> Option<&FilterRule> {
        if !self.enabled {
            return None;
        }
        self.rules.iter().find(|rule| rule.matches(url, page_domain, content_type))
    }

    /// Recompile all regex patterns in this list.
    pub fn recompile_all(&mut self) {
        for rule in &mut self.rules {
            rule.recompile();
        }
    }
}

/// Statistics about blocked content.
#[derive(Debug, Default)]
pub struct BlockStats {
    /// Total number of blocked requests.
    pub total_blocked: AtomicU64,
    /// Blocked requests per domain.
    blocked_by_domain: Arc<RwLock<HashMap<String, u64>>>,
    /// Blocked requests by content type.
    blocked_by_type: Arc<RwLock<HashMap<ContentType, u64>>>,
}

impl BlockStats {
    /// Create new block stats.
    pub fn new() -> Self {
        Self {
            total_blocked: AtomicU64::new(0),
            blocked_by_domain: Arc::new(RwLock::new(HashMap::new())),
            blocked_by_type: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a blocked request.
    pub async fn record_block(&self, domain: Option<&str>, content_type: Option<&ContentType>) {
        self.total_blocked.fetch_add(1, Ordering::SeqCst);

        if let Some(domain) = domain {
            let mut by_domain = self.blocked_by_domain.write().await;
            *by_domain.entry(domain.to_string()).or_insert(0) += 1;
        }

        if let Some(ct) = content_type {
            let mut by_type = self.blocked_by_type.write().await;
            *by_type.entry(ct.clone()).or_insert(0) += 1;
        }
    }

    /// Get total blocked count.
    pub fn get_total(&self) -> u64 {
        self.total_blocked.load(Ordering::SeqCst)
    }

    /// Get blocked count for a specific domain.
    pub async fn get_domain_count(&self, domain: &str) -> u64 {
        self.blocked_by_domain.read().await.get(domain).copied().unwrap_or(0)
    }

    /// Get blocked count for a specific content type.
    pub async fn get_type_count(&self, content_type: &ContentType) -> u64 {
        self.blocked_by_type.read().await.get(content_type).copied().unwrap_or(0)
    }

    /// Reset all statistics.
    pub async fn reset(&self) {
        self.total_blocked.store(0, Ordering::SeqCst);
        self.blocked_by_domain.write().await.clear();
        self.blocked_by_type.write().await.clear();
    }

    /// Get all domain stats.
    pub async fn get_all_domain_stats(&self) -> HashMap<String, u64> {
        self.blocked_by_domain.read().await.clone()
    }

    /// Get all content type stats.
    pub async fn get_all_type_stats(&self) -> HashMap<ContentType, u64> {
        self.blocked_by_type.read().await.clone()
    }
}

/// Result of checking a URL against the ad blocker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// URL is allowed.
    Allow,
    /// URL is blocked by a rule.
    Blocked {
        /// ID of the rule that blocked it.
        rule_id: String,
        /// ID of the list containing the rule.
        list_id: String,
    },
    /// URL is whitelisted by a rule.
    Whitelisted {
        /// ID of the whitelist rule.
        rule_id: String,
        /// ID of the list containing the rule.
        list_id: String,
    },
    /// Site is in the allowlist, bypassing all rules.
    SiteAllowed,
}

/// Main ad blocker structure.
pub struct AdBlocker {
    /// Whether ad blocking is enabled globally.
    enabled: Arc<RwLock<bool>>,
    /// Filter lists.
    filter_lists: Arc<RwLock<HashMap<String, FilterList>>>,
    /// Custom rules (not part of any list).
    custom_rules: Arc<RwLock<Vec<FilterRule>>>,
    /// Whitelisted/allowlisted sites (bypass all blocking).
    allowlist: Arc<RwLock<HashSet<String>>>,
    /// Blocking statistics.
    stats: Arc<BlockStats>,
}

impl AdBlocker {
    /// Create a new ad blocker with default settings.
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(RwLock::new(true)),
            filter_lists: Arc::new(RwLock::new(HashMap::new())),
            custom_rules: Arc::new(RwLock::new(Vec::new())),
            allowlist: Arc::new(RwLock::new(HashSet::new())),
            stats: Arc::new(BlockStats::new()),
        }
    }

    /// Create a new ad blocker with built-in filter rules.
    pub fn with_default_rules() -> Self {
        let blocker = Self::new();

        // We'll add default rules in an async context when the blocker is used
        blocker
    }

    /// Add default EasyList-style rules.
    pub async fn add_default_rules(&self) {
        let mut easylist = FilterList::new("easylist", "EasyList")
            .with_description("Primary ad blocking list");

        // Common ad network patterns
        let patterns = [
            ("easylist-1", "||doubleclick.net^"),
            ("easylist-2", "||googlesyndication.com^"),
            ("easylist-3", "||googleadservices.com^"),
            ("easylist-4", "||google-analytics.com/analytics.js"),
            ("easylist-5", "||adnxs.com^"),
            ("easylist-6", "||adsrvr.org^"),
            ("easylist-7", "||criteo.com^"),
            ("easylist-8", "||scorecardresearch.com^"),
            ("easylist-9", "||quantserve.com^"),
            ("easylist-10", "||outbrain.com/outbrain.js"),
            ("easylist-11", "||taboola.com^"),
            ("easylist-12", "||amazon-adsystem.com^"),
            ("easylist-13", "||facebook.com/tr^"),
            ("easylist-14", "||ads.twitter.com^"),
            ("easylist-15", "/pagead/"),
            ("easylist-16", "/ads/*"),
            ("easylist-17", "/adserver/"),
            ("easylist-18", "||advertising.com^"),
            ("easylist-19", "||bidswitch.net^"),
            ("easylist-20", "||rubiconproject.com^"),
        ];

        for (id, pattern) in patterns {
            easylist.add_rule(FilterRule::new(id, pattern));
        }

        self.add_filter_list(easylist).await;
    }

    /// Check if ad blocking is enabled.
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Enable or disable ad blocking.
    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().await = enabled;
    }

    /// Add a filter list.
    pub async fn add_filter_list(&self, list: FilterList) {
        let mut lists = self.filter_lists.write().await;
        lists.insert(list.id.clone(), list);
    }

    /// Remove a filter list by ID.
    pub async fn remove_filter_list(&self, list_id: &str) -> bool {
        let mut lists = self.filter_lists.write().await;
        lists.remove(list_id).is_some()
    }

    /// Get a filter list by ID.
    pub async fn get_filter_list(&self, list_id: &str) -> Option<FilterList> {
        let lists = self.filter_lists.read().await;
        lists.get(list_id).cloned()
    }

    /// Get all filter lists.
    pub async fn get_all_filter_lists(&self) -> Vec<FilterList> {
        let lists = self.filter_lists.read().await;
        lists.values().cloned().collect()
    }

    /// Enable or disable a filter list.
    pub async fn set_list_enabled(&self, list_id: &str, enabled: bool) -> bool {
        let mut lists = self.filter_lists.write().await;
        if let Some(list) = lists.get_mut(list_id) {
            list.set_enabled(enabled);
            true
        } else {
            false
        }
    }

    /// Add a custom rule.
    pub async fn add_rule(&self, rule: FilterRule) {
        let mut rules = self.custom_rules.write().await;
        rules.push(rule);
    }

    /// Remove a custom rule by ID.
    pub async fn remove_rule(&self, rule_id: &str) -> bool {
        let mut rules = self.custom_rules.write().await;
        let initial_len = rules.len();
        rules.retain(|r| r.id != rule_id);
        rules.len() < initial_len
    }

    /// Get all custom rules.
    pub async fn get_custom_rules(&self) -> Vec<FilterRule> {
        self.custom_rules.read().await.clone()
    }

    /// Add a site to the allowlist.
    pub async fn add_to_allowlist(&self, domain: impl Into<String>) {
        let mut allowlist = self.allowlist.write().await;
        allowlist.insert(domain.into().to_lowercase());
    }

    /// Remove a site from the allowlist.
    pub async fn remove_from_allowlist(&self, domain: &str) -> bool {
        let mut allowlist = self.allowlist.write().await;
        allowlist.remove(&domain.to_lowercase())
    }

    /// Check if a site is in the allowlist.
    pub async fn is_allowlisted(&self, domain: &str) -> bool {
        let allowlist = self.allowlist.read().await;
        let domain_lower = domain.to_lowercase();
        allowlist.iter().any(|d| {
            domain_lower == *d || domain_lower.ends_with(&format!(".{}", d))
        })
    }

    /// Get all allowlisted sites.
    pub async fn get_allowlist(&self) -> Vec<String> {
        let allowlist = self.allowlist.read().await;
        allowlist.iter().cloned().collect()
    }

    /// Check if a URL should be blocked.
    pub async fn check_url(&self, url: &str, page_domain: Option<&str>, content_type: Option<&ContentType>) -> CheckResult {
        // Check if ad blocking is enabled
        if !*self.enabled.read().await {
            return CheckResult::Allow;
        }

        // Check if the page domain is allowlisted
        if let Some(domain) = page_domain {
            if self.is_allowlisted(domain).await {
                return CheckResult::SiteAllowed;
            }
        }

        // Check custom rules first (allow rules have priority)
        let custom_rules = self.custom_rules.read().await;
        for rule in custom_rules.iter() {
            if rule.matches(url, page_domain, content_type) {
                match rule.action {
                    FilterAction::Allow => {
                        return CheckResult::Whitelisted {
                            rule_id: rule.id.clone(),
                            list_id: "custom".to_string(),
                        };
                    }
                    FilterAction::Block => {
                        // Record the block
                        let extracted_domain = Self::extract_domain(url);
                        self.stats.record_block(extracted_domain.as_deref(), content_type).await;
                        return CheckResult::Blocked {
                            rule_id: rule.id.clone(),
                            list_id: "custom".to_string(),
                        };
                    }
                }
            }
        }
        drop(custom_rules);

        // Check filter lists (allow rules first)
        let lists = self.filter_lists.read().await;

        // First pass: check for whitelist rules
        for (list_id, list) in lists.iter() {
            if !list.enabled {
                continue;
            }
            for rule in &list.rules {
                if rule.action == FilterAction::Allow && rule.matches(url, page_domain, content_type) {
                    return CheckResult::Whitelisted {
                        rule_id: rule.id.clone(),
                        list_id: list_id.clone(),
                    };
                }
            }
        }

        // Second pass: check for block rules
        for (list_id, list) in lists.iter() {
            if !list.enabled {
                continue;
            }
            for rule in &list.rules {
                if rule.action == FilterAction::Block && rule.matches(url, page_domain, content_type) {
                    // Record the block
                    let extracted_domain = Self::extract_domain(url);
                    self.stats.record_block(extracted_domain.as_deref(), content_type).await;
                    return CheckResult::Blocked {
                        rule_id: rule.id.clone(),
                        list_id: list_id.clone(),
                    };
                }
            }
        }

        CheckResult::Allow
    }

    /// Extract domain from URL.
    fn extract_domain(url: &str) -> Option<String> {
        Url::parse(url).ok().and_then(|u| u.host_str().map(String::from))
    }

    /// Get blocking statistics.
    pub fn get_stats(&self) -> &BlockStats {
        &self.stats
    }

    /// Get the total number of blocked requests.
    pub fn blocked_count(&self) -> u64 {
        self.stats.get_total()
    }

    /// Reset blocking statistics.
    pub async fn reset_stats(&self) {
        self.stats.reset().await;
    }

    /// Get total number of rules across all lists and custom rules.
    pub async fn total_rule_count(&self) -> usize {
        let lists = self.filter_lists.read().await;
        let custom_count = self.custom_rules.read().await.len();
        let list_count: usize = lists.values().map(|l| l.rule_count()).sum();
        custom_count + list_count
    }
}

impl Default for AdBlocker {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for AdBlocker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdBlocker")
            .field("blocked_count", &self.stats.get_total())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_rule_creation() {
        let rule = FilterRule::new("test-1", "||example.com^");
        assert_eq!(rule.id, "test-1");
        assert_eq!(rule.pattern, "||example.com^");
        assert_eq!(rule.action, FilterAction::Block);
        assert!(rule.enabled);
    }

    #[test]
    fn test_filter_rule_whitelist() {
        let rule = FilterRule::whitelist("allow-1", "||trusted.com^");
        assert_eq!(rule.action, FilterAction::Allow);
    }

    #[test]
    fn test_filter_rule_domain_restriction() {
        let rule = FilterRule::new("test-2", "*ads*")
            .for_domain("example.com")
            .except_domain("subdomain.example.com");

        assert!(rule.domains.contains("example.com"));
        assert!(rule.excluded_domains.contains("subdomain.example.com"));
    }

    #[test]
    fn test_filter_rule_content_type() {
        let rule = FilterRule::new("test-3", "*tracking*")
            .for_content_type(ContentType::Script)
            .for_content_type(ContentType::XmlHttpRequest);

        assert!(rule.content_types.contains(&ContentType::Script));
        assert!(rule.content_types.contains(&ContentType::XmlHttpRequest));
        assert!(!rule.content_types.contains(&ContentType::Image));
    }

    #[test]
    fn test_filter_rule_pattern_matching() {
        // Simple wildcard
        let rule = FilterRule::new("test-wild", "*ads*");
        assert!(rule.matches("https://example.com/ads/banner.jpg", None, None));
        assert!(!rule.matches("https://example.com/page", None, None));

        // Domain anchor
        let rule = FilterRule::new("test-domain", "||doubleclick.net^");
        assert!(rule.matches("https://doubleclick.net/ad.js", None, None));
        assert!(rule.matches("https://sub.doubleclick.net/track", None, None));
        assert!(!rule.matches("https://example.com/doubleclick.net", None, None));

        // Path matching
        let rule = FilterRule::new("test-path", "/pagead/");
        assert!(rule.matches("https://example.com/pagead/show_ads.js", None, None));
    }

    #[test]
    fn test_filter_rule_domain_matching() {
        let rule = FilterRule::new("test-domain-match", "*ad*")
            .for_domain("example.com");

        // Should match on example.com
        assert!(rule.matches("https://ad.com/script.js", Some("example.com"), None));
        assert!(rule.matches("https://ad.com/script.js", Some("sub.example.com"), None));

        // Should not match on other domains
        assert!(!rule.matches("https://ad.com/script.js", Some("other.com"), None));
    }

    #[test]
    fn test_filter_rule_excluded_domain() {
        let rule = FilterRule::new("test-exclude", "*ad*")
            .except_domain("trusted.com");

        // Should not match on trusted.com
        assert!(!rule.matches("https://ad.com/script.js", Some("trusted.com"), None));
        assert!(!rule.matches("https://ad.com/script.js", Some("sub.trusted.com"), None));

        // Should match on other domains
        assert!(rule.matches("https://ad.com/script.js", Some("example.com"), None));
    }

    #[test]
    fn test_filter_rule_content_type_matching() {
        let rule = FilterRule::new("test-script", "*tracking*")
            .for_content_type(ContentType::Script);

        // Should match script content type
        assert!(rule.matches("https://example.com/tracking.js", None, Some(&ContentType::Script)));

        // Should not match other content types
        assert!(!rule.matches("https://example.com/tracking.js", None, Some(&ContentType::Image)));
    }

    #[test]
    fn test_filter_rule_disabled() {
        let mut rule = FilterRule::new("test-disabled", "*ad*");
        rule.set_enabled(false);

        assert!(!rule.matches("https://ad.com/", None, None));
    }

    #[test]
    fn test_filter_list_creation() {
        let list = FilterList::new("test-list", "Test List")
            .with_description("A test filter list")
            .with_update_url("https://example.com/list.txt");

        assert_eq!(list.id, "test-list");
        assert_eq!(list.name, "Test List");
        assert_eq!(list.description, "A test filter list");
        assert_eq!(list.update_url, Some("https://example.com/list.txt".to_string()));
        assert!(list.enabled);
    }

    #[test]
    fn test_filter_list_rules() {
        let mut list = FilterList::new("test-list", "Test List");

        list.add_rule(FilterRule::new("rule-1", "*ad1*"));
        list.add_rule(FilterRule::new("rule-2", "*ad2*"));

        assert_eq!(list.rule_count(), 2);

        assert!(list.remove_rule("rule-1"));
        assert_eq!(list.rule_count(), 1);

        assert!(!list.remove_rule("nonexistent"));
    }

    #[test]
    fn test_filter_list_matching() {
        let mut list = FilterList::new("test-list", "Test List");
        list.add_rule(FilterRule::new("rule-1", "*ads*"));
        list.add_rule(FilterRule::new("rule-2", "*tracking*"));

        let matched = list.matches("https://example.com/ads/banner.jpg", None, None);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().id, "rule-1");

        let not_matched = list.matches("https://example.com/page", None, None);
        assert!(not_matched.is_none());
    }

    #[test]
    fn test_filter_list_disabled() {
        let mut list = FilterList::new("test-list", "Test List");
        list.add_rule(FilterRule::new("rule-1", "*ads*"));
        list.set_enabled(false);

        let matched = list.matches("https://example.com/ads/banner.jpg", None, None);
        assert!(matched.is_none());
    }

    #[tokio::test]
    async fn test_ad_blocker_creation() {
        let blocker = AdBlocker::new();
        assert!(blocker.is_enabled().await);
        assert_eq!(blocker.blocked_count(), 0);
    }

    #[tokio::test]
    async fn test_ad_blocker_enable_disable() {
        let blocker = AdBlocker::new();

        blocker.set_enabled(false).await;
        assert!(!blocker.is_enabled().await);

        blocker.set_enabled(true).await;
        assert!(blocker.is_enabled().await);
    }

    #[tokio::test]
    async fn test_ad_blocker_filter_lists() {
        let blocker = AdBlocker::new();

        let list = FilterList::new("my-list", "My List");
        blocker.add_filter_list(list).await;

        let retrieved = blocker.get_filter_list("my-list").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "My List");

        let all_lists = blocker.get_all_filter_lists().await;
        assert_eq!(all_lists.len(), 1);

        assert!(blocker.remove_filter_list("my-list").await);
        assert!(blocker.get_filter_list("my-list").await.is_none());
    }

    #[tokio::test]
    async fn test_ad_blocker_custom_rules() {
        let blocker = AdBlocker::new();

        blocker.add_rule(FilterRule::new("custom-1", "*myad*")).await;
        blocker.add_rule(FilterRule::new("custom-2", "*mytracker*")).await;

        let rules = blocker.get_custom_rules().await;
        assert_eq!(rules.len(), 2);

        assert!(blocker.remove_rule("custom-1").await);
        assert_eq!(blocker.get_custom_rules().await.len(), 1);
    }

    #[tokio::test]
    async fn test_ad_blocker_allowlist() {
        let blocker = AdBlocker::new();

        blocker.add_to_allowlist("trusted.com").await;

        assert!(blocker.is_allowlisted("trusted.com").await);
        assert!(blocker.is_allowlisted("sub.trusted.com").await);
        assert!(!blocker.is_allowlisted("other.com").await);

        let allowlist = blocker.get_allowlist().await;
        assert_eq!(allowlist.len(), 1);

        assert!(blocker.remove_from_allowlist("trusted.com").await);
        assert!(!blocker.is_allowlisted("trusted.com").await);
    }

    #[tokio::test]
    async fn test_ad_blocker_check_url_blocked() {
        let blocker = AdBlocker::new();

        let mut list = FilterList::new("test-list", "Test List");
        list.add_rule(FilterRule::new("rule-1", "||ads.example.com^"));
        blocker.add_filter_list(list).await;

        let result = blocker.check_url("https://ads.example.com/banner.js", None, None).await;
        assert!(matches!(result, CheckResult::Blocked { .. }));

        if let CheckResult::Blocked { rule_id, list_id } = result {
            assert_eq!(rule_id, "rule-1");
            assert_eq!(list_id, "test-list");
        }

        // Counter should be incremented
        assert_eq!(blocker.blocked_count(), 1);
    }

    #[tokio::test]
    async fn test_ad_blocker_check_url_allowed() {
        let blocker = AdBlocker::new();

        let mut list = FilterList::new("test-list", "Test List");
        list.add_rule(FilterRule::new("rule-1", "||ads.example.com^"));
        blocker.add_filter_list(list).await;

        let result = blocker.check_url("https://safe.example.com/page", None, None).await;
        assert_eq!(result, CheckResult::Allow);
    }

    #[tokio::test]
    async fn test_ad_blocker_check_url_whitelisted() {
        let blocker = AdBlocker::new();

        let mut list = FilterList::new("test-list", "Test List");
        list.add_rule(FilterRule::new("block-1", "*ads*"));
        list.add_rule(FilterRule::whitelist("allow-1", "||trusted-ads.com^"));
        blocker.add_filter_list(list).await;

        // Whitelisted URL should be allowed
        let result = blocker.check_url("https://trusted-ads.com/ad.js", None, None).await;
        assert!(matches!(result, CheckResult::Whitelisted { .. }));
    }

    #[tokio::test]
    async fn test_ad_blocker_check_url_site_allowed() {
        let blocker = AdBlocker::new();

        let mut list = FilterList::new("test-list", "Test List");
        list.add_rule(FilterRule::new("rule-1", "*ads*"));
        blocker.add_filter_list(list).await;

        blocker.add_to_allowlist("mysite.com").await;

        // Even though the URL would match blocking rules, the site is allowlisted
        let result = blocker.check_url("https://ads.com/banner.js", Some("mysite.com"), None).await;
        assert_eq!(result, CheckResult::SiteAllowed);
    }

    #[tokio::test]
    async fn test_ad_blocker_disabled() {
        let blocker = AdBlocker::new();

        let mut list = FilterList::new("test-list", "Test List");
        list.add_rule(FilterRule::new("rule-1", "*ads*"));
        blocker.add_filter_list(list).await;

        blocker.set_enabled(false).await;

        let result = blocker.check_url("https://ads.example.com/banner.js", None, None).await;
        assert_eq!(result, CheckResult::Allow);
    }

    #[tokio::test]
    async fn test_ad_blocker_list_enabled() {
        let blocker = AdBlocker::new();

        let mut list = FilterList::new("test-list", "Test List");
        list.add_rule(FilterRule::new("rule-1", "*ads*"));
        blocker.add_filter_list(list).await;

        // Disable the list
        assert!(blocker.set_list_enabled("test-list", false).await);

        let result = blocker.check_url("https://ads.example.com/banner.js", None, None).await;
        assert_eq!(result, CheckResult::Allow);

        // Re-enable
        assert!(blocker.set_list_enabled("test-list", true).await);

        let result = blocker.check_url("https://ads.example.com/banner.js", None, None).await;
        assert!(matches!(result, CheckResult::Blocked { .. }));
    }

    #[tokio::test]
    async fn test_ad_blocker_default_rules() {
        let blocker = AdBlocker::new();
        blocker.add_default_rules().await;

        // Should block common ad networks
        let result = blocker.check_url("https://doubleclick.net/ad.js", None, None).await;
        assert!(matches!(result, CheckResult::Blocked { .. }));

        let result = blocker.check_url("https://googlesyndication.com/pagead/js/adsbygoogle.js", None, None).await;
        assert!(matches!(result, CheckResult::Blocked { .. }));

        let result = blocker.check_url("https://google-analytics.com/analytics.js", None, None).await;
        assert!(matches!(result, CheckResult::Blocked { .. }));
    }

    #[tokio::test]
    async fn test_ad_blocker_stats() {
        let blocker = AdBlocker::new();

        let mut list = FilterList::new("test-list", "Test List");
        list.add_rule(FilterRule::new("rule-1", "||ads.example.com^"));
        blocker.add_filter_list(list).await;

        // Block some requests
        blocker.check_url("https://ads.example.com/1.js", None, Some(&ContentType::Script)).await;
        blocker.check_url("https://ads.example.com/2.js", None, Some(&ContentType::Script)).await;
        blocker.check_url("https://ads.example.com/3.png", None, Some(&ContentType::Image)).await;

        assert_eq!(blocker.blocked_count(), 3);

        let stats = blocker.get_stats();
        assert_eq!(stats.get_type_count(&ContentType::Script).await, 2);
        assert_eq!(stats.get_type_count(&ContentType::Image).await, 1);
        assert_eq!(stats.get_domain_count("ads.example.com").await, 3);

        // Reset stats
        blocker.reset_stats().await;
        assert_eq!(blocker.blocked_count(), 0);
    }

    #[tokio::test]
    async fn test_ad_blocker_total_rule_count() {
        let blocker = AdBlocker::new();

        let mut list1 = FilterList::new("list-1", "List 1");
        list1.add_rule(FilterRule::new("r1", "*a*"));
        list1.add_rule(FilterRule::new("r2", "*b*"));

        let mut list2 = FilterList::new("list-2", "List 2");
        list2.add_rule(FilterRule::new("r3", "*c*"));

        blocker.add_filter_list(list1).await;
        blocker.add_filter_list(list2).await;
        blocker.add_rule(FilterRule::new("custom-1", "*d*")).await;

        assert_eq!(blocker.total_rule_count().await, 4);
    }

    #[tokio::test]
    async fn test_block_stats() {
        let stats = BlockStats::new();

        stats.record_block(Some("ads.com"), Some(&ContentType::Script)).await;
        stats.record_block(Some("ads.com"), Some(&ContentType::Image)).await;
        stats.record_block(Some("tracker.com"), Some(&ContentType::Script)).await;

        assert_eq!(stats.get_total(), 3);
        assert_eq!(stats.get_domain_count("ads.com").await, 2);
        assert_eq!(stats.get_domain_count("tracker.com").await, 1);
        assert_eq!(stats.get_type_count(&ContentType::Script).await, 2);
        assert_eq!(stats.get_type_count(&ContentType::Image).await, 1);

        let domain_stats = stats.get_all_domain_stats().await;
        assert_eq!(domain_stats.len(), 2);

        let type_stats = stats.get_all_type_stats().await;
        assert_eq!(type_stats.len(), 2);

        stats.reset().await;
        assert_eq!(stats.get_total(), 0);
    }

    #[test]
    fn test_filter_action_default() {
        let action = FilterAction::default();
        assert_eq!(action, FilterAction::Block);
    }

    #[test]
    fn test_content_type_variants() {
        let types = vec![
            ContentType::Script,
            ContentType::Image,
            ContentType::Stylesheet,
            ContentType::XmlHttpRequest,
            ContentType::Subdocument,
            ContentType::Font,
            ContentType::Media,
            ContentType::WebSocket,
            ContentType::Other,
        ];

        // Verify all variants are distinct
        for (i, t1) in types.iter().enumerate() {
            for (j, t2) in types.iter().enumerate() {
                if i == j {
                    assert_eq!(t1, t2);
                } else {
                    assert_ne!(t1, t2);
                }
            }
        }
    }

    #[test]
    fn test_check_result_variants() {
        let allow = CheckResult::Allow;
        let blocked = CheckResult::Blocked {
            rule_id: "r1".to_string(),
            list_id: "l1".to_string(),
        };
        let whitelisted = CheckResult::Whitelisted {
            rule_id: "r2".to_string(),
            list_id: "l2".to_string(),
        };
        let site_allowed = CheckResult::SiteAllowed;

        assert_ne!(allow, blocked);
        assert_ne!(blocked, whitelisted);
        assert_ne!(whitelisted, site_allowed);
    }

    #[test]
    fn test_filter_rule_recompile() {
        let mut rule = FilterRule::new("test", "||example.com^");

        // Change pattern and recompile
        rule.pattern = "||new-pattern.com^".to_string();
        rule.recompile();

        assert!(rule.matches("https://new-pattern.com/page", None, None));
        assert!(!rule.matches("https://example.com/page", None, None));
    }

    #[test]
    fn test_filter_list_recompile_all() {
        let mut list = FilterList::new("test", "Test");
        list.add_rule(FilterRule::new("r1", "||a.com^"));
        list.add_rule(FilterRule::new("r2", "||b.com^"));

        // Modify patterns
        list.rules[0].pattern = "||x.com^".to_string();
        list.rules[1].pattern = "||y.com^".to_string();

        // Recompile all
        list.recompile_all();

        assert!(list.matches("https://x.com/", None, None).is_some());
        assert!(list.matches("https://y.com/", None, None).is_some());
        assert!(list.matches("https://a.com/", None, None).is_none());
    }
}
