use history::storage::HistoryStorage;
use history::types::{HistoryVisit, TransitionType, SearchQuery};
use tempfile::tempdir;

fn create_test_visit(id: &str, url: &str, title: &str, time: i64) -> HistoryVisit {
    HistoryVisit {
        id: id.to_string(),
        url: url.to_string(),
        title: title.to_string(),
        visit_time: time,
        visit_duration: None,
        from_url: None,
        transition_type: TransitionType::Typed,
    }
}

#[tokio::test]
async fn test_storage_create_and_init() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let storage = HistoryStorage::new(db_path.to_str().unwrap()).await;
    assert!(storage.is_ok());
}

#[tokio::test]
async fn test_storage_insert_and_get() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    let visit = create_test_visit("id1", "https://example.com", "Example", 1000);
    storage.insert(&visit).await.unwrap();

    let retrieved = storage.get(&visit.id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), visit);
}

#[tokio::test]
async fn test_storage_get_nonexistent() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    let result = storage.get("nonexistent").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_storage_delete() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    let visit = create_test_visit("id1", "https://example.com", "Example", 1000);
    storage.insert(&visit).await.unwrap();

    storage.delete(&visit.id).await.unwrap();

    let result = storage.get(&visit.id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_storage_get_visits_for_url() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    let url = "https://example.com";
    storage.insert(&create_test_visit("id1", url, "Example 1", 1000)).await.unwrap();
    storage.insert(&create_test_visit("id2", url, "Example 2", 2000)).await.unwrap();
    storage.insert(&create_test_visit("id3", "https://other.com", "Other", 1500)).await.unwrap();

    let visits = storage.get_visits_for_url(url).await.unwrap();
    assert_eq!(visits.len(), 2);
    assert!(visits.iter().all(|v| v.url == url));
}

#[tokio::test]
async fn test_storage_search_by_text() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    storage.insert(&create_test_visit("id1", "https://rust-lang.org", "Rust Programming", 1000)).await.unwrap();
    storage.insert(&create_test_visit("id2", "https://example.com", "Example Site", 2000)).await.unwrap();
    storage.insert(&create_test_visit("id3", "https://rustacean.net", "Rustacean", 3000)).await.unwrap();

    let query = SearchQuery {
        text: Some("rust".to_string()),
        start_time: None,
        end_time: None,
        limit: 10,
    };

    let results = storage.search(&query).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|v| v.url.contains("rust-lang")));
    assert!(results.iter().any(|v| v.url.contains("rustacean")));
}

#[tokio::test]
async fn test_storage_search_by_time_range() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    storage.insert(&create_test_visit("id1", "https://a.com", "A", 1000)).await.unwrap();
    storage.insert(&create_test_visit("id2", "https://b.com", "B", 2000)).await.unwrap();
    storage.insert(&create_test_visit("id3", "https://c.com", "C", 3000)).await.unwrap();

    let query = SearchQuery {
        text: None,
        start_time: Some(1500),
        end_time: Some(2500),
        limit: 10,
    };

    let results = storage.search(&query).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "id2");
}

#[tokio::test]
async fn test_storage_search_limit() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    for i in 0..10 {
        storage.insert(&create_test_visit(&format!("id{}", i), "https://example.com", "Example", i * 1000)).await.unwrap();
    }

    let query = SearchQuery {
        text: None,
        start_time: None,
        end_time: None,
        limit: 5,
    };

    let results = storage.search(&query).await.unwrap();
    assert_eq!(results.len(), 5);
}

#[tokio::test]
async fn test_storage_get_most_visited() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    let url1 = "https://popular.com";
    let url2 = "https://less-popular.com";

    // Visit url1 three times
    storage.insert(&create_test_visit("id1", url1, "Popular", 1000)).await.unwrap();
    storage.insert(&create_test_visit("id2", url1, "Popular", 2000)).await.unwrap();
    storage.insert(&create_test_visit("id3", url1, "Popular", 3000)).await.unwrap();

    // Visit url2 once
    storage.insert(&create_test_visit("id4", url2, "Less Popular", 1500)).await.unwrap();

    let stats = storage.get_most_visited(10).await.unwrap();
    assert_eq!(stats.len(), 2);
    assert_eq!(stats[0].url, url1);
    assert_eq!(stats[0].visit_count, 3);
    assert_eq!(stats[1].url, url2);
    assert_eq!(stats[1].visit_count, 1);
}

#[tokio::test]
async fn test_storage_get_recent() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    storage.insert(&create_test_visit("id1", "https://a.com", "A", 1000)).await.unwrap();
    storage.insert(&create_test_visit("id2", "https://b.com", "B", 2000)).await.unwrap();
    storage.insert(&create_test_visit("id3", "https://c.com", "C", 3000)).await.unwrap();

    let recent = storage.get_recent(2).await.unwrap();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].id, "id3"); // Most recent
    assert_eq!(recent[1].id, "id2");
}

#[tokio::test]
async fn test_storage_clear_older_than() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    storage.insert(&create_test_visit("id1", "https://a.com", "A", 1000)).await.unwrap();
    storage.insert(&create_test_visit("id2", "https://b.com", "B", 2000)).await.unwrap();
    storage.insert(&create_test_visit("id3", "https://c.com", "C", 3000)).await.unwrap();

    let deleted = storage.clear_older_than(2500).await.unwrap();
    assert_eq!(deleted, 2); // id1 and id2

    let remaining = storage.count_visits().await.unwrap();
    assert_eq!(remaining, 1);
}

#[tokio::test]
async fn test_storage_clear_all() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    storage.insert(&create_test_visit("id1", "https://a.com", "A", 1000)).await.unwrap();
    storage.insert(&create_test_visit("id2", "https://b.com", "B", 2000)).await.unwrap();

    storage.clear_all().await.unwrap();

    let count = storage.count_visits().await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_storage_count_visits() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    assert_eq!(storage.count_visits().await.unwrap(), 0);

    storage.insert(&create_test_visit("id1", "https://a.com", "A", 1000)).await.unwrap();
    assert_eq!(storage.count_visits().await.unwrap(), 1);

    storage.insert(&create_test_visit("id2", "https://b.com", "B", 2000)).await.unwrap();
    assert_eq!(storage.count_visits().await.unwrap(), 2);
}

#[tokio::test]
async fn test_storage_count_visits_for_url() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    let url = "https://example.com";
    storage.insert(&create_test_visit("id1", url, "Example", 1000)).await.unwrap();
    storage.insert(&create_test_visit("id2", url, "Example", 2000)).await.unwrap();
    storage.insert(&create_test_visit("id3", "https://other.com", "Other", 1500)).await.unwrap();

    let count = storage.count_visits_for_url(url).await.unwrap();
    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_storage_update_visit_duration() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let mut storage = HistoryStorage::new(db_path.to_str().unwrap()).await.unwrap();

    let visit = create_test_visit("id1", "https://example.com", "Example", 1000);
    storage.insert(&visit).await.unwrap();

    storage.update_visit_duration(&visit.id, 120).await.unwrap();

    let updated = storage.get(&visit.id).await.unwrap().unwrap();
    assert_eq!(updated.visit_duration, Some(120));
}
