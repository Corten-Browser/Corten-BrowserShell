use downloads::validation::*;

#[test]
fn test_validate_url_valid_http() {
    let result = validate_url("http://example.com/file.zip");
    assert!(result.is_ok());
}

#[test]
fn test_validate_url_valid_https() {
    let result = validate_url("https://example.com/file.pdf");
    assert!(result.is_ok());
}

#[test]
fn test_validate_url_invalid_scheme() {
    let result = validate_url("ftp://example.com/file.zip");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("scheme"));
}

#[test]
fn test_validate_url_invalid_format() {
    let result = validate_url("not a url");
    assert!(result.is_err());
}

#[test]
fn test_validate_url_empty() {
    let result = validate_url("");
    assert!(result.is_err());
}

#[test]
fn test_sanitize_file_name_normal() {
    let result = sanitize_file_name("document.pdf");
    assert_eq!(result, "document.pdf");
}

#[test]
fn test_sanitize_file_name_path_traversal() {
    let result = sanitize_file_name("../../../etc/passwd");
    assert_eq!(result, "etc_passwd");
}

#[test]
fn test_sanitize_file_name_absolute_path() {
    let result = sanitize_file_name("/etc/passwd");
    assert_eq!(result, "etc_passwd");
}

#[test]
fn test_sanitize_file_name_special_chars() {
    let result = sanitize_file_name("file:name*with?invalid<chars>.txt");
    assert_eq!(result, "file_name_with_invalid_chars_.txt");
}

#[test]
fn test_sanitize_file_name_empty() {
    let result = sanitize_file_name("");
    assert_eq!(result, "download");
}

#[test]
fn test_validate_save_path_valid() {
    let result = validate_save_path("/tmp/test.txt");
    assert!(result.is_ok());
}

#[test]
fn test_validate_save_path_relative() {
    let result = validate_save_path("test.txt");
    assert!(result.is_ok());
}

#[test]
fn test_validate_save_path_empty() {
    let result = validate_save_path("");
    assert!(result.is_err());
}

#[test]
fn test_extract_file_name_from_url() {
    let result = extract_file_name_from_url("https://example.com/downloads/file.zip");
    assert_eq!(result, "file.zip");
}

#[test]
fn test_extract_file_name_from_url_no_extension() {
    let result = extract_file_name_from_url("https://example.com/downloads/document");
    assert_eq!(result, "document");
}

#[test]
fn test_extract_file_name_from_url_query_params() {
    let result = extract_file_name_from_url("https://example.com/file.pdf?token=abc123");
    assert_eq!(result, "file.pdf");
}

#[test]
fn test_extract_file_name_from_url_trailing_slash() {
    let result = extract_file_name_from_url("https://example.com/downloads/");
    assert_eq!(result, "download");
}
