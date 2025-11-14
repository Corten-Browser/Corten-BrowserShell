use shared_types::*;

#[test]
fn test_window_id_creation() {
    let id1 = WindowId::new();
    let id2 = WindowId::new();

    // Each ID should be unique
    assert_ne!(id1, id2);
}

#[test]
fn test_window_id_clone() {
    let id1 = WindowId::new();
    let id2 = id1.clone();

    // Cloned IDs should be equal
    assert_eq!(id1, id2);
}

#[test]
fn test_window_id_serialization() {
    let id = WindowId::new();

    // Serialize to JSON
    let json = serde_json::to_string(&id).expect("Failed to serialize");

    // Deserialize back
    let deserialized: WindowId = serde_json::from_str(&json).expect("Failed to deserialize");

    // Should be equal
    assert_eq!(id, deserialized);
}

#[test]
fn test_tab_id_creation() {
    let id1 = TabId::new();
    let id2 = TabId::new();

    // Each ID should be unique
    assert_ne!(id1, id2);
}

#[test]
fn test_tab_id_serialization() {
    let id = TabId::new();

    let json = serde_json::to_string(&id).expect("Failed to serialize");
    let deserialized: TabId = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(id, deserialized);
}

#[test]
fn test_process_id_creation() {
    let id1 = ProcessId::new(1234);
    let id2 = ProcessId::new(5678);

    // Different process IDs
    assert_ne!(id1, id2);
}

#[test]
fn test_process_id_same_value() {
    let id1 = ProcessId::new(1234);
    let id2 = ProcessId::new(1234);

    // Same process IDs should be equal
    assert_eq!(id1, id2);
}

#[test]
fn test_process_id_serialization() {
    let id = ProcessId::new(9999);

    let json = serde_json::to_string(&id).expect("Failed to serialize");
    let deserialized: ProcessId = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(id, deserialized);
}

#[test]
fn test_render_surface_id_creation() {
    let id1 = RenderSurfaceId::new();
    let id2 = RenderSurfaceId::new();

    assert_ne!(id1, id2);
}

#[test]
fn test_render_surface_id_serialization() {
    let id = RenderSurfaceId::new();

    let json = serde_json::to_string(&id).expect("Failed to serialize");
    let deserialized: RenderSurfaceId = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(id, deserialized);
}

#[test]
fn test_download_id_creation() {
    let id1 = DownloadId::new();
    let id2 = DownloadId::new();

    assert_ne!(id1, id2);
}

#[test]
fn test_download_id_serialization() {
    let id = DownloadId::new();

    let json = serde_json::to_string(&id).expect("Failed to serialize");
    let deserialized: DownloadId = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(id, deserialized);
}

#[test]
fn test_bookmark_id_creation() {
    let id1 = BookmarkId::new();
    let id2 = BookmarkId::new();

    assert_ne!(id1, id2);
}

#[test]
fn test_bookmark_id_serialization() {
    let id = BookmarkId::new();

    let json = serde_json::to_string(&id).expect("Failed to serialize");
    let deserialized: BookmarkId = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(id, deserialized);
}

#[test]
fn test_process_id_get_value() {
    let id = ProcessId::new(42);

    // Should be able to extract the inner value
    assert_eq!(id.as_u32(), 42);
}
