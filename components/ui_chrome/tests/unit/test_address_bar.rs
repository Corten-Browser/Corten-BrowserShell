// @validates: REQ-UI-001
//! Unit tests for Address Bar widget

use ui_chrome::widgets::address_bar::{AddressBar, UrlValidationResult};

#[test]
fn test_address_bar_creation() {
    let bar = AddressBar::new();
    assert_eq!(bar.get_text(), "");
    assert!(!bar.is_loading());
}

#[test]
fn test_address_bar_set_text() {
    let mut bar = AddressBar::new();
    bar.set_text("https://example.com".to_string());
    assert_eq!(bar.get_text(), "https://example.com");
}

#[test]
fn test_address_bar_validate_url_valid() {
    let bar = AddressBar::new();
    let result = bar.validate_url("https://example.com");
    assert!(matches!(result, UrlValidationResult::Valid(_)));
}

#[test]
fn test_address_bar_validate_url_invalid() {
    let bar = AddressBar::new();
    let result = bar.validate_url("ht://invalid");
    assert!(matches!(result, UrlValidationResult::Invalid(_)));
}

#[test]
fn test_address_bar_validate_url_search_query() {
    let bar = AddressBar::new();
    let result = bar.validate_url("hello world");
    assert!(matches!(result, UrlValidationResult::SearchQuery(_)));
}

#[test]
fn test_address_bar_loading_state() {
    let mut bar = AddressBar::new();
    assert!(!bar.is_loading());

    bar.set_loading(true);
    assert!(bar.is_loading());

    bar.set_loading(false);
    assert!(!bar.is_loading());
}

#[test]
fn test_address_bar_security_indicator() {
    let mut bar = AddressBar::new();
    bar.set_text("https://example.com".to_string());
    assert!(bar.is_secure());

    bar.set_text("http://example.com".to_string());
    assert!(!bar.is_secure());
}

#[test]
fn test_address_bar_focus_state() {
    let mut bar = AddressBar::new();
    assert!(!bar.is_focused());

    bar.set_focused(true);
    assert!(bar.is_focused());

    bar.set_focused(false);
    assert!(!bar.is_focused());
}
