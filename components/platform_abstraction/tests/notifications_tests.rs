// @validates: REQ-005
//! Integration tests for system notifications

use platform_abstraction::{Notifier, Notification};

#[tokio::test]
#[should_panic(expected = "not yet implemented")]
async fn test_notifier_send_fails_unimplemented() {
    // RED: Notification send not implemented

    // Given a notifier and notification
    let notifier = Notifier::new();
    let notification = Notification::new("Title", "Body");

    // When we try to send notification
    // Then it should panic (unimplemented)
    let _ = notifier.send(&notification).await;
}

#[test]
fn test_notification_can_be_created() {
    // This should pass - just testing struct creation

    // Given title and body
    // When we create a notification
    let notification = Notification::new("Test Title", "Test Body");

    // Then notification should have correct fields
    assert_eq!(notification.title, "Test Title");
    assert_eq!(notification.body, "Test Body");
    assert_eq!(notification.icon, None);
}

#[test]
fn test_notification_with_icon() {
    // This should pass - testing builder pattern

    // Given title, body, and icon
    // When we create notification with icon
    let notification = Notification::new("Title", "Body")
        .with_icon("/path/to/icon.png");

    // Then notification should have icon
    assert_eq!(notification.title, "Title");
    assert_eq!(notification.body, "Body");
    assert_eq!(notification.icon, Some("/path/to/icon.png".to_string()));
}
