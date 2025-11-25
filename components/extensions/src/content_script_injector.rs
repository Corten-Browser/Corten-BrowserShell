//! Content script injection system for extensions.
//!
//! This module handles injecting JavaScript and CSS into web pages
//! based on URL patterns and run timing specified in extension manifests.

use crate::types::{ContentScript, ContentScriptRunAt, ExtensionId, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

/// A script to be injected into a page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionScript {
    /// Extension ID
    pub extension_id: ExtensionId,
    /// Script source code
    pub source: String,
    /// Script type (JavaScript or CSS)
    pub script_type: InjectionScriptType,
    /// When to inject
    pub run_at: ContentScriptRunAt,
    /// Whether to run in all frames
    pub all_frames: bool,
}

/// Type of script to inject
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InjectionScriptType {
    /// JavaScript code
    JavaScript,
    /// CSS stylesheet
    CSS,
}

/// Content script injection manager
pub struct ContentScriptInjector {
    /// Registered content scripts by extension ID
    scripts: Arc<RwLock<HashMap<ExtensionId, Vec<ContentScript>>>>,
    /// Script cache (URL pattern -> compiled scripts)
    cache: Arc<RwLock<HashMap<String, Vec<InjectionScript>>>>,
}

impl ContentScriptInjector {
    /// Create a new content script injector
    pub fn new() -> Self {
        Self {
            scripts: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register content scripts for an extension
    pub async fn register_extension(
        &self,
        extension_id: ExtensionId,
        content_scripts: Vec<ContentScript>,
    ) -> Result<()> {
        let mut scripts = self.scripts.write().await;
        scripts.insert(extension_id, content_scripts);

        // Clear cache when new scripts are registered
        let mut cache = self.cache.write().await;
        cache.clear();

        Ok(())
    }

    /// Unregister content scripts for an extension
    pub async fn unregister_extension(&self, extension_id: ExtensionId) -> Result<()> {
        let mut scripts = self.scripts.write().await;
        scripts.remove(&extension_id);

        // Clear cache
        let mut cache = self.cache.write().await;
        cache.clear();

        Ok(())
    }

    /// Get scripts that should be injected for a given URL
    pub async fn get_scripts_for_url(&self, url: &Url) -> Result<Vec<InjectionScript>> {
        let url_str = url.as_str();

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(url_str) {
                return Ok(cached.clone());
            }
        }

        // Compute matching scripts
        let scripts = self.scripts.read().await;
        let mut matching_scripts = Vec::new();

        for (extension_id, content_scripts) in scripts.iter() {
            for content_script in content_scripts {
                if self.matches_url(content_script, url) {
                    // Add JavaScript scripts
                    for js_file in &content_script.js {
                        matching_scripts.push(InjectionScript {
                            extension_id: *extension_id,
                            source: js_file.clone(), // In production, would load file content
                            script_type: InjectionScriptType::JavaScript,
                            run_at: content_script.run_at,
                            all_frames: content_script.all_frames,
                        });
                    }

                    // Add CSS scripts
                    for css_file in &content_script.css {
                        matching_scripts.push(InjectionScript {
                            extension_id: *extension_id,
                            source: css_file.clone(), // In production, would load file content
                            script_type: InjectionScriptType::CSS,
                            run_at: content_script.run_at,
                            all_frames: content_script.all_frames,
                        });
                    }
                }
            }
        }

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            cache.insert(url_str.to_string(), matching_scripts.clone());
        }

        Ok(matching_scripts)
    }

    /// Get scripts that should run at a specific timing
    pub async fn get_scripts_for_timing(
        &self,
        url: &Url,
        run_at: ContentScriptRunAt,
    ) -> Result<Vec<InjectionScript>> {
        let all_scripts = self.get_scripts_for_url(url).await?;
        Ok(all_scripts
            .into_iter()
            .filter(|s| s.run_at == run_at)
            .collect())
    }

    /// Check if a content script matches a URL
    fn matches_url(&self, content_script: &ContentScript, url: &Url) -> bool {
        // Check if URL matches any of the match patterns
        let has_match = content_script
            .matches
            .iter()
            .any(|pattern| pattern.matches(url.as_str()));

        if !has_match {
            return false;
        }

        // Check if URL is excluded
        let is_excluded = content_script
            .exclude_matches
            .iter()
            .any(|pattern| pattern.matches(url.as_str()));

        !is_excluded
    }

    /// Execute script injection for a page load
    pub async fn inject_for_page_load(
        &self,
        url: &Url,
        run_at: ContentScriptRunAt,
        is_main_frame: bool,
    ) -> Result<Vec<InjectionScript>> {
        let scripts = self.get_scripts_for_timing(url, run_at).await?;

        // Filter by frame context
        let filtered = if is_main_frame {
            scripts
        } else {
            scripts
                .into_iter()
                .filter(|s| s.all_frames)
                .collect()
        };

        tracing::debug!(
            url = %url,
            run_at = ?run_at,
            is_main_frame = is_main_frame,
            script_count = filtered.len(),
            "Injecting content scripts"
        );

        Ok(filtered)
    }

    /// Get all registered extensions
    pub async fn list_extensions(&self) -> Vec<ExtensionId> {
        let scripts = self.scripts.read().await;
        scripts.keys().copied().collect()
    }

    /// Get content scripts for an extension
    pub async fn get_extension_scripts(&self, extension_id: ExtensionId) -> Option<Vec<ContentScript>> {
        let scripts = self.scripts.read().await;
        scripts.get(&extension_id).cloned()
    }

    /// Clear all cached script lookups
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

impl Default for ContentScriptInjector {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to build injection context
#[derive(Debug, Clone)]
pub struct InjectionContext {
    /// URL being loaded
    pub url: Url,
    /// Whether this is the main frame
    pub is_main_frame: bool,
    /// Current document readiness state
    pub document_state: DocumentState,
    /// Frame ID
    pub frame_id: u64,
}

/// Document readiness state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentState {
    /// Document is loading (parsing)
    Loading,
    /// DOM is ready but resources are still loading
    Interactive,
    /// Document and all resources are loaded
    Complete,
}

impl InjectionContext {
    /// Create a new injection context
    pub fn new(url: Url, is_main_frame: bool, frame_id: u64) -> Self {
        Self {
            url,
            is_main_frame,
            document_state: DocumentState::Loading,
            frame_id,
        }
    }

    /// Determine which content scripts should run now
    pub fn should_inject_timing(&self) -> Option<ContentScriptRunAt> {
        match self.document_state {
            DocumentState::Loading => Some(ContentScriptRunAt::DocumentStart),
            DocumentState::Interactive => Some(ContentScriptRunAt::DocumentEnd),
            DocumentState::Complete => Some(ContentScriptRunAt::DocumentIdle),
        }
    }

    /// Update document state
    pub fn set_document_state(&mut self, state: DocumentState) {
        self.document_state = state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContentScript, ContentScriptMatch};

    fn create_test_script(pattern: &str) -> ContentScript {
        ContentScript {
            js: vec!["test.js".to_string()],
            css: vec!["test.css".to_string()],
            matches: vec![ContentScriptMatch::new(pattern.to_string())],
            exclude_matches: vec![],
            run_at: ContentScriptRunAt::DocumentIdle,
            all_frames: false,
        }
    }

    #[tokio::test]
    async fn test_injector_creation() {
        let injector = ContentScriptInjector::new();
        let extensions = injector.list_extensions().await;
        assert!(extensions.is_empty());
    }

    #[tokio::test]
    async fn test_register_extension() {
        let injector = ContentScriptInjector::new();
        let ext_id = ExtensionId::new();
        let scripts = vec![create_test_script("https://example.com/*")];

        injector.register_extension(ext_id, scripts.clone()).await.unwrap();

        let registered = injector.get_extension_scripts(ext_id).await;
        assert!(registered.is_some());
        assert_eq!(registered.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_unregister_extension() {
        let injector = ContentScriptInjector::new();
        let ext_id = ExtensionId::new();
        let scripts = vec![create_test_script("https://example.com/*")];

        injector.register_extension(ext_id, scripts).await.unwrap();
        injector.unregister_extension(ext_id).await.unwrap();

        let registered = injector.get_extension_scripts(ext_id).await;
        assert!(registered.is_none());
    }

    #[tokio::test]
    async fn test_get_scripts_for_url() {
        let injector = ContentScriptInjector::new();
        let ext_id = ExtensionId::new();
        let scripts = vec![create_test_script("https://example.com/*")];

        injector.register_extension(ext_id, scripts).await.unwrap();

        let url = Url::parse("https://example.com/page").unwrap();
        let matching = injector.get_scripts_for_url(&url).await.unwrap();

        // Should have 1 JS and 1 CSS script
        assert_eq!(matching.len(), 2);
    }

    #[tokio::test]
    async fn test_url_pattern_matching() {
        let injector = ContentScriptInjector::new();
        let ext_id = ExtensionId::new();

        let mut script = create_test_script("https://example.com/*");
        script.exclude_matches = vec![ContentScriptMatch::new("https://example.com/exclude".to_string())];

        injector.register_extension(ext_id, vec![script]).await.unwrap();

        let url1 = Url::parse("https://example.com/page").unwrap();
        let matching1 = injector.get_scripts_for_url(&url1).await.unwrap();
        assert_eq!(matching1.len(), 2); // JS + CSS

        let url2 = Url::parse("https://example.com/exclude").unwrap();
        let matching2 = injector.get_scripts_for_url(&url2).await.unwrap();
        assert_eq!(matching2.len(), 0); // Excluded
    }

    #[tokio::test]
    async fn test_get_scripts_for_timing() {
        let injector = ContentScriptInjector::new();
        let ext_id = ExtensionId::new();

        let mut script1 = create_test_script("https://example.com/*");
        script1.run_at = ContentScriptRunAt::DocumentStart;

        let mut script2 = create_test_script("https://example.com/*");
        script2.run_at = ContentScriptRunAt::DocumentIdle;

        injector.register_extension(ext_id, vec![script1, script2]).await.unwrap();

        let url = Url::parse("https://example.com/page").unwrap();
        let start_scripts = injector
            .get_scripts_for_timing(&url, ContentScriptRunAt::DocumentStart)
            .await
            .unwrap();

        assert_eq!(start_scripts.len(), 2); // JS + CSS from first script
    }

    #[tokio::test]
    async fn test_inject_for_page_load() {
        let injector = ContentScriptInjector::new();
        let ext_id = ExtensionId::new();

        let mut script = create_test_script("https://example.com/*");
        script.all_frames = false;

        injector.register_extension(ext_id, vec![script]).await.unwrap();

        let url = Url::parse("https://example.com/page").unwrap();

        // Main frame should get scripts
        let main_frame = injector
            .inject_for_page_load(&url, ContentScriptRunAt::DocumentIdle, true)
            .await
            .unwrap();
        assert_eq!(main_frame.len(), 2);

        // Sub-frame should not (all_frames = false)
        let sub_frame = injector
            .inject_for_page_load(&url, ContentScriptRunAt::DocumentIdle, false)
            .await
            .unwrap();
        assert_eq!(sub_frame.len(), 0);
    }

    #[test]
    fn test_injection_context() {
        let url = Url::parse("https://example.com").unwrap();
        let mut context = InjectionContext::new(url, true, 1);

        assert_eq!(context.document_state, DocumentState::Loading);
        assert_eq!(context.should_inject_timing(), Some(ContentScriptRunAt::DocumentStart));

        context.set_document_state(DocumentState::Interactive);
        assert_eq!(context.should_inject_timing(), Some(ContentScriptRunAt::DocumentEnd));

        context.set_document_state(DocumentState::Complete);
        assert_eq!(context.should_inject_timing(), Some(ContentScriptRunAt::DocumentIdle));
    }
}
