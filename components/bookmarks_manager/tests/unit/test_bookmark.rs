//! Unit tests for Bookmark struct

use bookmarks_manager::Bookmark;
use shared_types::BookmarkId;

#[test]
fn test_bookmark_creation_with_all_fields() {
    /// Given all bookmark fields are provided
    /// When creating a bookmark
    /// Then all fields should be set correctly
    let id = BookmarkId::new();
    let bookmark = Bookmark {
        id: Some(id),
        url: "https://example.com".to_string(),
        title: "Example Site".to_string(),
        folder: Some("Work".to_string()),
        tags: vec!["important".to_string(), "reference".to_string()],
        created_at: 1234567890,
    };

    assert_eq!(bookmark.id, Some(id));
    assert_eq!(bookmark.url, "https://example.com");
    assert_eq!(bookmark.title, "Example Site");
    assert_eq!(bookmark.folder, Some("Work".to_string()));
    assert_eq!(bookmark.tags.len(), 2);
    assert_eq!(bookmark.created_at, 1234567890);
}

#[test]
fn test_bookmark_without_optional_fields() {
    /// Given optional fields are not provided
    /// When creating a bookmark
    /// Then it should be valid with None values
    let bookmark = Bookmark {
        id: None,
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        folder: None,
        tags: vec![],
        created_at: 1234567890,
    };

    assert!(bookmark.id.is_none());
    assert!(bookmark.folder.is_none());
    assert!(bookmark.tags.is_empty());
}

#[test]
fn test_bookmark_serialization_to_yaml() {
    /// Given a bookmark instance
    /// When serializing to YAML
    /// Then it should produce valid YAML
    let id = BookmarkId::new();
    let bookmark = Bookmark {
        id: Some(id),
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        folder: Some("Personal".to_string()),
        tags: vec!["test".to_string()],
        created_at: 1234567890,
    };

    let yaml = serde_yaml::to_string(&bookmark).expect("Should serialize to YAML");

    assert!(yaml.contains("url: https://example.com"));
    assert!(yaml.contains("title: Example"));
    assert!(yaml.contains("folder: Personal"));
    assert!(yaml.contains("test"));
}

#[test]
fn test_bookmark_deserialization_from_yaml() {
    /// Given a YAML string representing a bookmark
    /// When deserializing from YAML
    /// Then it should create a valid Bookmark instance
    let yaml = r#"
url: https://example.com
title: Example Site
folder: Work
tags:
  - important
  - reference
created_at: 1234567890
"#;

    let bookmark: Bookmark = serde_yaml::from_str(yaml).expect("Should deserialize from YAML");

    assert_eq!(bookmark.url, "https://example.com");
    assert_eq!(bookmark.title, "Example Site");
    assert_eq!(bookmark.folder, Some("Work".to_string()));
    assert_eq!(bookmark.tags.len(), 2);
    assert_eq!(bookmark.created_at, 1234567890);
}

#[test]
fn test_bookmark_roundtrip_serialization() {
    /// Given a bookmark instance
    /// When serializing and then deserializing
    /// Then the result should match the original
    let original = Bookmark {
        id: None, // IDs are not serialized in YAML
        url: "https://rust-lang.org".to_string(),
        title: "Rust Programming Language".to_string(),
        folder: Some("Programming".to_string()),
        tags: vec!["rust".to_string(), "programming".to_string()],
        created_at: 9876543210,
    };

    let yaml = serde_yaml::to_string(&original).expect("Should serialize");
    let deserialized: Bookmark = serde_yaml::from_str(&yaml).expect("Should deserialize");

    assert_eq!(deserialized.url, original.url);
    assert_eq!(deserialized.title, original.title);
    assert_eq!(deserialized.folder, original.folder);
    assert_eq!(deserialized.tags, original.tags);
    assert_eq!(deserialized.created_at, original.created_at);
}
