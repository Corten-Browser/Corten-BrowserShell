// @validates: REQ-UI-004
//! Unit tests for Menu system

use ui_chrome::menu::{MenuSystem, MenuItem, MenuId};

#[test]
fn test_menu_system_creation() {
    let menu = MenuSystem::new();
    assert_eq!(menu.get_all_menus().len(), 0);
}

#[test]
fn test_menu_add_menu() {
    let mut menu = MenuSystem::new();
    let file_menu_id = menu.add_menu("File".to_string());

    assert!(menu.get_menu(file_menu_id).is_some());
    assert_eq!(menu.get_menu(file_menu_id).unwrap().label, "File");
}

#[test]
fn test_menu_add_menu_item() {
    let mut menu = MenuSystem::new();
    let file_menu_id = menu.add_menu("File".to_string());

    let item = MenuItem {
        label: "New Window".to_string(),
        shortcut: Some("Ctrl+N".to_string()),
        enabled: true,
        visible: true,
        separator_after: false,
    };

    let item_id = menu.add_menu_item(file_menu_id, item.clone());
    assert!(item_id.is_ok());

    let menu_data = menu.get_menu(file_menu_id).unwrap();
    assert_eq!(menu_data.items.len(), 1);
}

#[test]
fn test_menu_add_item_to_nonexistent_menu() {
    let mut menu = MenuSystem::new();
    let item = MenuItem {
        label: "Test".to_string(),
        shortcut: None,
        enabled: true,
        visible: true,
        separator_after: false,
    };

    let result = menu.add_menu_item(999, item);
    assert!(result.is_err());
}

#[test]
fn test_menu_set_item_enabled() {
    let mut menu = MenuSystem::new();
    let file_menu_id = menu.add_menu("File".to_string());

    let item = MenuItem {
        label: "Save".to_string(),
        shortcut: Some("Ctrl+S".to_string()),
        enabled: true,
        visible: true,
        separator_after: false,
    };

    let item_id = menu.add_menu_item(file_menu_id, item).unwrap();

    let result = menu.set_item_enabled(file_menu_id, item_id, false);
    assert!(result.is_ok());

    let menu_data = menu.get_menu(file_menu_id).unwrap();
    assert!(!menu_data.items[0].enabled);
}

#[test]
fn test_menu_get_item_shortcut() {
    let mut menu = MenuSystem::new();
    let file_menu_id = menu.add_menu("File".to_string());

    let item = MenuItem {
        label: "Open".to_string(),
        shortcut: Some("Ctrl+O".to_string()),
        enabled: true,
        visible: true,
        separator_after: false,
    };

    menu.add_menu_item(file_menu_id, item).unwrap();

    let menu_data = menu.get_menu(file_menu_id).unwrap();
    assert_eq!(menu_data.items[0].shortcut, Some("Ctrl+O".to_string()));
}

#[test]
fn test_menu_remove_menu() {
    let mut menu = MenuSystem::new();
    let file_menu_id = menu.add_menu("File".to_string());

    assert!(menu.get_menu(file_menu_id).is_some());

    let result = menu.remove_menu(file_menu_id);
    assert!(result.is_ok());
    assert!(menu.get_menu(file_menu_id).is_none());
}

#[test]
fn test_menu_get_all_menus() {
    let mut menu = MenuSystem::new();
    menu.add_menu("File".to_string());
    menu.add_menu("Edit".to_string());
    menu.add_menu("View".to_string());

    let all_menus = menu.get_all_menus();
    assert_eq!(all_menus.len(), 3);
}

#[test]
fn test_menu_item_separator() {
    let item = MenuItem {
        label: "---".to_string(),
        shortcut: None,
        enabled: false,
        visible: true,
        separator_after: true,
    };

    assert!(item.separator_after);
}
