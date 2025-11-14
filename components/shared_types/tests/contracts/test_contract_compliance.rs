/// Contract compliance tests
///
/// These tests verify that all types defined in contracts/shared_types.yaml
/// are implemented and can be used as specified in the contract.
use shared_types::*;

#[test]
fn test_contract_window_id_exists() {
    // Contract specifies WindowId as newtype over uuid
    let _id: WindowId = WindowId::new();
}

#[test]
fn test_contract_tab_id_exists() {
    // Contract specifies TabId as newtype over uuid
    let _id: TabId = TabId::new();
}

#[test]
fn test_contract_process_id_exists() {
    // Contract specifies ProcessId as newtype over u32
    let _id: ProcessId = ProcessId::new(1234);
}

#[test]
fn test_contract_render_surface_id_exists() {
    // Contract specifies RenderSurfaceId as newtype over uuid
    let _id: RenderSurfaceId = RenderSurfaceId::new();
}

#[test]
fn test_contract_download_id_exists() {
    // Contract specifies DownloadId as newtype over uuid
    let _id: DownloadId = DownloadId::new();
}

#[test]
fn test_contract_bookmark_id_exists() {
    // Contract specifies BookmarkId as newtype over uuid
    let _id: BookmarkId = BookmarkId::new();
}

#[test]
fn test_contract_window_config_fields() {
    // Contract specifies WindowConfig struct with specific fields
    let config = WindowConfig {
        title: "Test".to_string(),
        width: 1024,
        height: 768,
        x: Some(0),
        y: Some(0),
        fullscreen: false,
        resizable: true,
        decorations: true,
        always_on_top: false,
        skip_taskbar: false,
    };

    // Verify all fields from contract exist and have correct types
    let _: String = config.title;
    let _: u32 = config.width;
    let _: u32 = config.height;
    let _: Option<i32> = config.x;
    let _: Option<i32> = config.y;
    let _: bool = config.fullscreen;
    let _: bool = config.resizable;
    let _: bool = config.decorations;
    let _: bool = config.always_on_top;
    let _: bool = config.skip_taskbar;
}

#[test]
fn test_contract_keyboard_shortcut_variants() {
    // Contract specifies KeyboardShortcut enum with specific variants
    let _: KeyboardShortcut = KeyboardShortcut::CtrlT;
    let _: KeyboardShortcut = KeyboardShortcut::CtrlW;
    let _: KeyboardShortcut = KeyboardShortcut::CtrlN;
    let _: KeyboardShortcut = KeyboardShortcut::CtrlShiftT;
    let _: KeyboardShortcut = KeyboardShortcut::CtrlL;
    let _: KeyboardShortcut = KeyboardShortcut::F5;
    let _: KeyboardShortcut = KeyboardShortcut::CtrlR;
    let _: KeyboardShortcut = KeyboardShortcut::CtrlShiftR;
}

#[test]
fn test_contract_component_error_variants() {
    // Contract specifies ComponentError enum with specific variants
    let _: ComponentError = ComponentError::InitializationFailed("test".to_string());
    let _: ComponentError = ComponentError::MessageRoutingFailed("test".to_string());
    let _: ComponentError = ComponentError::InvalidState("test".to_string());
    let _: ComponentError = ComponentError::ResourceNotFound("test".to_string());
    let _: ComponentError = ComponentError::PermissionDenied("test".to_string());
}

#[test]
fn test_contract_window_error_variants() {
    // Contract specifies WindowError enum with specific variants
    let window_id = WindowId::new();

    let _: WindowError = WindowError::CreationFailed("test".to_string());
    let _: WindowError = WindowError::NotFound(window_id);
    let _: WindowError = WindowError::InvalidConfig("test".to_string());
    let _: WindowError = WindowError::PlatformError("test".to_string());
}

#[test]
fn test_contract_tab_error_variants() {
    // Contract specifies TabError enum with specific variants
    let tab_id = TabId::new();

    let _: TabError = TabError::CreationFailed("test".to_string());
    let _: TabError = TabError::NotFound(tab_id);
    let _: TabError = TabError::NavigationFailed("test".to_string());
    let _: TabError = TabError::ProcessIsolationFailed("test".to_string());
}

#[test]
fn test_contract_all_types_serializable() {
    // Contract requires all types to be serializable
    let window_id = WindowId::new();
    let _json = serde_json::to_string(&window_id).expect("WindowId should serialize");

    let tab_id = TabId::new();
    let _json = serde_json::to_string(&tab_id).expect("TabId should serialize");

    let process_id = ProcessId::new(123);
    let _json = serde_json::to_string(&process_id).expect("ProcessId should serialize");

    let render_surface_id = RenderSurfaceId::new();
    let _json =
        serde_json::to_string(&render_surface_id).expect("RenderSurfaceId should serialize");

    let download_id = DownloadId::new();
    let _json = serde_json::to_string(&download_id).expect("DownloadId should serialize");

    let bookmark_id = BookmarkId::new();
    let _json = serde_json::to_string(&bookmark_id).expect("BookmarkId should serialize");

    let config = WindowConfig::default();
    let _json = serde_json::to_string(&config).expect("WindowConfig should serialize");

    let shortcut = KeyboardShortcut::CtrlT;
    let _json = serde_json::to_string(&shortcut).expect("KeyboardShortcut should serialize");

    let error = ComponentError::InitializationFailed("test".to_string());
    let _json = serde_json::to_string(&error).expect("ComponentError should serialize");

    let error = WindowError::CreationFailed("test".to_string());
    let _json = serde_json::to_string(&error).expect("WindowError should serialize");

    let error = TabError::CreationFailed("test".to_string());
    let _json = serde_json::to_string(&error).expect("TabError should serialize");
}

#[test]
fn test_contract_error_types_implement_std_error() {
    use std::error::Error;

    // Contract requires all error types to implement std::error::Error
    let component_error = ComponentError::InitializationFailed("test".to_string());
    let _: &dyn Error = &component_error;

    let window_error = WindowError::CreationFailed("test".to_string());
    let _: &dyn Error = &window_error;

    let tab_error = TabError::CreationFailed("test".to_string());
    let _: &dyn Error = &tab_error;
}

#[test]
fn test_contract_types_derive_debug() {
    // Contract requires Debug trait for all types
    use std::fmt::Debug;

    fn assert_debug<T: Debug>() {}

    assert_debug::<WindowId>();
    assert_debug::<TabId>();
    assert_debug::<ProcessId>();
    assert_debug::<RenderSurfaceId>();
    assert_debug::<DownloadId>();
    assert_debug::<BookmarkId>();
    assert_debug::<WindowConfig>();
    assert_debug::<KeyboardShortcut>();
    assert_debug::<ComponentError>();
    assert_debug::<WindowError>();
    assert_debug::<TabError>();
}

#[test]
fn test_contract_types_derive_clone() {
    // Contract requires Clone trait for most types
    use std::clone::Clone;

    fn assert_clone<T: Clone>() {}

    assert_clone::<WindowId>();
    assert_clone::<TabId>();
    assert_clone::<ProcessId>();
    assert_clone::<RenderSurfaceId>();
    assert_clone::<DownloadId>();
    assert_clone::<BookmarkId>();
    assert_clone::<WindowConfig>();
    assert_clone::<KeyboardShortcut>();
    assert_clone::<ComponentError>();
    assert_clone::<WindowError>();
    assert_clone::<TabError>();
}
