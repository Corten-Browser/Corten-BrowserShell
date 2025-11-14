//! Unit tests for message types
//!
//! Tests ComponentMessage, ComponentResponse, and MessagePriority enums.

use message_bus::{ComponentMessage, ComponentResponse, MessagePriority};
use shared_types::{KeyboardShortcut, TabId, WindowConfig, WindowId};

#[test]
fn test_component_message_create_window() {
    let config = WindowConfig::default();
    let msg = ComponentMessage::CreateWindow(config.clone());

    match msg {
        ComponentMessage::CreateWindow(c) => {
            assert_eq!(c.title, config.title);
        }
        _ => panic!("Expected CreateWindow variant"),
    }
}

#[test]
fn test_component_message_close_window() {
    let window_id = WindowId::new();
    let msg = ComponentMessage::CloseWindow(window_id);

    match msg {
        ComponentMessage::CloseWindow(id) => {
            assert_eq!(id, window_id);
        }
        _ => panic!("Expected CloseWindow variant"),
    }
}

#[test]
fn test_component_message_create_tab() {
    let window_id = WindowId::new();
    let url = Some("https://example.com".to_string());
    let msg = ComponentMessage::CreateTab(window_id, url.clone());

    match msg {
        ComponentMessage::CreateTab(wid, u) => {
            assert_eq!(wid, window_id);
            assert_eq!(u, url);
        }
        _ => panic!("Expected CreateTab variant"),
    }
}

#[test]
fn test_component_message_close_tab() {
    let tab_id = TabId::new();
    let msg = ComponentMessage::CloseTab(tab_id);

    match msg {
        ComponentMessage::CloseTab(id) => {
            assert_eq!(id, tab_id);
        }
        _ => panic!("Expected CloseTab variant"),
    }
}

#[test]
fn test_component_message_navigate_tab() {
    let tab_id = TabId::new();
    let url = "https://example.com".to_string();
    let msg = ComponentMessage::NavigateTab(tab_id, url.clone());

    match msg {
        ComponentMessage::NavigateTab(tid, u) => {
            assert_eq!(tid, tab_id);
            assert_eq!(u, url);
        }
        _ => panic!("Expected NavigateTab variant"),
    }
}

#[test]
fn test_component_message_keyboard_shortcut() {
    let shortcut = KeyboardShortcut::CtrlT;
    let msg = ComponentMessage::KeyboardShortcut(shortcut);

    match msg {
        ComponentMessage::KeyboardShortcut(s) => {
            assert_eq!(s, shortcut);
        }
        _ => panic!("Expected KeyboardShortcut variant"),
    }
}

#[test]
fn test_component_response_window_created() {
    let window_id = WindowId::new();
    let response = ComponentResponse::WindowCreated(window_id);

    match response {
        ComponentResponse::WindowCreated(id) => {
            assert_eq!(id, window_id);
        }
        _ => panic!("Expected WindowCreated variant"),
    }
}

#[test]
fn test_component_response_tab_created() {
    let tab_id = TabId::new();
    let response = ComponentResponse::TabCreated(tab_id);

    match response {
        ComponentResponse::TabCreated(id) => {
            assert_eq!(id, tab_id);
        }
        _ => panic!("Expected TabCreated variant"),
    }
}

#[test]
fn test_component_response_navigation_started() {
    let tab_id = TabId::new();
    let response = ComponentResponse::NavigationStarted(tab_id);

    match response {
        ComponentResponse::NavigationStarted(id) => {
            assert_eq!(id, tab_id);
        }
        _ => panic!("Expected NavigationStarted variant"),
    }
}

#[test]
fn test_component_response_success() {
    let response = ComponentResponse::Success;

    match response {
        ComponentResponse::Success => {}
        _ => panic!("Expected Success variant"),
    }
}

#[test]
fn test_component_response_error() {
    let error_msg = "Test error".to_string();
    let response = ComponentResponse::Error(error_msg.clone());

    match response {
        ComponentResponse::Error(msg) => {
            assert_eq!(msg, error_msg);
        }
        _ => panic!("Expected Error variant"),
    }
}

#[test]
fn test_message_priority_ordering() {
    // Critical should be highest priority
    assert!(MessagePriority::Critical > MessagePriority::High);
    assert!(MessagePriority::High > MessagePriority::Normal);
    assert!(MessagePriority::Normal > MessagePriority::Low);
}

#[test]
fn test_message_priority_equality() {
    assert_eq!(MessagePriority::Critical, MessagePriority::Critical);
    assert_eq!(MessagePriority::High, MessagePriority::High);
    assert_eq!(MessagePriority::Normal, MessagePriority::Normal);
    assert_eq!(MessagePriority::Low, MessagePriority::Low);
}

#[test]
fn test_message_types_are_cloneable() {
    let window_id = WindowId::new();
    let msg = ComponentMessage::CloseWindow(window_id);
    let _msg_clone = msg.clone();

    let response = ComponentResponse::Success;
    let _response_clone = response.clone();

    let priority = MessagePriority::High;
    let _priority_clone = priority;
}

#[test]
fn test_message_types_are_debuggable() {
    let msg = ComponentMessage::CloseWindow(WindowId::new());
    let debug_str = format!("{:?}", msg);
    assert!(debug_str.contains("CloseWindow"));

    let response = ComponentResponse::Success;
    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("Success"));

    let priority = MessagePriority::Critical;
    let debug_str = format!("{:?}", priority);
    assert!(debug_str.contains("Critical"));
}
