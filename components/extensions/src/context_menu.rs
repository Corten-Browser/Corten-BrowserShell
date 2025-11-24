//! Context Menu API
//!
//! Provides context menu contribution points for extensions.

use crate::types::ExtensionId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context in which a menu can appear
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MenuContext {
    /// All contexts
    All,
    /// Right-click on page background
    Page,
    /// Right-click on a frame
    Frame,
    /// Right-click on selected text
    Selection,
    /// Right-click on a link
    Link,
    /// Right-click on an editable element
    Editable,
    /// Right-click on an image
    Image,
    /// Right-click on a video
    Video,
    /// Right-click on an audio element
    Audio,
    /// Browser action context menu
    BrowserAction,
    /// Page action context menu
    PageAction,
    /// Launcher (for Chrome apps)
    Launcher,
}

impl MenuContext {
    /// Parse from manifest string
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "all" => Some(MenuContext::All),
            "page" => Some(MenuContext::Page),
            "frame" => Some(MenuContext::Frame),
            "selection" => Some(MenuContext::Selection),
            "link" => Some(MenuContext::Link),
            "editable" => Some(MenuContext::Editable),
            "image" => Some(MenuContext::Image),
            "video" => Some(MenuContext::Video),
            "audio" => Some(MenuContext::Audio),
            "browser_action" => Some(MenuContext::BrowserAction),
            "page_action" => Some(MenuContext::PageAction),
            "launcher" => Some(MenuContext::Launcher),
            _ => None,
        }
    }
}

/// Type of context menu item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextMenuItemType {
    /// Normal clickable item
    Normal,
    /// Checkbox item
    Checkbox,
    /// Radio button item
    Radio,
    /// Separator line
    Separator,
}

impl Default for ContextMenuItemType {
    fn default() -> Self {
        Self::Normal
    }
}

/// Unique identifier for a context menu item
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextMenuItemId(String);

impl ContextMenuItemId {
    /// Create a new context menu item ID
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Get the ID string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A context menu item contributed by an extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMenuItem {
    /// Unique identifier within the extension
    pub id: ContextMenuItemId,
    /// Parent item ID (for submenus)
    pub parent_id: Option<ContextMenuItemId>,
    /// Display title (%s is replaced with selection)
    pub title: String,
    /// Type of item
    pub item_type: ContextMenuItemType,
    /// Contexts where this item appears
    pub contexts: Vec<MenuContext>,
    /// Whether the item is visible
    pub visible: bool,
    /// Whether the item is enabled
    pub enabled: bool,
    /// Whether checkbox/radio is checked
    pub checked: bool,
    /// URL patterns where this item should appear
    pub document_url_patterns: Vec<String>,
    /// URL patterns where this item should NOT appear
    pub exclude_document_url_patterns: Vec<String>,
    /// Target URL patterns (for links, images, etc.)
    pub target_url_patterns: Vec<String>,
}

impl ContextMenuItem {
    /// Create a new context menu item
    pub fn new(id: String, title: String) -> Self {
        Self {
            id: ContextMenuItemId::new(id),
            parent_id: None,
            title,
            item_type: ContextMenuItemType::Normal,
            contexts: vec![MenuContext::All],
            visible: true,
            enabled: true,
            checked: false,
            document_url_patterns: Vec::new(),
            exclude_document_url_patterns: Vec::new(),
            target_url_patterns: Vec::new(),
        }
    }

    /// Create a separator
    pub fn separator(id: String) -> Self {
        Self {
            id: ContextMenuItemId::new(id),
            parent_id: None,
            title: String::new(),
            item_type: ContextMenuItemType::Separator,
            contexts: vec![MenuContext::All],
            visible: true,
            enabled: true,
            checked: false,
            document_url_patterns: Vec::new(),
            exclude_document_url_patterns: Vec::new(),
            target_url_patterns: Vec::new(),
        }
    }

    /// Set parent (for submenu)
    pub fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_id = Some(ContextMenuItemId::new(parent_id));
        self
    }

    /// Set contexts
    pub fn with_contexts(mut self, contexts: Vec<MenuContext>) -> Self {
        self.contexts = contexts;
        self
    }

    /// Set type
    pub fn with_type(mut self, item_type: ContextMenuItemType) -> Self {
        self.item_type = item_type;
        self
    }

    /// Set document URL patterns
    pub fn with_document_patterns(mut self, patterns: Vec<String>) -> Self {
        self.document_url_patterns = patterns;
        self
    }

    /// Check if this item should appear in a given context
    pub fn matches_context(&self, context: MenuContext) -> bool {
        self.contexts.contains(&MenuContext::All) || self.contexts.contains(&context)
    }

    /// Check if this item should appear for a given document URL
    pub fn matches_document_url(&self, url: &str) -> bool {
        // If no patterns specified, match all
        if self.document_url_patterns.is_empty() {
            return true;
        }

        // Check include patterns
        let matches_include = self
            .document_url_patterns
            .iter()
            .any(|p| url_matches_pattern(url, p));

        // Check exclude patterns
        let matches_exclude = self
            .exclude_document_url_patterns
            .iter()
            .any(|p| url_matches_pattern(url, p));

        matches_include && !matches_exclude
    }
}

/// Simple URL pattern matching
fn url_matches_pattern(url: &str, pattern: &str) -> bool {
    if pattern == "<all_urls>" {
        return true;
    }

    // Simple wildcard matching
    let regex_pattern = pattern
        .replace('.', r"\.")
        .replace('*', ".*")
        .replace('?', ".");

    regex::Regex::new(&format!("^{}$", regex_pattern))
        .map(|re| re.is_match(url))
        .unwrap_or(false)
}

/// Context Menu API
///
/// Manages context menu items for all extensions
pub struct ContextMenuApi {
    /// Items by extension (extension_id -> items)
    items: HashMap<ExtensionId, Vec<ContextMenuItem>>,
}

impl ContextMenuApi {
    /// Create a new ContextMenuApi
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /// Add a context menu item for an extension
    pub fn add_item(&mut self, extension_id: ExtensionId, item: ContextMenuItem) {
        self.items
            .entry(extension_id)
            .or_insert_with(Vec::new)
            .push(item);
    }

    /// Update an existing context menu item
    pub fn update_item(
        &mut self,
        extension_id: ExtensionId,
        item_id: &ContextMenuItemId,
        update: impl FnOnce(&mut ContextMenuItem),
    ) -> bool {
        if let Some(items) = self.items.get_mut(&extension_id) {
            if let Some(item) = items.iter_mut().find(|i| &i.id == item_id) {
                update(item);
                return true;
            }
        }
        false
    }

    /// Remove a context menu item
    pub fn remove_item(&mut self, extension_id: ExtensionId, item_id: &ContextMenuItemId) -> bool {
        if let Some(items) = self.items.get_mut(&extension_id) {
            let initial_len = items.len();
            items.retain(|i| &i.id != item_id);
            return items.len() < initial_len;
        }
        false
    }

    /// Remove all items for an extension
    pub fn remove_all_for_extension(&mut self, extension_id: ExtensionId) {
        self.items.remove(&extension_id);
    }

    /// Get all items for an extension
    pub fn get_items(&self, extension_id: ExtensionId) -> Option<&[ContextMenuItem]> {
        self.items.get(&extension_id).map(|v| v.as_slice())
    }

    /// Get items that should appear in a given context
    pub fn get_items_for_context(
        &self,
        context: MenuContext,
        document_url: &str,
    ) -> Vec<(ExtensionId, &ContextMenuItem)> {
        let mut result = Vec::new();

        for (ext_id, items) in &self.items {
            for item in items {
                if item.visible
                    && item.matches_context(context)
                    && item.matches_document_url(document_url)
                {
                    result.push((*ext_id, item));
                }
            }
        }

        result
    }

    /// Build menu tree for an extension (root items and their children)
    pub fn get_menu_tree(&self, extension_id: ExtensionId) -> Vec<MenuTreeNode<'_>> {
        let Some(items) = self.items.get(&extension_id) else {
            return Vec::new();
        };

        // Find root items (no parent)
        let root_items: Vec<&ContextMenuItem> =
            items.iter().filter(|i| i.parent_id.is_none()).collect();

        // Build tree recursively
        root_items
            .into_iter()
            .map(|item| self.build_tree_node(item, items))
            .collect()
    }

    fn build_tree_node<'a>(
        &self,
        item: &'a ContextMenuItem,
        all_items: &'a [ContextMenuItem],
    ) -> MenuTreeNode<'a> {
        let children: Vec<MenuTreeNode> = all_items
            .iter()
            .filter(|i| i.parent_id.as_ref() == Some(&item.id))
            .map(|child| self.build_tree_node(child, all_items))
            .collect();

        MenuTreeNode { item, children }
    }
}

impl Default for ContextMenuApi {
    fn default() -> Self {
        Self::new()
    }
}

/// A node in the menu tree (for hierarchical menus)
#[derive(Debug)]
pub struct MenuTreeNode<'a> {
    /// The menu item
    pub item: &'a ContextMenuItem,
    /// Child items (for submenus)
    pub children: Vec<MenuTreeNode<'a>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_item_creation() {
        let item = ContextMenuItem::new("my-item".to_string(), "My Item".to_string())
            .with_contexts(vec![MenuContext::Selection, MenuContext::Link]);

        assert_eq!(item.id.as_str(), "my-item");
        assert_eq!(item.title, "My Item");
        assert!(item.matches_context(MenuContext::Selection));
        assert!(item.matches_context(MenuContext::Link));
        assert!(!item.matches_context(MenuContext::Image));
    }

    #[test]
    fn test_separator() {
        let sep = ContextMenuItem::separator("sep1".to_string());
        assert_eq!(sep.item_type, ContextMenuItemType::Separator);
        assert!(sep.title.is_empty());
    }

    #[test]
    fn test_context_menu_api() {
        let mut api = ContextMenuApi::new();
        let ext_id = ExtensionId::from_string("test-ext");

        let item1 = ContextMenuItem::new("item1".to_string(), "Item 1".to_string());
        let item2 = ContextMenuItem::new("item2".to_string(), "Item 2".to_string());

        api.add_item(ext_id, item1);
        api.add_item(ext_id, item2);

        let items = api.get_items(ext_id).unwrap();
        assert_eq!(items.len(), 2);

        api.remove_item(ext_id, &ContextMenuItemId::new("item1".to_string()));
        let items = api.get_items(ext_id).unwrap();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_context_matching() {
        let item = ContextMenuItem::new("test".to_string(), "Test".to_string())
            .with_contexts(vec![MenuContext::All]);

        // All context should match everything
        assert!(item.matches_context(MenuContext::Page));
        assert!(item.matches_context(MenuContext::Selection));
        assert!(item.matches_context(MenuContext::Link));
    }

    #[test]
    fn test_document_url_matching() {
        let item = ContextMenuItem::new("test".to_string(), "Test".to_string())
            .with_document_patterns(vec!["https://example.com/*".to_string()]);

        assert!(item.matches_document_url("https://example.com/page"));
        assert!(!item.matches_document_url("https://other.com/page"));
    }

    #[test]
    fn test_menu_tree() {
        let mut api = ContextMenuApi::new();
        let ext_id = ExtensionId::from_string("test-ext");

        let parent = ContextMenuItem::new("parent".to_string(), "Parent".to_string());
        let child1 = ContextMenuItem::new("child1".to_string(), "Child 1".to_string())
            .with_parent("parent".to_string());
        let child2 = ContextMenuItem::new("child2".to_string(), "Child 2".to_string())
            .with_parent("parent".to_string());

        api.add_item(ext_id, parent);
        api.add_item(ext_id, child1);
        api.add_item(ext_id, child2);

        let tree = api.get_menu_tree(ext_id);
        assert_eq!(tree.len(), 1); // One root item
        assert_eq!(tree[0].children.len(), 2); // Two children
    }
}
