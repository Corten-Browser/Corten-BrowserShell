//! Unit tests for SessionManager

use session_manager::{ClosedTab, SessionManager, SessionState, TabState, WindowState};
use tempfile::TempDir;

fn get_temp_db() -> (TempDir, String) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test_session.db").to_str().unwrap().to_string();
    (dir, db_path)
}

#[tokio::test]
async fn test_session_manager_creation() {
    // GIVEN: Database path
    let (_dir, db_path) = get_temp_db();

    // WHEN: Creating new SessionManager
    let result = SessionManager::new(&db_path).await;

    // THEN: Manager created successfully
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_session_manager_creates_schema() {
    // GIVEN: New database
    let (_dir, db_path) = get_temp_db();

    // WHEN: Creating SessionManager
    let _manager = SessionManager::new(&db_path).await.unwrap();

    // THEN: Database file exists
    assert!(std::path::Path::new(&db_path).exists());
}

#[tokio::test]
async fn test_save_empty_session() {
    // GIVEN: SessionManager and empty session state
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();
    let state = SessionState::new(1234567890);

    // WHEN: Saving session
    let result = manager.save_session(&state).await;

    // THEN: Session saved successfully with ID
    assert!(result.is_ok());
    let session_id = result.unwrap();
    assert!(session_id > 0);
}

#[tokio::test]
async fn test_save_session_with_windows() {
    // GIVEN: Session state with 2 windows
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut state = SessionState::new(1234567890);
    state.windows.push(WindowState::new("win-1".to_string(), 800, 600));
    state.windows.push(WindowState::new("win-2".to_string(), 1024, 768));

    // WHEN: Saving session
    let session_id = manager.save_session(&state).await.unwrap();

    // THEN: Session and windows saved correctly
    let restored = manager.get_session(session_id).await.unwrap().unwrap();
    assert_eq!(restored.windows.len(), 2);
    assert_eq!(restored.windows[0].id, "win-1");
    assert_eq!(restored.windows[1].id, "win-2");
}

#[tokio::test]
async fn test_save_session_with_tabs() {
    // GIVEN: Session with window containing 3 tabs
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut state = SessionState::new(1234567890);
    let mut window = WindowState::new("win-1".to_string(), 800, 600);
    window.tabs.push(TabState::new(
        "tab-1".to_string(),
        "https://example.com".to_string(),
        "Example".to_string(),
        0,
    ));
    window.tabs.push(TabState::new(
        "tab-2".to_string(),
        "https://test.com".to_string(),
        "Test".to_string(),
        1,
    ));
    window.tabs.push(TabState::new(
        "tab-3".to_string(),
        "https://demo.com".to_string(),
        "Demo".to_string(),
        2,
    ));
    state.windows.push(window);

    // WHEN: Saving session
    let session_id = manager.save_session(&state).await.unwrap();

    // THEN: All tabs saved with correct positions
    let restored = manager.get_session(session_id).await.unwrap().unwrap();
    assert_eq!(restored.windows[0].tabs.len(), 3);
    assert_eq!(restored.windows[0].tabs[0].url, "https://example.com");
    assert_eq!(restored.windows[0].tabs[1].url, "https://test.com");
    assert_eq!(restored.windows[0].tabs[2].url, "https://demo.com");
}

#[tokio::test]
async fn test_restore_most_recent_session() {
    // GIVEN: Multiple saved sessions
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let state1 = SessionState::new(1000);
    let state2 = SessionState::new(2000);
    let state3 = SessionState::new(3000);

    manager.save_session(&state1).await.unwrap();
    manager.save_session(&state2).await.unwrap();
    manager.save_session(&state3).await.unwrap();

    // WHEN: Restoring session
    let restored = manager.restore_session().await.unwrap();

    // THEN: Most recent session returned
    assert!(restored.is_some());
    assert_eq!(restored.unwrap().timestamp, 3000);
}

#[tokio::test]
async fn test_restore_session_by_id() {
    // GIVEN: Multiple sessions with specific IDs
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let state1 = SessionState::new(1000);
    let state2 = SessionState::new(2000);

    let id1 = manager.save_session(&state1).await.unwrap();
    let id2 = manager.save_session(&state2).await.unwrap();

    // WHEN: Getting session by ID
    let restored1 = manager.get_session(id1).await.unwrap().unwrap();
    let restored2 = manager.get_session(id2).await.unwrap().unwrap();

    // THEN: Correct sessions returned
    assert_eq!(restored1.timestamp, 1000);
    assert_eq!(restored2.timestamp, 2000);
}

#[tokio::test]
async fn test_restore_nonexistent_session() {
    // GIVEN: Empty database
    let (_dir, db_path) = get_temp_db();
    let manager = SessionManager::new(&db_path).await.unwrap();

    // WHEN: Restoring session
    let restored = manager.restore_session().await.unwrap();

    // THEN: None returned
    assert!(restored.is_none());
}

#[tokio::test]
async fn test_save_window_positions() {
    // GIVEN: Window with specific x, y, width, height
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut state = SessionState::new(1234567890);
    let mut window = WindowState::new("win-1".to_string(), 1920, 1080);
    window.x = Some(100);
    window.y = Some(200);
    state.windows.push(window);

    // WHEN: Saving and restoring
    let session_id = manager.save_session(&state).await.unwrap();
    let restored = manager.get_session(session_id).await.unwrap().unwrap();

    // THEN: Positions restored correctly
    assert_eq!(restored.windows[0].x, Some(100));
    assert_eq!(restored.windows[0].y, Some(200));
    assert_eq!(restored.windows[0].width, 1920);
    assert_eq!(restored.windows[0].height, 1080);
}

#[tokio::test]
async fn test_save_maximized_state() {
    // GIVEN: Maximized window
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut state = SessionState::new(1234567890);
    let mut window = WindowState::new("win-1".to_string(), 1920, 1080);
    window.maximized = true;
    state.windows.push(window);

    // WHEN: Saving and restoring
    let session_id = manager.save_session(&state).await.unwrap();
    let restored = manager.get_session(session_id).await.unwrap().unwrap();

    // THEN: Maximized state preserved
    assert!(restored.windows[0].maximized);
}

#[tokio::test]
async fn test_save_active_tab_index() {
    // GIVEN: Window with active tab at index 2
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut state = SessionState::new(1234567890);
    let mut window = WindowState::new("win-1".to_string(), 800, 600);
    window.tabs.push(TabState::new(
        "tab-1".to_string(),
        "https://a.com".to_string(),
        "A".to_string(),
        0,
    ));
    window.tabs.push(TabState::new(
        "tab-2".to_string(),
        "https://b.com".to_string(),
        "B".to_string(),
        1,
    ));
    window.tabs.push(TabState::new(
        "tab-3".to_string(),
        "https://c.com".to_string(),
        "C".to_string(),
        2,
    ));
    window.active_tab_index = Some(2);
    state.windows.push(window);

    // WHEN: Saving and restoring
    let session_id = manager.save_session(&state).await.unwrap();
    let restored = manager.get_session(session_id).await.unwrap().unwrap();

    // THEN: Active tab index restored
    assert_eq!(restored.windows[0].active_tab_index, Some(2));
}

#[tokio::test]
async fn test_list_sessions() {
    // GIVEN: 5 saved sessions
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    for i in 1..=5 {
        let state = SessionState::new(i * 1000);
        manager.save_session(&state).await.unwrap();
    }

    // WHEN: Listing sessions
    let sessions = manager.list_sessions().await.unwrap();

    // THEN: All sessions returned with timestamps
    assert_eq!(sessions.len(), 5);
    // Should be in descending timestamp order
    assert_eq!(sessions[0].1, 5000);
    assert_eq!(sessions[4].1, 1000);
}

#[tokio::test]
async fn test_cleanup_old_sessions() {
    // GIVEN: 10 saved sessions
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    for i in 1..=10 {
        let state = SessionState::new(i * 1000);
        manager.save_session(&state).await.unwrap();
    }

    // WHEN: Cleanup keeping 3
    let deleted = manager.cleanup_old_sessions(3).await.unwrap();

    // THEN: 7 sessions deleted, 3 remain
    assert_eq!(deleted, 7);
    let remaining = manager.list_sessions().await.unwrap();
    assert_eq!(remaining.len(), 3);
}

#[tokio::test]
async fn test_cleanup_preserves_recent() {
    // GIVEN: Sessions with different timestamps
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    for i in 1..=10 {
        let state = SessionState::new(i * 1000);
        manager.save_session(&state).await.unwrap();
    }

    // WHEN: Cleanup keeping 5
    manager.cleanup_old_sessions(5).await.unwrap();

    // THEN: Most recent 5 preserved
    let remaining = manager.list_sessions().await.unwrap();
    assert_eq!(remaining.len(), 5);
    assert_eq!(remaining[0].1, 10000); // Most recent
    assert_eq!(remaining[4].1, 6000);  // 5th most recent
}

#[tokio::test]
async fn test_multiple_saves_create_separate_sessions() {
    // GIVEN: SessionManager
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    // WHEN: Saving 3 different states
    let id1 = manager.save_session(&SessionState::new(1000)).await.unwrap();
    let id2 = manager.save_session(&SessionState::new(2000)).await.unwrap();
    let id3 = manager.save_session(&SessionState::new(3000)).await.unwrap();

    // THEN: 3 separate session records created
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);

    let sessions = manager.list_sessions().await.unwrap();
    assert_eq!(sessions.len(), 3);
}

#[tokio::test]
async fn test_record_closed_tab() {
    // GIVEN: SessionManager and closed tab
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let tab = ClosedTab::new(
        "tab-1".to_string(),
        "https://example.com".to_string(),
        "Example".to_string(),
        1234567890,
    );

    // WHEN: Recording closed tab
    let result = manager.record_closed_tab(tab).await;

    // THEN: Tab saved successfully
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_recently_closed_tabs() {
    // GIVEN: 5 closed tabs
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    for i in 1..=5 {
        let tab = ClosedTab::new(
            format!("tab-{}", i),
            format!("https://example{}.com", i),
            format!("Example {}", i),
            i * 1000,
        );
        manager.record_closed_tab(tab).await.unwrap();
    }

    // WHEN: Getting recently closed (limit 10)
    let tabs = manager.get_recently_closed(10).await.unwrap();

    // THEN: All 5 tabs returned in reverse chronological order
    assert_eq!(tabs.len(), 5);
    assert_eq!(tabs[0].closed_at, 5000); // Most recent
    assert_eq!(tabs[4].closed_at, 1000); // Oldest
}

#[tokio::test]
async fn test_recently_closed_limit() {
    // GIVEN: 20 closed tabs
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    for i in 1..=20 {
        let tab = ClosedTab::new(
            format!("tab-{}", i),
            format!("https://example{}.com", i),
            format!("Example {}", i),
            i * 1000,
        );
        manager.record_closed_tab(tab).await.unwrap();
    }

    // WHEN: Getting recently closed (limit 10)
    let tabs = manager.get_recently_closed(10).await.unwrap();

    // THEN: Only 10 most recent tabs returned
    assert_eq!(tabs.len(), 10);
    assert_eq!(tabs[0].closed_at, 20000); // Most recent
    assert_eq!(tabs[9].closed_at, 11000); // 10th most recent
}

#[tokio::test]
async fn test_clear_recently_closed() {
    // GIVEN: 10 closed tabs
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    for i in 1..=10 {
        let tab = ClosedTab::new(
            format!("tab-{}", i),
            format!("https://example{}.com", i),
            format!("Example {}", i),
            i * 1000,
        );
        manager.record_closed_tab(tab).await.unwrap();
    }

    // WHEN: Clearing recently closed
    let cleared = manager.clear_recently_closed().await.unwrap();

    // THEN: All closed tabs removed
    assert_eq!(cleared, 10);
    let tabs = manager.get_recently_closed(100).await.unwrap();
    assert_eq!(tabs.len(), 0);
}

#[tokio::test]
async fn test_closed_tab_with_window_id() {
    // GIVEN: Closed tab with window ID
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut tab = ClosedTab::new(
        "tab-1".to_string(),
        "https://example.com".to_string(),
        "Example".to_string(),
        1234567890,
    );
    tab.window_id = Some("win-1".to_string());

    // WHEN: Recording and retrieving
    manager.record_closed_tab(tab).await.unwrap();
    let tabs = manager.get_recently_closed(10).await.unwrap();

    // THEN: Window ID preserved
    assert_eq!(tabs[0].window_id, Some("win-1".to_string()));
}

#[tokio::test]
async fn test_closed_tab_with_position() {
    // GIVEN: Closed tab at position 5
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut tab = ClosedTab::new(
        "tab-1".to_string(),
        "https://example.com".to_string(),
        "Example".to_string(),
        1234567890,
    );
    tab.position = Some(5);

    // WHEN: Recording and retrieving
    manager.record_closed_tab(tab).await.unwrap();
    let tabs = manager.get_recently_closed(10).await.unwrap();

    // THEN: Position preserved
    assert_eq!(tabs[0].position, Some(5));
}

#[tokio::test]
async fn test_export_session_to_json() {
    // GIVEN: Saved session with windows and tabs
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut state = SessionState::new(1234567890);
    let mut window = WindowState::new("win-1".to_string(), 800, 600);
    window.tabs.push(TabState::new(
        "tab-1".to_string(),
        "https://example.com".to_string(),
        "Example".to_string(),
        0,
    ));
    state.windows.push(window);

    let session_id = manager.save_session(&state).await.unwrap();

    // WHEN: Exporting to JSON
    let json = manager.export_session(Some(session_id)).await.unwrap();

    // THEN: Valid JSON string returned
    assert!(!json.is_empty());
    assert!(json.contains("\"windows\""));
    assert!(json.contains("\"timestamp\""));
}

#[tokio::test]
async fn test_import_session_from_json() {
    // GIVEN: Valid session JSON
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let json = r#"{
        "windows": [
            {
                "id": "win-1",
                "x": 100,
                "y": 200,
                "width": 800,
                "height": 600,
                "maximized": false,
                "tabs": [],
                "active_tab_index": null
            }
        ],
        "timestamp": 1234567890
    }"#;

    // WHEN: Importing session
    let session_id = manager.import_session(json).await.unwrap();

    // THEN: Session restored with correct data
    let restored = manager.get_session(session_id).await.unwrap().unwrap();
    assert_eq!(restored.windows.len(), 1);
    assert_eq!(restored.windows[0].id, "win-1");
    assert_eq!(restored.timestamp, 1234567890);
}

#[tokio::test]
async fn test_export_import_roundtrip() {
    // GIVEN: Session state
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut state = SessionState::new(1234567890);
    let mut window = WindowState::new("win-1".to_string(), 800, 600);
    window.x = Some(100);
    window.y = Some(200);
    window.tabs.push(TabState::new(
        "tab-1".to_string(),
        "https://example.com".to_string(),
        "Example".to_string(),
        0,
    ));
    state.windows.push(window);

    let original_id = manager.save_session(&state).await.unwrap();

    // WHEN: Exporting then importing
    let json = manager.export_session(Some(original_id)).await.unwrap();
    let new_id = manager.import_session(&json).await.unwrap();

    // THEN: Restored state matches original
    let restored = manager.get_session(new_id).await.unwrap().unwrap();
    assert_eq!(restored.windows.len(), 1);
    assert_eq!(restored.windows[0].id, "win-1");
    assert_eq!(restored.windows[0].tabs.len(), 1);
    assert_eq!(restored.timestamp, 1234567890);
}

#[tokio::test]
async fn test_import_invalid_json() {
    // GIVEN: Invalid JSON string
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    // WHEN: Importing
    let result = manager.import_session("invalid json{{{").await;

    // THEN: Error returned
    assert!(result.is_err());
}

#[tokio::test]
async fn test_save_session_with_no_windows() {
    // GIVEN: Session state with empty windows vector
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let state = SessionState::new(1234567890);

    // WHEN: Saving
    let session_id = manager.save_session(&state).await.unwrap();

    // THEN: Session saved successfully
    let restored = manager.get_session(session_id).await.unwrap().unwrap();
    assert_eq!(restored.windows.len(), 0);
}

#[tokio::test]
async fn test_save_window_with_no_tabs() {
    // GIVEN: Window with empty tabs vector
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut state = SessionState::new(1234567890);
    state.windows.push(WindowState::new("win-1".to_string(), 800, 600));

    // WHEN: Saving
    let session_id = manager.save_session(&state).await.unwrap();

    // THEN: Window saved successfully
    let restored = manager.get_session(session_id).await.unwrap().unwrap();
    assert_eq!(restored.windows.len(), 1);
    assert_eq!(restored.windows[0].tabs.len(), 0);
}

#[tokio::test]
async fn test_clear_removes_all_data() {
    // GIVEN: Database with sessions and closed tabs
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    // Add sessions
    for i in 1..=3 {
        manager.save_session(&SessionState::new(i * 1000)).await.unwrap();
    }

    // Add closed tabs
    for i in 1..=5 {
        manager
            .record_closed_tab(ClosedTab::new(
                format!("tab-{}", i),
                format!("https://example{}.com", i),
                format!("Example {}", i),
                i * 1000,
            ))
            .await
            .unwrap();
    }

    // WHEN: Clearing
    let cleared = manager.clear().await.unwrap();

    // THEN: All tables emptied
    assert!(cleared > 0);
    assert_eq!(manager.list_sessions().await.unwrap().len(), 0);
    assert_eq!(manager.get_recently_closed(100).await.unwrap().len(), 0);
}

#[tokio::test]
async fn test_url_validation_in_tabs() {
    // GIVEN: Tab with empty URL
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut state = SessionState::new(1234567890);
    let mut window = WindowState::new("win-1".to_string(), 800, 600);
    window.tabs.push(TabState::new(
        "tab-1".to_string(),
        "".to_string(), // Empty URL
        "New Tab".to_string(),
        0,
    ));
    state.windows.push(window);

    // WHEN: Saving
    let session_id = manager.save_session(&state).await.unwrap();

    // THEN: Still saved (URLs can be empty for new tabs)
    let restored = manager.get_session(session_id).await.unwrap().unwrap();
    assert_eq!(restored.windows[0].tabs[0].url, "");
}

#[tokio::test]
async fn test_large_session_state() {
    // GIVEN: Session with 100 windows, 10 tabs each
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut state = SessionState::new(1234567890);
    for w in 0..100 {
        let mut window = WindowState::new(format!("win-{}", w), 800, 600);
        for t in 0..10 {
            window.tabs.push(TabState::new(
                format!("tab-{}-{}", w, t),
                format!("https://example{}.com/{}", w, t),
                format!("Title {} {}", w, t),
                t,
            ));
        }
        state.windows.push(window);
    }

    // WHEN: Saving and restoring
    let session_id = manager.save_session(&state).await.unwrap();
    let restored = manager.get_session(session_id).await.unwrap().unwrap();

    // THEN: All data preserved correctly
    assert_eq!(restored.windows.len(), 100);
    for w in 0..100 {
        assert_eq!(restored.windows[w].tabs.len(), 10);
    }
}

#[tokio::test]
async fn test_timestamp_accuracy() {
    // GIVEN: Two sessions saved 1 second apart
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let state1 = SessionState::new(1000);
    let state2 = SessionState::new(2000);

    manager.save_session(&state1).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    manager.save_session(&state2).await.unwrap();

    // WHEN: Checking timestamps
    let sessions = manager.list_sessions().await.unwrap();

    // THEN: Timestamps correctly ordered
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].1, 2000);
    assert_eq!(sessions[1].1, 1000);
}
