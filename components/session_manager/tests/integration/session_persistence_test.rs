//! Integration tests for session persistence

use session_manager::{ClosedTab, SessionManager, SessionState, TabState, WindowState};
use tempfile::TempDir;

fn get_temp_db() -> (TempDir, String) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test_session.db").to_str().unwrap().to_string();
    (dir, db_path)
}

#[tokio::test]
async fn test_session_persistence_across_restarts() {
    // GIVEN: SessionManager saves state and is dropped
    let (_dir, db_path) = get_temp_db();

    let session_id = {
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

        manager.save_session(&state).await.unwrap()
    }; // Manager dropped here

    // WHEN: Creating new SessionManager with same database
    let manager = SessionManager::new(&db_path).await.unwrap();

    // THEN: Previous session can be restored
    let restored = manager.get_session(session_id).await.unwrap().unwrap();
    assert_eq!(restored.windows.len(), 1);
    assert_eq!(restored.windows[0].tabs.len(), 1);
    assert_eq!(restored.windows[0].tabs[0].url, "https://example.com");
}

#[tokio::test]
async fn test_crash_recovery_scenario() {
    // GIVEN: Session saved during normal operation
    let (_dir, db_path) = get_temp_db();

    {
        let mut manager = SessionManager::new(&db_path).await.unwrap();

        let mut state = SessionState::new(1234567890);
        let mut window = WindowState::new("win-1".to_string(), 1024, 768);
        window.tabs.push(TabState::new(
            "tab-1".to_string(),
            "https://work.com".to_string(),
            "Work".to_string(),
            0,
        ));
        window.tabs.push(TabState::new(
            "tab-2".to_string(),
            "https://docs.com".to_string(),
            "Docs".to_string(),
            1,
        ));
        state.windows.push(window);

        manager.save_session(&state).await.unwrap();
    } // Simulate crash - manager dropped without cleanup

    // WHEN: Simulating crash and recovery
    let manager = SessionManager::new(&db_path).await.unwrap();
    let restored = manager.restore_session().await.unwrap();

    // THEN: Session restored to last saved state
    assert!(restored.is_some());
    let state = restored.unwrap();
    assert_eq!(state.windows.len(), 1);
    assert_eq!(state.windows[0].tabs.len(), 2);
}

#[tokio::test]
async fn test_closed_tabs_persist_across_restarts() {
    // GIVEN: Recently closed tabs recorded
    let (_dir, db_path) = get_temp_db();

    {
        let mut manager = SessionManager::new(&db_path).await.unwrap();

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
    } // Manager dropped

    // WHEN: Restarting SessionManager
    let manager = SessionManager::new(&db_path).await.unwrap();

    // THEN: Closed tabs still available
    let tabs = manager.get_recently_closed(10).await.unwrap();
    assert_eq!(tabs.len(), 5);
}

#[tokio::test]
async fn test_session_cleanup_doesnt_affect_closed_tabs() {
    // GIVEN: Old sessions and recent closed tabs
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    // Save 10 sessions
    for i in 1..=10 {
        manager.save_session(&SessionState::new(i * 1000)).await.unwrap();
    }

    // Record 5 closed tabs
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

    // WHEN: Cleaning up old sessions
    manager.cleanup_old_sessions(3).await.unwrap();

    // THEN: Closed tabs remain intact
    let tabs = manager.get_recently_closed(10).await.unwrap();
    assert_eq!(tabs.len(), 5);
}

#[tokio::test]
async fn test_multiple_windows_multiple_tabs_scenario() {
    // GIVEN: Complex session (3 windows, varying tab counts)
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut state = SessionState::new(1234567890);

    // Window 1: 2 tabs
    let mut window1 = WindowState::new("win-1".to_string(), 800, 600);
    window1.tabs.push(TabState::new(
        "tab-1-1".to_string(),
        "https://a.com".to_string(),
        "A".to_string(),
        0,
    ));
    window1.tabs.push(TabState::new(
        "tab-1-2".to_string(),
        "https://b.com".to_string(),
        "B".to_string(),
        1,
    ));
    state.windows.push(window1);

    // Window 2: 3 tabs
    let mut window2 = WindowState::new("win-2".to_string(), 1024, 768);
    window2.tabs.push(TabState::new(
        "tab-2-1".to_string(),
        "https://c.com".to_string(),
        "C".to_string(),
        0,
    ));
    window2.tabs.push(TabState::new(
        "tab-2-2".to_string(),
        "https://d.com".to_string(),
        "D".to_string(),
        1,
    ));
    window2.tabs.push(TabState::new(
        "tab-2-3".to_string(),
        "https://e.com".to_string(),
        "E".to_string(),
        2,
    ));
    state.windows.push(window2);

    // Window 3: 1 tab
    let mut window3 = WindowState::new("win-3".to_string(), 1920, 1080);
    window3.tabs.push(TabState::new(
        "tab-3-1".to_string(),
        "https://f.com".to_string(),
        "F".to_string(),
        0,
    ));
    state.windows.push(window3);

    // WHEN: Saving and restoring
    let session_id = manager.save_session(&state).await.unwrap();
    let restored = manager.get_session(session_id).await.unwrap().unwrap();

    // THEN: All windows and tabs restored correctly
    assert_eq!(restored.windows.len(), 3);
    assert_eq!(restored.windows[0].tabs.len(), 2);
    assert_eq!(restored.windows[1].tabs.len(), 3);
    assert_eq!(restored.windows[2].tabs.len(), 1);
}

#[tokio::test]
async fn test_session_evolution_over_time() {
    // GIVEN: Session saved at T0, T1, T2
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let state_t0 = SessionState::new(1000);
    let state_t1 = SessionState::new(2000);
    let state_t2 = SessionState::new(3000);

    manager.save_session(&state_t0).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    manager.save_session(&state_t1).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    manager.save_session(&state_t2).await.unwrap();

    // WHEN: Restoring at T3
    let restored = manager.restore_session().await.unwrap();

    // THEN: Most recent (T2) session restored
    assert!(restored.is_some());
    assert_eq!(restored.unwrap().timestamp, 3000);
}

#[tokio::test]
async fn test_export_import_backup_workflow() {
    // GIVEN: Active session
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut state = SessionState::new(1234567890);
    let mut window = WindowState::new("win-1".to_string(), 800, 600);
    window.tabs.push(TabState::new(
        "tab-1".to_string(),
        "https://important.com".to_string(),
        "Important".to_string(),
        0,
    ));
    state.windows.push(window);

    let session_id = manager.save_session(&state).await.unwrap();

    // WHEN: Exporting to JSON, clearing DB, reimporting
    let json_backup = manager.export_session(Some(session_id)).await.unwrap();
    manager.clear().await.unwrap();

    assert_eq!(manager.list_sessions().await.unwrap().len(), 0);

    let new_id = manager.import_session(&json_backup).await.unwrap();

    // THEN: Session fully restored from backup
    let restored = manager.get_session(new_id).await.unwrap().unwrap();
    assert_eq!(restored.windows.len(), 1);
    assert_eq!(restored.windows[0].tabs[0].url, "https://important.com");
}

#[tokio::test]
async fn test_partial_session_restore() {
    // GIVEN: Session with some tabs (URLs can be empty)
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let mut state = SessionState::new(1234567890);
    let mut window = WindowState::new("win-1".to_string(), 800, 600);
    window.tabs.push(TabState::new(
        "tab-1".to_string(),
        "https://valid.com".to_string(),
        "Valid".to_string(),
        0,
    ));
    window.tabs.push(TabState::new(
        "tab-2".to_string(),
        "".to_string(), // Empty URL (new tab)
        "New Tab".to_string(),
        1,
    ));
    state.windows.push(window);

    // WHEN: Restoring
    let session_id = manager.save_session(&state).await.unwrap();
    let restored = manager.get_session(session_id).await.unwrap().unwrap();

    // THEN: All tabs restored (empty URLs allowed)
    assert_eq!(restored.windows[0].tabs.len(), 2);
}

#[tokio::test]
async fn test_session_versioning() {
    // GIVEN: Sessions saved at different times
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    let id1 = manager.save_session(&SessionState::new(1000)).await.unwrap();
    let id2 = manager.save_session(&SessionState::new(2000)).await.unwrap();
    let id3 = manager.save_session(&SessionState::new(3000)).await.unwrap();

    // WHEN: Listing all sessions
    let sessions = manager.list_sessions().await.unwrap();

    // THEN: Sessions can be distinguished by ID and timestamp
    assert_eq!(sessions.len(), 3);
    assert!(sessions.iter().any(|(id, ts)| *id == id1 && *ts == 1000));
    assert!(sessions.iter().any(|(id, ts)| *id == id2 && *ts == 2000));
    assert!(sessions.iter().any(|(id, ts)| *id == id3 && *ts == 3000));
}

#[tokio::test]
async fn test_max_closed_tabs_limit() {
    // GIVEN: Recording 1000 closed tabs
    let (_dir, db_path) = get_temp_db();
    let mut manager = SessionManager::new(&db_path).await.unwrap();

    for i in 1..=1000 {
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

    // WHEN: Getting recently closed
    let tabs = manager.get_recently_closed(1000).await.unwrap();

    // THEN: Only reasonable number returned (max 100)
    assert!(tabs.len() <= 100);
}
