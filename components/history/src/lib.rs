pub mod types;
pub mod validation;
pub mod frecency;
pub mod storage;

use crate::storage::HistoryStorage;
use crate::types::{HistoryVisit, PageStats, SearchQuery, VisitId};
use crate::validation::{validate_timestamp, validate_title, validate_url};
use anyhow::{Context, Result};
use uuid::Uuid;

/// History manager interface
pub struct HistoryManager {
    storage: HistoryStorage,
}

impl HistoryManager {
    /// Create new history manager with database path
    pub async fn new(db_path: &str) -> Result<Self> {
        let storage = HistoryStorage::new(db_path)
            .await
            .context("Failed to create history storage")?;

        Ok(Self { storage })
    }

    /// Record a visit
    pub async fn record_visit(&mut self, mut visit: HistoryVisit) -> Result<VisitId> {
        // Validate inputs
        validate_url(&visit.url)?;
        validate_timestamp(visit.visit_time)?;

        // Sanitize title
        visit.title = validate_title(&visit.title);

        // Generate ID if not provided
        if visit.id.is_empty() {
            visit.id = Uuid::new_v4().to_string();
        }

        let id = visit.id.clone();
        self.storage.insert(&visit).await?;

        Ok(id)
    }

    /// Get visit by ID
    pub async fn get_visit(&self, id: &VisitId) -> Result<Option<HistoryVisit>> {
        self.storage.get(id).await
    }

    /// Delete visit
    pub async fn delete_visit(&mut self, id: &VisitId) -> Result<()> {
        self.storage.delete(id).await
    }

    /// Get all visits for a URL
    pub async fn get_visits_for_url(&self, url: &str) -> Result<Vec<HistoryVisit>> {
        validate_url(url)?;
        self.storage.get_visits_for_url(url).await
    }

    /// Search history
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<HistoryVisit>> {
        self.storage.search(&query).await
    }

    /// Get most visited pages
    pub async fn get_most_visited(&self, limit: usize) -> Result<Vec<PageStats>> {
        self.storage.get_most_visited(limit).await
    }

    /// Get recent visits
    pub async fn get_recent(&self, limit: usize) -> Result<Vec<HistoryVisit>> {
        self.storage.get_recent(limit).await
    }

    /// Get frecency-scored pages (frequency + recency)
    pub async fn get_frecent(&self, limit: usize) -> Result<Vec<PageStats>> {
        let current_time = chrono::Utc::now().timestamp();
        self.storage.get_frecent(limit, current_time).await
    }

    /// Clear history older than timestamp
    pub async fn clear_older_than(&mut self, timestamp: i64) -> Result<usize> {
        validate_timestamp(timestamp)?;
        self.storage.clear_older_than(timestamp).await
    }

    /// Clear all history
    pub async fn clear_all(&mut self) -> Result<()> {
        self.storage.clear_all().await
    }

    /// Get total visit count
    pub async fn count_visits(&self) -> Result<usize> {
        self.storage.count_visits().await
    }

    /// Get visit count for specific URL
    pub async fn count_visits_for_url(&self, url: &str) -> Result<usize> {
        validate_url(url)?;
        self.storage.count_visits_for_url(url).await
    }

    /// Update visit duration (when tab closes)
    pub async fn update_visit_duration(&mut self, id: &VisitId, duration: i64) -> Result<()> {
        if duration < 0 {
            return Err(anyhow::anyhow!("Duration cannot be negative"));
        }
        self.storage.update_visit_duration(id, duration).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TransitionType;

    fn create_test_visit(url: &str, title: &str, time: i64) -> HistoryVisit {
        HistoryVisit {
            id: String::new(),
            url: url.to_string(),
            title: title.to_string(),
            visit_time: time,
            visit_duration: None,
            from_url: None,
            transition_type: TransitionType::Typed,
        }
    }

    #[tokio::test]
    async fn test_manager_new() {
        let manager = HistoryManager::new(":memory:").await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_record_visit_generates_id() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();
        let visit = create_test_visit("https://example.com", "Example", 1000000000);

        let id = manager.record_visit(visit).await.unwrap();
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn test_record_visit_validates_url() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();
        let visit = create_test_visit("invalid url", "Test", 1000000000);

        let result = manager.record_visit(visit).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_record_visit_validates_timestamp() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();
        let visit = create_test_visit("https://example.com", "Example", -1);

        let result = manager.record_visit(visit).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_record_visit_sanitizes_title() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();
        let visit = create_test_visit("https://example.com", "   ", 1000000000);

        let id = manager.record_visit(visit).await.unwrap();
        let retrieved = manager.get_visit(&id).await.unwrap().unwrap();

        assert_eq!(retrieved.title, "Untitled");
    }

    #[tokio::test]
    async fn test_get_visit() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();
        let visit = create_test_visit("https://example.com", "Example", 1000000000);

        let id = manager.record_visit(visit.clone()).await.unwrap();
        let retrieved = manager.get_visit(&id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().url, visit.url);
    }

    #[tokio::test]
    async fn test_delete_visit() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();
        let visit = create_test_visit("https://example.com", "Example", 1000000000);

        let id = manager.record_visit(visit).await.unwrap();
        manager.delete_visit(&id).await.unwrap();

        let retrieved = manager.get_visit(&id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_get_visits_for_url() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();
        let url = "https://example.com";

        manager.record_visit(create_test_visit(url, "Test 1", 1000000000)).await.unwrap();
        manager.record_visit(create_test_visit(url, "Test 2", 1000000100)).await.unwrap();
        manager.record_visit(create_test_visit("https://other.com", "Other", 1000000050)).await.unwrap();

        let visits = manager.get_visits_for_url(url).await.unwrap();
        assert_eq!(visits.len(), 2);
    }

    #[tokio::test]
    async fn test_search() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();

        manager.record_visit(create_test_visit("https://rust-lang.org", "Rust", 1000000000)).await.unwrap();
        manager.record_visit(create_test_visit("https://example.com", "Example", 1000000100)).await.unwrap();

        let query = SearchQuery {
            text: Some("rust".to_string()),
            start_time: None,
            end_time: None,
            limit: 10,
        };

        let results = manager.search(query).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_get_most_visited() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();
        let popular_url = "https://popular.com";

        for i in 0..5 {
            manager.record_visit(create_test_visit(popular_url, "Popular", 1000000000 + i)).await.unwrap();
        }
        manager.record_visit(create_test_visit("https://other.com", "Other", 1000000000)).await.unwrap();

        let stats = manager.get_most_visited(10).await.unwrap();
        assert_eq!(stats[0].url, popular_url);
        assert_eq!(stats[0].visit_count, 5);
    }

    #[tokio::test]
    async fn test_get_recent() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();

        manager.record_visit(create_test_visit("https://a.com", "A", 1000000000)).await.unwrap();
        manager.record_visit(create_test_visit("https://b.com", "B", 1000000100)).await.unwrap();
        manager.record_visit(create_test_visit("https://c.com", "C", 1000000200)).await.unwrap();

        let recent = manager.get_recent(2).await.unwrap();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].url, "https://c.com");
    }

    #[tokio::test]
    async fn test_clear_older_than() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();

        manager.record_visit(create_test_visit("https://a.com", "A", 1000000000)).await.unwrap();
        manager.record_visit(create_test_visit("https://b.com", "B", 1000000100)).await.unwrap();
        manager.record_visit(create_test_visit("https://c.com", "C", 1000000200)).await.unwrap();

        let deleted = manager.clear_older_than(1000000150).await.unwrap();
        assert_eq!(deleted, 2);

        let remaining = manager.count_visits().await.unwrap();
        assert_eq!(remaining, 1);
    }

    #[tokio::test]
    async fn test_clear_all() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();

        manager.record_visit(create_test_visit("https://a.com", "A", 1000000000)).await.unwrap();
        manager.record_visit(create_test_visit("https://b.com", "B", 1000000100)).await.unwrap();

        manager.clear_all().await.unwrap();

        let count = manager.count_visits().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_count_visits() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();

        assert_eq!(manager.count_visits().await.unwrap(), 0);

        manager.record_visit(create_test_visit("https://a.com", "A", 1000000000)).await.unwrap();
        assert_eq!(manager.count_visits().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_count_visits_for_url() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();
        let url = "https://example.com";

        manager.record_visit(create_test_visit(url, "Test 1", 1000000000)).await.unwrap();
        manager.record_visit(create_test_visit(url, "Test 2", 1000000100)).await.unwrap();
        manager.record_visit(create_test_visit("https://other.com", "Other", 1000000050)).await.unwrap();

        let count = manager.count_visits_for_url(url).await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_update_visit_duration() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();
        let visit = create_test_visit("https://example.com", "Example", 1000000000);

        let id = manager.record_visit(visit).await.unwrap();
        manager.update_visit_duration(&id, 120).await.unwrap();

        let updated = manager.get_visit(&id).await.unwrap().unwrap();
        assert_eq!(updated.visit_duration, Some(120));
    }

    #[tokio::test]
    async fn test_update_visit_duration_negative() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();
        let visit = create_test_visit("https://example.com", "Example", 1000000000);

        let id = manager.record_visit(visit).await.unwrap();
        let result = manager.update_visit_duration(&id, -1).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_frecent() {
        let mut manager = HistoryManager::new(":memory:").await.unwrap();
        let now = chrono::Utc::now().timestamp();

        // Recent visits should score higher
        for i in 0..3 {
            manager.record_visit(create_test_visit("https://recent.com", "Recent", now - i * 3600)).await.unwrap();
        }

        // Old visits should score lower
        for i in 0..5 {
            manager.record_visit(create_test_visit("https://old.com", "Old", now - 365 * 86400 - i * 3600)).await.unwrap();
        }

        let frecent = manager.get_frecent(10).await.unwrap();

        // Recent page should be first despite fewer visits
        assert_eq!(frecent[0].url, "https://recent.com");
        assert!(frecent[0].frecency_score > frecent[1].frecency_score);
    }
}
