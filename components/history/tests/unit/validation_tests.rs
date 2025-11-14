use history::validation::{validate_url, validate_title, validate_timestamp};

#[test]
fn test_validate_url_valid() {
    assert!(validate_url("https://example.com").is_ok());
    assert!(validate_url("http://example.com").is_ok());
    assert!(validate_url("https://sub.example.com/path?query=1").is_ok());
    assert!(validate_url("https://example.com:8080/path").is_ok());
}

#[test]
fn test_validate_url_invalid() {
    assert!(validate_url("").is_err());
    assert!(validate_url("not a url").is_err());
    assert!(validate_url("ftp://example.com").is_err());
    assert!(validate_url("javascript:alert(1)").is_err());
}

#[test]
fn test_validate_title_valid() {
    assert_eq!(validate_title("Normal Title"), "Normal Title");
    assert_eq!(validate_title("Title with numbers 123"), "Title with numbers 123");
    assert_eq!(validate_title("   Trimmed   "), "Trimmed");
}

#[test]
fn test_validate_title_empty() {
    assert_eq!(validate_title(""), "Untitled");
    assert_eq!(validate_title("   "), "Untitled");
}

#[test]
fn test_validate_title_too_long() {
    let long_title = "a".repeat(300);
    let result = validate_title(&long_title);
    assert_eq!(result.len(), 255);
}

#[test]
fn test_validate_timestamp_valid() {
    let now = chrono::Utc::now().timestamp();
    assert!(validate_timestamp(now).is_ok());

    // Past timestamp
    assert!(validate_timestamp(1609459200).is_ok()); // 2021-01-01

    // Recent past
    assert!(validate_timestamp(now - 86400).is_ok());
}

#[test]
fn test_validate_timestamp_invalid() {
    // Future timestamp (more than 1 day ahead)
    let future = chrono::Utc::now().timestamp() + 100000;
    assert!(validate_timestamp(future).is_err());

    // Very old timestamp (before 2000)
    assert!(validate_timestamp(946684800 - 1).is_err());

    // Negative timestamp
    assert!(validate_timestamp(-1).is_err());
}
