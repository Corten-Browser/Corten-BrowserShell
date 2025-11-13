// @validates: REQ-004
//! Integration tests for clipboard operations

use platform_abstraction::Clipboard;

#[tokio::test]
#[should_panic(expected = "not yet implemented")]
async fn test_clipboard_read_text_fails_unimplemented() {
    // RED: Clipboard read not implemented

    // Given a clipboard
    let clipboard = Clipboard::new();

    // When we try to read from clipboard
    // Then it should panic (unimplemented)
    let _ = clipboard.read_text().await;
}

#[tokio::test]
#[should_panic(expected = "not yet implemented")]
async fn test_clipboard_write_text_fails_unimplemented() {
    // RED: Clipboard write not implemented

    // Given a clipboard
    let mut clipboard = Clipboard::new();

    // When we try to write to clipboard
    // Then it should panic (unimplemented)
    let _ = clipboard.write_text("test").await;
}
