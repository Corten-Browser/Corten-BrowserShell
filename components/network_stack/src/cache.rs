//! HTTP caching implementation with memory and disk storage.
//!
//! This module provides caching for HTTP responses following standard cache
//! semantics including Cache-Control, ETag, and Last-Modified headers.

use crate::error::{NetworkError, NetworkResult};
use crate::request::NetworkRequest;
use crate::response::{CacheStatus, NetworkResponse};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// The cached response
    pub response: NetworkResponse,
    /// When this entry was cached
    pub cached_at: DateTime<Utc>,
    /// When this entry expires (if known)
    pub expires_at: Option<DateTime<Utc>>,
    /// ETag for validation
    pub etag: Option<String>,
    /// Last-Modified timestamp for validation
    pub last_modified: Option<String>,
    /// Maximum age in seconds
    pub max_age: Option<u64>,
    /// Whether this can be cached
    pub cacheable: bool,
}

impl CacheEntry {
    /// Create a new cache entry from a response
    pub fn from_response(response: &NetworkResponse) -> Self {
        let cached_at = Utc::now();
        let mut expires_at = None;
        let mut max_age = None;
        let mut cacheable = true;

        // Parse Cache-Control header
        if let Some(cache_control) = response.headers.get("cache-control") {
            let cc = cache_control.to_lowercase();

            if cc.contains("no-store") || cc.contains("no-cache") {
                cacheable = false;
            }

            // Extract max-age
            if let Some(idx) = cc.find("max-age=") {
                let age_str = &cc[idx + 8..];
                if let Some(end) = age_str.find(|c: char| !c.is_ascii_digit()) {
                    if let Ok(age) = age_str[..end].parse::<u64>() {
                        max_age = Some(age);
                        expires_at = Some(cached_at + Duration::seconds(age as i64));
                    }
                } else if let Ok(age) = age_str.parse::<u64>() {
                    max_age = Some(age);
                    expires_at = Some(cached_at + Duration::seconds(age as i64));
                }
            }
        }

        // Parse Expires header if no max-age
        if expires_at.is_none() {
            if let Some(expires) = response.headers.get("expires") {
                if let Ok(dt) = DateTime::parse_from_rfc2822(expires) {
                    expires_at = Some(dt.with_timezone(&Utc));
                }
            }
        }

        let etag = response.headers.get("etag").cloned();
        let last_modified = response.headers.get("last-modified").cloned();

        Self {
            response: response.clone(),
            cached_at,
            expires_at,
            etag,
            last_modified,
            max_age,
            cacheable,
        }
    }

    /// Check if this cache entry is still fresh
    pub fn is_fresh(&self) -> bool {
        if !self.cacheable {
            return false;
        }

        if let Some(expires) = self.expires_at {
            Utc::now() < expires
        } else {
            // No expiration means check if reasonably fresh (default 5 minutes)
            let age = Utc::now().signed_duration_since(self.cached_at);
            age < Duration::minutes(5)
        }
    }

    /// Check if this entry can be revalidated
    pub fn can_revalidate(&self) -> bool {
        self.etag.is_some() || self.last_modified.is_some()
    }
}

/// HTTP cache storage trait
#[async_trait::async_trait]
pub trait CacheStorage: Send + Sync {
    /// Get a cached response
    async fn get(&self, url: &Url) -> NetworkResult<Option<CacheEntry>>;

    /// Store a response in the cache
    async fn put(&self, url: &Url, entry: CacheEntry) -> NetworkResult<()>;

    /// Remove a cached response
    async fn remove(&self, url: &Url) -> NetworkResult<()>;

    /// Clear all cached responses
    async fn clear(&self) -> NetworkResult<()>;

    /// Get the size of the cache in bytes
    async fn size(&self) -> NetworkResult<usize>;
}

/// In-memory cache storage
#[derive(Debug, Clone)]
pub struct MemoryCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    max_size: usize,
    current_size: Arc<RwLock<usize>>,
}

impl MemoryCache {
    /// Create a new memory cache with a maximum size in bytes
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            current_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Estimate the size of a cache entry
    fn estimate_size(entry: &CacheEntry) -> usize {
        entry.response.body.len() + 1024 // Body + metadata overhead
    }

    /// Evict entries if cache is too large
    async fn evict_if_needed(&self, needed: usize) -> NetworkResult<()> {
        let mut entries = self.entries.write().await;
        let mut current_size = self.current_size.write().await;

        while *current_size + needed > self.max_size && !entries.is_empty() {
            // Simple LRU: remove oldest entry
            if let Some((url, entry)) = entries
                .iter()
                .min_by_key(|(_, e)| e.cached_at)
                .map(|(k, v)| (k.clone(), v.clone()))
            {
                entries.remove(&url);
                *current_size = current_size.saturating_sub(Self::estimate_size(&entry));
            }
        }

        Ok(())
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new(50 * 1024 * 1024) // 50MB default
    }
}

#[async_trait::async_trait]
impl CacheStorage for MemoryCache {
    async fn get(&self, url: &Url) -> NetworkResult<Option<CacheEntry>> {
        let entries = self.entries.read().await;
        let entry = entries.get(url.as_str()).cloned();

        if let Some(ref e) = entry {
            if !e.is_fresh() {
                drop(entries);
                // Remove stale entry
                self.remove(url).await?;
                return Ok(None);
            }
        }

        Ok(entry)
    }

    async fn put(&self, url: &Url, entry: CacheEntry) -> NetworkResult<()> {
        if !entry.cacheable {
            return Ok(());
        }

        let size = Self::estimate_size(&entry);
        self.evict_if_needed(size).await?;

        let mut entries = self.entries.write().await;
        let mut current_size = self.current_size.write().await;

        entries.insert(url.to_string(), entry);
        *current_size += size;

        Ok(())
    }

    async fn remove(&self, url: &Url) -> NetworkResult<()> {
        let mut entries = self.entries.write().await;
        let mut current_size = self.current_size.write().await;

        if let Some(entry) = entries.remove(url.as_str()) {
            *current_size = current_size.saturating_sub(Self::estimate_size(&entry));
        }

        Ok(())
    }

    async fn clear(&self) -> NetworkResult<()> {
        let mut entries = self.entries.write().await;
        let mut current_size = self.current_size.write().await;

        entries.clear();
        *current_size = 0;

        Ok(())
    }

    async fn size(&self) -> NetworkResult<usize> {
        let current_size = self.current_size.read().await;
        Ok(*current_size)
    }
}

/// Disk-based cache storage
#[derive(Debug, Clone)]
pub struct DiskCache {
    cache_dir: PathBuf,
    max_size: usize,
}

impl DiskCache {
    /// Create a new disk cache
    pub fn new(cache_dir: PathBuf, max_size: usize) -> NetworkResult<Self> {
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| NetworkError::Internal(format!("Failed to create cache directory: {}", e)))?;

        Ok(Self { cache_dir, max_size })
    }

    /// Get the cache file path for a URL
    fn cache_path(&self, url: &Url) -> PathBuf {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        url.as_str().hash(&mut hasher);
        let hash = hasher.finish();

        self.cache_dir.join(format!("{:x}.cache", hash))
    }
}

#[async_trait::async_trait]
impl CacheStorage for DiskCache {
    async fn get(&self, url: &Url) -> NetworkResult<Option<CacheEntry>> {
        let path = self.cache_path(url);

        if !path.exists() {
            return Ok(None);
        }

        let data = tokio::fs::read(&path).await
            .map_err(|e| NetworkError::Internal(format!("Failed to read cache: {}", e)))?;

        let entry: CacheEntry = serde_json::from_slice(&data)
            .map_err(|e| NetworkError::Internal(format!("Failed to deserialize cache: {}", e)))?;

        if !entry.is_fresh() {
            let _ = tokio::fs::remove_file(&path).await;
            return Ok(None);
        }

        Ok(Some(entry))
    }

    async fn put(&self, url: &Url, entry: CacheEntry) -> NetworkResult<()> {
        if !entry.cacheable {
            return Ok(());
        }

        let path = self.cache_path(url);
        let data = serde_json::to_vec(&entry)
            .map_err(|e| NetworkError::Internal(format!("Failed to serialize cache: {}", e)))?;

        tokio::fs::write(&path, data).await
            .map_err(|e| NetworkError::Internal(format!("Failed to write cache: {}", e)))?;

        Ok(())
    }

    async fn remove(&self, url: &Url) -> NetworkResult<()> {
        let path = self.cache_path(url);
        if path.exists() {
            tokio::fs::remove_file(&path).await
                .map_err(|e| NetworkError::Internal(format!("Failed to remove cache: {}", e)))?;
        }
        Ok(())
    }

    async fn clear(&self) -> NetworkResult<()> {
        let mut entries = tokio::fs::read_dir(&self.cache_dir).await
            .map_err(|e| NetworkError::Internal(format!("Failed to read cache dir: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| NetworkError::Internal(format!("Failed to read entry: {}", e)))? {

            if entry.path().extension().map(|e| e == "cache").unwrap_or(false) {
                let _ = tokio::fs::remove_file(entry.path()).await;
            }
        }

        Ok(())
    }

    async fn size(&self) -> NetworkResult<usize> {
        let mut total = 0;
        let mut entries = tokio::fs::read_dir(&self.cache_dir).await
            .map_err(|e| NetworkError::Internal(format!("Failed to read cache dir: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| NetworkError::Internal(format!("Failed to read entry: {}", e)))? {

            if let Ok(metadata) = entry.metadata().await {
                total += metadata.len() as usize;
            }
        }

        Ok(total)
    }
}

/// Caching interceptor that uses a cache storage backend
#[derive(Debug, Clone)]
pub struct CachingInterceptor<S: CacheStorage> {
    storage: Arc<S>,
}

impl<S: CacheStorage> CachingInterceptor<S> {
    /// Create a new caching interceptor
    pub fn new(storage: S) -> Self {
        Self {
            storage: Arc::new(storage),
        }
    }

    /// Get the storage backend
    pub fn storage(&self) -> Arc<S> {
        Arc::clone(&self.storage)
    }
}

#[async_trait::async_trait]
impl<S: CacheStorage + std::fmt::Debug + 'static> crate::interceptor::RequestInterceptor for CachingInterceptor<S> {
    async fn intercept_request(
        &self,
        request: NetworkRequest,
    ) -> NetworkResult<crate::interceptor::InterceptorOutcome<NetworkRequest>> {
        use crate::interceptor::InterceptorOutcome;
        use crate::request::CacheMode;

        // Check cache mode
        match request.cache_mode {
            CacheMode::NoStore | CacheMode::Reload => {
                // Bypass cache
                return Ok(InterceptorOutcome::Continue(request));
            }
            _ => {}
        }

        // Try to get from cache
        if let Some(entry) = self.storage.get(&request.url).await? {
            if entry.is_fresh() {
                tracing::debug!(url = %request.url, "Cache hit");
                let mut response = entry.response.clone();
                response.cache_status = CacheStatus::Hit;
                return Ok(InterceptorOutcome::ShortCircuit(response));
            } else if entry.can_revalidate() {
                tracing::debug!(url = %request.url, "Cache stale, revalidating");
                // Add conditional headers for revalidation
                let mut req = request;
                if let Some(etag) = &entry.etag {
                    req = req.header("If-None-Match", etag);
                }
                if let Some(last_modified) = &entry.last_modified {
                    req = req.header("If-Modified-Since", last_modified);
                }
                return Ok(InterceptorOutcome::Continue(req));
            }
        }

        tracing::debug!(url = %request.url, "Cache miss");
        Ok(InterceptorOutcome::Continue(request))
    }

    fn name(&self) -> &str {
        "CachingInterceptor"
    }

    fn priority(&self) -> i32 {
        200 // High priority - check cache before anything else
    }
}

#[async_trait::async_trait]
impl<S: CacheStorage + std::fmt::Debug + 'static> crate::interceptor::ResponseInterceptor for CachingInterceptor<S> {
    async fn intercept_response(
        &self,
        request: &NetworkRequest,
        response: NetworkResponse,
    ) -> NetworkResult<NetworkResponse> {
        use crate::request::CacheMode;

        // Don't cache if mode is NoStore
        if matches!(request.cache_mode, CacheMode::NoStore) {
            return Ok(response);
        }

        // Handle 304 Not Modified
        if response.status.as_u16() == 304 {
            // Return cached response
            if let Some(entry) = self.storage.get(&request.url).await? {
                tracing::debug!(url = %request.url, "304 Not Modified, using cached response");
                let mut cached = entry.response;
                cached.cache_status = CacheStatus::Revalidated;
                return Ok(cached);
            }
        }

        // Cache successful responses
        if response.status.is_success() {
            let entry = CacheEntry::from_response(&response);
            if entry.cacheable {
                tracing::debug!(url = %request.url, cacheable = entry.cacheable, "Caching response");
                self.storage.put(&request.url, entry).await?;
            }
        }

        Ok(response)
    }

    fn name(&self) -> &str {
        "CachingInterceptor"
    }

    fn priority(&self) -> i32 {
        200
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::response::StatusCode;

    #[test]
    fn test_cache_entry_fresh() {
        let mut response = NetworkResponse::new(StatusCode::OK, Url::parse("https://example.com").unwrap());
        response.headers.insert("cache-control".to_string(), "max-age=3600".to_string());

        let entry = CacheEntry::from_response(&response);
        assert!(entry.is_fresh());
        assert_eq!(entry.max_age, Some(3600));
    }

    #[test]
    fn test_cache_entry_no_store() {
        let mut response = NetworkResponse::new(StatusCode::OK, Url::parse("https://example.com").unwrap());
        response.headers.insert("cache-control".to_string(), "no-store".to_string());

        let entry = CacheEntry::from_response(&response);
        assert!(!entry.cacheable);
    }

    #[tokio::test]
    async fn test_memory_cache() {
        let cache = MemoryCache::new(1024 * 1024);
        let url = Url::parse("https://example.com").unwrap();

        let mut response = NetworkResponse::new(StatusCode::OK, url.clone());
        response.headers.insert("cache-control".to_string(), "max-age=3600".to_string());
        response.body = b"test body".to_vec();

        let entry = CacheEntry::from_response(&response);
        cache.put(&url, entry.clone()).await.unwrap();

        let cached = cache.get(&url).await.unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().response.body, b"test body");
    }

    #[tokio::test]
    async fn test_memory_cache_clear() {
        let cache = MemoryCache::new(1024 * 1024);
        let url = Url::parse("https://example.com").unwrap();

        let mut response = NetworkResponse::new(StatusCode::OK, url.clone());
        response.headers.insert("cache-control".to_string(), "max-age=3600".to_string());

        let entry = CacheEntry::from_response(&response);
        cache.put(&url, entry).await.unwrap();

        cache.clear().await.unwrap();
        let cached = cache.get(&url).await.unwrap();
        assert!(cached.is_none());
    }
}
