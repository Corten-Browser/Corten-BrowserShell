// @validates: REQ-UI-002
//! Unit tests for Tab Bar widget

use ui_chrome::widgets::tab_bar::{TabBar, TabInfo};

#[test]
fn test_tab_bar_creation() {
    let bar = TabBar::new();
    assert_eq!(bar.tab_count(), 0);
    assert!(bar.active_tab().is_none());
}

#[test]
fn test_tab_bar_add_tab() {
    let mut bar = TabBar::new();
    let info = TabInfo {
        title: "Test Tab".to_string(),
        url: Some("https://example.com".to_string()),
        favicon: None,
        loading: false,
    };

    let tab_id = bar.add_tab(info.clone());
    assert_eq!(bar.tab_count(), 1);
    assert_eq!(bar.active_tab(), Some(tab_id));
}

#[test]
fn test_tab_bar_close_tab() {
    let mut bar = TabBar::new();
    let info = TabInfo {
        title: "Test Tab".to_string(),
        url: None,
        favicon: None,
        loading: false,
    };

    let tab_id = bar.add_tab(info);
    assert_eq!(bar.tab_count(), 1);

    let result = bar.close_tab(tab_id);
    assert!(result.is_ok());
    assert_eq!(bar.tab_count(), 0);
}

#[test]
fn test_tab_bar_close_nonexistent_tab() {
    let mut bar = TabBar::new();
    let result = bar.close_tab(999);
    assert!(result.is_err());
}

#[test]
fn test_tab_bar_activate_tab() {
    let mut bar = TabBar::new();
    let info1 = TabInfo {
        title: "Tab 1".to_string(),
        url: None,
        favicon: None,
        loading: false,
    };
    let info2 = TabInfo {
        title: "Tab 2".to_string(),
        url: None,
        favicon: None,
        loading: false,
    };

    let tab1 = bar.add_tab(info1);
    let tab2 = bar.add_tab(info2);

    assert_eq!(bar.active_tab(), Some(tab2));

    let result = bar.set_active_tab(tab1);
    assert!(result.is_ok());
    assert_eq!(bar.active_tab(), Some(tab1));
}

#[test]
fn test_tab_bar_get_tab_info() {
    let mut bar = TabBar::new();
    let info = TabInfo {
        title: "Test Tab".to_string(),
        url: Some("https://example.com".to_string()),
        favicon: None,
        loading: false,
    };

    let tab_id = bar.add_tab(info.clone());
    let retrieved = bar.get_tab_info(tab_id);

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().title, "Test Tab");
}

#[test]
fn test_tab_bar_update_tab_info() {
    let mut bar = TabBar::new();
    let info = TabInfo {
        title: "Old Title".to_string(),
        url: None,
        favicon: None,
        loading: false,
    };

    let tab_id = bar.add_tab(info);

    let updated = TabInfo {
        title: "New Title".to_string(),
        url: Some("https://example.com".to_string()),
        favicon: None,
        loading: true,
    };

    let result = bar.update_tab_info(tab_id, updated);
    assert!(result.is_ok());

    let retrieved = bar.get_tab_info(tab_id).unwrap();
    assert_eq!(retrieved.title, "New Title");
    assert!(retrieved.loading);
}

#[test]
fn test_tab_bar_get_all_tabs() {
    let mut bar = TabBar::new();
    let info1 = TabInfo {
        title: "Tab 1".to_string(),
        url: None,
        favicon: None,
        loading: false,
    };
    let info2 = TabInfo {
        title: "Tab 2".to_string(),
        url: None,
        favicon: None,
        loading: false,
    };

    bar.add_tab(info1);
    bar.add_tab(info2);

    let tabs = bar.get_all_tabs();
    assert_eq!(tabs.len(), 2);
}
