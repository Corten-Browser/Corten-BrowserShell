// @implements: REQ-UI-004
//! Menu System
//!
//! Application menu structure with keyboard shortcuts.

use std::collections::HashMap;

/// Menu ID type
pub type MenuId = u64;

/// Menu item ID type
pub type MenuItemId = usize;

/// Menu item definition
#[derive(Debug, Clone, PartialEq)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub visible: bool,
    pub separator_after: bool,
}

/// Menu data
#[derive(Debug, Clone)]
pub struct Menu {
    pub label: String,
    pub items: Vec<MenuItem>,
}

/// Error types for menu operations
#[derive(Debug, Clone, PartialEq)]
pub enum MenuError {
    MenuNotFound,
    ItemNotFound,
    InvalidId,
}

impl std::fmt::Display for MenuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuError::MenuNotFound => write!(f, "Menu not found"),
            MenuError::ItemNotFound => write!(f, "Menu item not found"),
            MenuError::InvalidId => write!(f, "Invalid ID"),
        }
    }
}

impl std::error::Error for MenuError {}

/// Menu System state
#[derive(Debug, Clone)]
pub struct MenuSystem {
    menus: HashMap<MenuId, Menu>,
    next_menu_id: MenuId,
}

impl MenuSystem {
    /// Create a new Menu System
    pub fn new() -> Self {
        Self {
            menus: HashMap::new(),
            next_menu_id: 1,
        }
    }

    /// Add a new menu
    pub fn add_menu(&mut self, label: String) -> MenuId {
        let menu_id = self.next_menu_id;
        self.next_menu_id += 1;

        let menu = Menu {
            label,
            items: Vec::new(),
        };

        self.menus.insert(menu_id, menu);
        menu_id
    }

    /// Get menu by ID
    pub fn get_menu(&self, menu_id: MenuId) -> Option<&Menu> {
        self.menus.get(&menu_id)
    }

    /// Add menu item to a menu
    pub fn add_menu_item(&mut self, menu_id: MenuId, item: MenuItem) -> Result<MenuItemId, MenuError> {
        let menu = self.menus.get_mut(&menu_id).ok_or(MenuError::MenuNotFound)?;

        menu.items.push(item);
        Ok(menu.items.len() - 1)
    }

    /// Set menu item enabled state
    pub fn set_item_enabled(
        &mut self,
        menu_id: MenuId,
        item_id: MenuItemId,
        enabled: bool,
    ) -> Result<(), MenuError> {
        let menu = self.menus.get_mut(&menu_id).ok_or(MenuError::MenuNotFound)?;

        if item_id >= menu.items.len() {
            return Err(MenuError::ItemNotFound);
        }

        menu.items[item_id].enabled = enabled;
        Ok(())
    }

    /// Set menu item visible state
    pub fn set_item_visible(
        &mut self,
        menu_id: MenuId,
        item_id: MenuItemId,
        visible: bool,
    ) -> Result<(), MenuError> {
        let menu = self.menus.get_mut(&menu_id).ok_or(MenuError::MenuNotFound)?;

        if item_id >= menu.items.len() {
            return Err(MenuError::ItemNotFound);
        }

        menu.items[item_id].visible = visible;
        Ok(())
    }

    /// Remove a menu
    pub fn remove_menu(&mut self, menu_id: MenuId) -> Result<(), MenuError> {
        self.menus.remove(&menu_id).ok_or(MenuError::MenuNotFound)?;
        Ok(())
    }

    /// Get all menus
    pub fn get_all_menus(&self) -> Vec<(MenuId, &Menu)> {
        self.menus.iter().map(|(&id, menu)| (id, menu)).collect()
    }
}

impl Default for MenuSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_system_new_creates_empty() {
        let menu = MenuSystem::new();
        assert_eq!(menu.get_all_menus().len(), 0);
    }

    #[test]
    fn menu_system_add_menu_increments_id() {
        let mut menu = MenuSystem::new();
        let id1 = menu.add_menu("File".to_string());
        let id2 = menu.add_menu("Edit".to_string());
        assert_ne!(id1, id2);
    }

    #[test]
    fn menu_system_get_menu_returns_data() {
        let mut menu = MenuSystem::new();
        let id = menu.add_menu("Test".to_string());
        let menu_data = menu.get_menu(id).unwrap();
        assert_eq!(menu_data.label, "Test");
    }

    #[test]
    fn menu_item_creation() {
        let item = MenuItem {
            label: "Test".to_string(),
            shortcut: Some("Ctrl+T".to_string()),
            enabled: true,
            visible: true,
            separator_after: false,
        };

        assert_eq!(item.label, "Test");
        assert_eq!(item.shortcut, Some("Ctrl+T".to_string()));
        assert!(item.enabled);
    }

    #[test]
    fn menu_add_item_increases_item_count() {
        let mut menu = MenuSystem::new();
        let menu_id = menu.add_menu("File".to_string());

        let item = MenuItem {
            label: "New".to_string(),
            shortcut: None,
            enabled: true,
            visible: true,
            separator_after: false,
        };

        menu.add_menu_item(menu_id, item).unwrap();
        let menu_data = menu.get_menu(menu_id).unwrap();
        assert_eq!(menu_data.items.len(), 1);
    }
}
