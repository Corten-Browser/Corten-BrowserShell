//! Extension Manifest Parser
//!
//! Parses Chrome extension manifest v3 format.

use crate::browser_action::BrowserAction;
use crate::permissions::PermissionSet;
use crate::types::{ContentScript, ContentScriptMatch, ContentScriptRunAt, ExtensionError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during manifest parsing
#[derive(Error, Debug)]
pub enum ManifestParseError {
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field value: {0}")]
    InvalidValue(String),

    #[error("Unsupported manifest version: {0}")]
    UnsupportedVersion(u32),
}

impl From<ManifestParseError> for ExtensionError {
    fn from(e: ManifestParseError) -> Self {
        ExtensionError::InvalidManifest(e.to_string())
    }
}

/// Parsed extension manifest (Manifest V3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    /// Internal name
    pub name: String,
    /// Display name (defaults to name)
    pub display_name: String,
    /// Version string
    pub version: String,
    /// Manifest version (should be 3)
    pub manifest_version: u32,
    /// Description
    pub description: String,
    /// Author
    pub author: Option<String>,
    /// Homepage URL
    pub homepage_url: Option<String>,
    /// Icons (size -> path)
    pub icons: HashMap<u32, String>,
    /// Permissions
    pub permissions: PermissionSet,
    /// Optional permissions
    pub optional_permissions: PermissionSet,
    /// Host permissions
    pub host_permissions: Vec<String>,
    /// Browser action configuration
    pub browser_action: Option<BrowserAction>,
    /// Content scripts
    pub content_scripts: Vec<ContentScript>,
    /// Background script configuration
    pub background: Option<BackgroundConfig>,
    /// Options page
    pub options_page: Option<String>,
    /// Options UI
    pub options_ui: Option<OptionsUiConfig>,
    /// Web accessible resources
    pub web_accessible_resources: Vec<WebAccessibleResource>,
    /// Content Security Policy
    pub content_security_policy: Option<ContentSecurityPolicyConfig>,
}

impl ExtensionManifest {
    /// Parse a manifest from JSON string
    pub fn parse(json: &str) -> Result<Self, ManifestParseError> {
        let raw: RawManifest = serde_json::from_str(json)?;
        Self::from_raw(raw)
    }

    /// Convert from raw manifest structure
    fn from_raw(raw: RawManifest) -> Result<Self, ManifestParseError> {
        // Validate required fields
        if raw.name.is_empty() {
            return Err(ManifestParseError::MissingField("name".to_string()));
        }
        if raw.version.is_empty() {
            return Err(ManifestParseError::MissingField("version".to_string()));
        }

        // Check manifest version (we support v2 and v3)
        if raw.manifest_version < 2 || raw.manifest_version > 3 {
            return Err(ManifestParseError::UnsupportedVersion(raw.manifest_version));
        }

        // Parse permissions
        let permissions = PermissionSet::from_strings(&raw.permissions);
        let optional_permissions = PermissionSet::from_strings(&raw.optional_permissions);

        // Parse browser action (action in v3, browser_action in v2)
        let browser_action = raw
            .action
            .or(raw.browser_action)
            .map(|ba| BrowserAction {
                default_icon: ba.default_icon.unwrap_or_default(),
                default_title: ba.default_title.unwrap_or_default(),
                default_popup: ba.default_popup,
            });

        // Parse content scripts
        let content_scripts = raw
            .content_scripts
            .into_iter()
            .map(|cs| ContentScript {
                js: cs.js,
                css: cs.css,
                matches: cs
                    .matches
                    .into_iter()
                    .map(ContentScriptMatch::new)
                    .collect(),
                exclude_matches: cs
                    .exclude_matches
                    .into_iter()
                    .map(ContentScriptMatch::new)
                    .collect(),
                run_at: match cs.run_at.as_deref() {
                    Some("document_start") => ContentScriptRunAt::DocumentStart,
                    Some("document_end") => ContentScriptRunAt::DocumentEnd,
                    Some("document_idle") | None => ContentScriptRunAt::DocumentIdle,
                    _ => ContentScriptRunAt::DocumentIdle,
                },
                all_frames: cs.all_frames,
            })
            .collect();

        // Parse background
        let background = raw.background.map(|bg| BackgroundConfig {
            service_worker: bg.service_worker,
            scripts: bg.scripts,
            persistent: bg.persistent,
        });

        // Parse options UI
        let options_ui = raw.options_ui.map(|ou| OptionsUiConfig {
            page: ou.page,
            open_in_tab: ou.open_in_tab,
        });

        // Parse icons
        let icons = raw
            .icons
            .into_iter()
            .filter_map(|(k, v)| k.parse::<u32>().ok().map(|size| (size, v)))
            .collect();

        // Parse web accessible resources
        let web_accessible_resources = raw
            .web_accessible_resources
            .into_iter()
            .map(|war| WebAccessibleResource {
                resources: war.resources,
                matches: war.matches,
            })
            .collect();

        // Parse CSP
        let content_security_policy = raw.content_security_policy.map(|csp| match csp {
            RawCsp::String(s) => ContentSecurityPolicyConfig {
                extension_pages: Some(s),
                sandbox: None,
            },
            RawCsp::Object { extension_pages, sandbox } => ContentSecurityPolicyConfig {
                extension_pages,
                sandbox,
            },
        });

        Ok(ExtensionManifest {
            name: raw.name.clone(),
            display_name: raw.short_name.unwrap_or(raw.name),
            version: raw.version,
            manifest_version: raw.manifest_version,
            description: raw.description.unwrap_or_default(),
            author: raw.author,
            homepage_url: raw.homepage_url,
            icons,
            permissions,
            optional_permissions,
            host_permissions: raw.host_permissions,
            browser_action,
            content_scripts,
            background,
            options_page: raw.options_page,
            options_ui,
            web_accessible_resources,
            content_security_policy,
        })
    }
}

/// Background script configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundConfig {
    /// Service worker path (Manifest V3)
    pub service_worker: Option<String>,
    /// Background scripts (Manifest V2)
    pub scripts: Vec<String>,
    /// Whether the background page is persistent (V2 only)
    pub persistent: bool,
}

/// Options UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsUiConfig {
    /// Path to options page
    pub page: String,
    /// Whether to open in a new tab
    pub open_in_tab: bool,
}

/// Web accessible resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAccessibleResource {
    /// Resource paths
    pub resources: Vec<String>,
    /// URL patterns that can access these resources
    pub matches: Vec<String>,
}

/// Content Security Policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSecurityPolicyConfig {
    /// CSP for extension pages
    pub extension_pages: Option<String>,
    /// CSP for sandboxed pages
    pub sandbox: Option<String>,
}

// Raw manifest structure for parsing
#[derive(Debug, Deserialize)]
struct RawManifest {
    name: String,
    short_name: Option<String>,
    version: String,
    manifest_version: u32,
    description: Option<String>,
    author: Option<String>,
    homepage_url: Option<String>,
    #[serde(default)]
    icons: HashMap<String, String>,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default)]
    optional_permissions: Vec<String>,
    #[serde(default)]
    host_permissions: Vec<String>,
    action: Option<RawBrowserAction>,
    browser_action: Option<RawBrowserAction>,
    #[serde(default)]
    content_scripts: Vec<RawContentScript>,
    background: Option<RawBackground>,
    options_page: Option<String>,
    options_ui: Option<RawOptionsUi>,
    #[serde(default)]
    web_accessible_resources: Vec<RawWebAccessibleResource>,
    content_security_policy: Option<RawCsp>,
}

#[derive(Debug, Deserialize)]
struct RawBrowserAction {
    default_icon: Option<HashMap<u32, String>>,
    default_title: Option<String>,
    default_popup: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawContentScript {
    #[serde(default)]
    js: Vec<String>,
    #[serde(default)]
    css: Vec<String>,
    #[serde(default)]
    matches: Vec<String>,
    #[serde(default)]
    exclude_matches: Vec<String>,
    run_at: Option<String>,
    #[serde(default)]
    all_frames: bool,
}

#[derive(Debug, Deserialize)]
struct RawBackground {
    service_worker: Option<String>,
    #[serde(default)]
    scripts: Vec<String>,
    #[serde(default)]
    persistent: bool,
}

#[derive(Debug, Deserialize)]
struct RawOptionsUi {
    page: String,
    #[serde(default)]
    open_in_tab: bool,
}

#[derive(Debug, Deserialize)]
struct RawWebAccessibleResource {
    #[serde(default)]
    resources: Vec<String>,
    #[serde(default)]
    matches: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawCsp {
    String(String),
    Object {
        extension_pages: Option<String>,
        sandbox: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_manifest() {
        let json = r#"{
            "name": "Test Extension",
            "version": "1.0.0",
            "manifest_version": 3
        }"#;

        let manifest = ExtensionManifest::parse(json).unwrap();
        assert_eq!(manifest.name, "Test Extension");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.manifest_version, 3);
    }

    #[test]
    fn test_parse_full_manifest() {
        let json = r#"{
            "name": "Full Extension",
            "short_name": "Full Ext",
            "version": "2.0.0",
            "manifest_version": 3,
            "description": "A full test extension",
            "author": "Test Author",
            "homepage_url": "https://example.com",
            "icons": {
                "16": "icon16.png",
                "48": "icon48.png",
                "128": "icon128.png"
            },
            "permissions": ["storage", "tabs"],
            "host_permissions": ["https://example.com/*"],
            "action": {
                "default_title": "Click me",
                "default_popup": "popup.html"
            },
            "content_scripts": [{
                "matches": ["https://*/*"],
                "js": ["content.js"],
                "run_at": "document_end"
            }],
            "background": {
                "service_worker": "background.js"
            }
        }"#;

        let manifest = ExtensionManifest::parse(json).unwrap();
        assert_eq!(manifest.display_name, "Full Ext");
        assert_eq!(manifest.icons.len(), 3);
        assert!(manifest.permissions.contains(&crate::permissions::Permission::Storage));
        assert!(manifest.browser_action.is_some());
        assert_eq!(manifest.content_scripts.len(), 1);
        assert!(manifest.background.is_some());
    }

    #[test]
    fn test_parse_manifest_v2() {
        let json = r#"{
            "name": "V2 Extension",
            "version": "1.0.0",
            "manifest_version": 2,
            "browser_action": {
                "default_title": "V2 Action"
            },
            "background": {
                "scripts": ["background.js"],
                "persistent": false
            }
        }"#;

        let manifest = ExtensionManifest::parse(json).unwrap();
        assert_eq!(manifest.manifest_version, 2);
        assert!(manifest.browser_action.is_some());
        assert_eq!(
            manifest.background.as_ref().unwrap().scripts,
            vec!["background.js"]
        );
    }

    #[test]
    fn test_parse_missing_name() {
        let json = r#"{
            "version": "1.0.0",
            "manifest_version": 3
        }"#;

        let result = ExtensionManifest::parse(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unsupported_version() {
        let json = r#"{
            "name": "Test",
            "version": "1.0.0",
            "manifest_version": 1
        }"#;

        let result = ExtensionManifest::parse(json);
        assert!(matches!(
            result,
            Err(ManifestParseError::UnsupportedVersion(1))
        ));
    }

    #[test]
    fn test_content_script_run_at() {
        let json = r#"{
            "name": "Test",
            "version": "1.0.0",
            "manifest_version": 3,
            "content_scripts": [
                {"matches": ["*://*/*"], "js": ["a.js"], "run_at": "document_start"},
                {"matches": ["*://*/*"], "js": ["b.js"], "run_at": "document_end"},
                {"matches": ["*://*/*"], "js": ["c.js"]}
            ]
        }"#;

        let manifest = ExtensionManifest::parse(json).unwrap();
        assert_eq!(manifest.content_scripts.len(), 3);
        assert_eq!(
            manifest.content_scripts[0].run_at,
            ContentScriptRunAt::DocumentStart
        );
        assert_eq!(
            manifest.content_scripts[1].run_at,
            ContentScriptRunAt::DocumentEnd
        );
        assert_eq!(
            manifest.content_scripts[2].run_at,
            ContentScriptRunAt::DocumentIdle
        );
    }
}
