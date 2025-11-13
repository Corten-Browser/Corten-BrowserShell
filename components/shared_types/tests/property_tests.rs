// @validates: REQ-002, REQ-003
//! Property-based tests for shared_types
//!
//! Uses proptest to verify properties hold for all possible inputs

use proptest::prelude::*;
use shared_types::tab::{TabId, Url};
use shared_types::window::WindowId;

// Property: TabId serialization round-trip
proptest! {
    #[test]
    fn tab_id_serialization_roundtrip(value in any::<u128>()) {
        let id = TabId(value);
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: TabId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, deserialized);
    }
}

// Property: WindowId serialization round-trip
proptest! {
    #[test]
    fn window_id_serialization_roundtrip(value in any::<u128>()) {
        let id = WindowId(value);
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: WindowId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, deserialized);
    }
}

// Property: TabId equality is reflexive
proptest! {
    #[test]
    fn tab_id_equality_reflexive(value in any::<u128>()) {
        let id = TabId(value);
        prop_assert_eq!(id, id);
    }
}

// Property: WindowId equality is reflexive
proptest! {
    #[test]
    fn window_id_equality_reflexive(value in any::<u128>()) {
        let id = WindowId(value);
        prop_assert_eq!(id, id);
    }
}

// Property: TabId equality is symmetric
proptest! {
    #[test]
    fn tab_id_equality_symmetric(value in any::<u128>()) {
        let id1 = TabId(value);
        let id2 = TabId(value);
        prop_assert_eq!(id1 == id2, id2 == id1);
    }
}

// Property: WindowId equality is symmetric
proptest! {
    #[test]
    fn window_id_equality_symmetric(value in any::<u128>()) {
        let id1 = WindowId(value);
        let id2 = WindowId(value);
        prop_assert_eq!(id1 == id2, id2 == id1);
    }
}

// Property: URL parsing rejects empty strings
proptest! {
    #[test]
    fn url_rejects_empty_string(_value in any::<u8>()) {
        let result = Url::parse("");
        prop_assert!(result.is_err());
    }
}

// Property: URL parsing accepts non-empty strings
proptest! {
    #[test]
    fn url_accepts_non_empty_string(s in "[a-zA-Z0-9:/.-]+") {
        let result = Url::parse(&s);
        prop_assert!(result.is_ok());
        if let Ok(url) = result {
            prop_assert_eq!(url.as_str(), s);
        }
    }
}

// Property: URL as_str returns the original string
proptest! {
    #[test]
    fn url_as_str_returns_original(s in "[a-zA-Z0-9:/.-]+") {
        let url = Url(s.clone());
        prop_assert_eq!(url.as_str(), s);
    }
}

// Property: TabId hashing is consistent
proptest! {
    #[test]
    fn tab_id_hash_consistent(value in any::<u128>()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let id = TabId(value);
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        id.hash(&mut hasher1);
        id.hash(&mut hasher2);

        prop_assert_eq!(hasher1.finish(), hasher2.finish());
    }
}

// Property: WindowId hashing is consistent
proptest! {
    #[test]
    fn window_id_hash_consistent(value in any::<u128>()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let id = WindowId(value);
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        id.hash(&mut hasher1);
        id.hash(&mut hasher2);

        prop_assert_eq!(hasher1.finish(), hasher2.finish());
    }
}
