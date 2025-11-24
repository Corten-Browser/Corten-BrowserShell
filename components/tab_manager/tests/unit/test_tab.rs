//! Unit tests for Tab struct

use shared_types::{ProcessId, RenderSurfaceId, TabId, WindowId};
use tab_manager::{Tab, TabInfo, TabLoadState};
use url::Url;

#[test]
fn test_tab_creation() {
    /**
     * Given a tab ID, window ID, and URL
     * When creating a new Tab
     * Then the tab should have correct initial state
     */
    let tab_id = TabId::new();
    let window_id = WindowId::new();
    let url = Some(Url::parse("https://example.com").unwrap());
    let render_surface = RenderSurfaceId::new();

    let tab = Tab {
        id: tab_id,
        window_id,
        title: String::from("Example Domain"),
        url: url.clone(),
        loading: false,
        can_go_back: false,
        can_go_forward: false,
        favicon: None,
        process_id: None,
        render_surface,
        is_private: false,
        load_state: TabLoadState::Unloaded,
    };

    assert_eq!(tab.id, tab_id);
    assert_eq!(tab.window_id, window_id);
    assert_eq!(tab.title, "Example Domain");
    assert_eq!(tab.url, url);
    assert_eq!(tab.loading, false);
    assert_eq!(tab.can_go_back, false);
    assert_eq!(tab.can_go_forward, false);
    assert_eq!(tab.favicon, None);
    assert_eq!(tab.process_id, None);
    assert_eq!(tab.is_private, false);
    assert_eq!(tab.load_state, TabLoadState::Unloaded);
}

#[test]
fn test_tab_with_process_id() {
    /**
     * Given a tab with an associated process
     * When the tab is created with a process ID
     * Then the process ID should be stored correctly
     */
    let tab_id = TabId::new();
    let window_id = WindowId::new();
    let process_id = ProcessId::new(1234);
    let render_surface = RenderSurfaceId::new();

    let tab = Tab {
        id: tab_id,
        window_id,
        title: String::new(),
        url: None,
        loading: false,
        can_go_back: false,
        can_go_forward: false,
        favicon: None,
        process_id: Some(process_id),
        render_surface,
        is_private: false,
        load_state: TabLoadState::Unloaded,
    };

    assert_eq!(tab.process_id, Some(process_id));
}

#[test]
fn test_tab_info_from_tab() {
    /**
     * Given a Tab struct
     * When converting to TabInfo
     * Then TabInfo should contain the correct fields
     */
    let tab_id = TabId::new();
    let window_id = WindowId::new();
    let url = Some(Url::parse("https://example.com").unwrap());
    let render_surface = RenderSurfaceId::new();

    let tab = Tab {
        id: tab_id,
        window_id,
        title: String::from("Example"),
        url: url.clone(),
        loading: true,
        can_go_back: true,
        can_go_forward: false,
        favicon: None,
        process_id: None,
        render_surface,
        is_private: false,
        load_state: TabLoadState::Loaded,
    };

    let info = TabInfo::from(&tab);

    assert_eq!(info.id, tab_id);
    assert_eq!(info.window_id, window_id);
    assert_eq!(info.title, "Example");
    assert_eq!(info.url, url);
    assert_eq!(info.loading, true);
    assert_eq!(info.can_go_back, true);
    assert_eq!(info.can_go_forward, false);
    assert_eq!(info.is_private, false);
    assert_eq!(info.load_state, TabLoadState::Loaded);
}
