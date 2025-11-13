// @validates: REQ-002 (ID type definitions)
use shared_types::tab::TabId;
use shared_types::window::WindowId;

#[test]
fn test_tab_id_new() {
    let id1 = TabId::new();
    let id2 = TabId::new();
    assert_ne!(id1, id2, "Each new TabId should be unique");
}

#[test]
fn test_tab_id_equality() {
    let id = TabId(12345);
    let same_id = TabId(12345);
    assert_eq!(id, same_id);
}

#[test]
fn test_tab_id_serialization() {
    let id = TabId(9876543210);
    let json = serde_json::to_string(&id).expect("Should serialize");
    let deserialized: TabId = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(id, deserialized);
}

#[test]
fn test_window_id_new() {
    let id1 = WindowId::new();
    let id2 = WindowId::new();
    assert_ne!(id1, id2, "Each new WindowId should be unique");
}

#[test]
fn test_window_id_equality() {
    let id = WindowId(54321);
    let same_id = WindowId(54321);
    assert_eq!(id, same_id);
}

#[test]
fn test_window_id_serialization() {
    let id = WindowId(1234567890);
    let json = serde_json::to_string(&id).expect("Should serialize");
    let deserialized: WindowId = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(id, deserialized);
}

#[test]
fn test_tab_id_can_be_hashed() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    let id = TabId(111);
    map.insert(id, "value");
    assert_eq!(map.get(&id), Some(&"value"));
}

#[test]
fn test_window_id_can_be_hashed() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    let id = WindowId(222);
    map.insert(id, "value");
    assert_eq!(map.get(&id), Some(&"value"));
}
