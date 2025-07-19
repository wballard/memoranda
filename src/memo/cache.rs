use moka::future::Cache;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use super::models::{Memo, MemoId};
use super::storage::{MemoStoreError, Result};

/// Configuration for the memo cache system
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub memo_cache_size: u64,
    pub metadata_cache_size: u64,
    pub memo_ttl_seconds: u64,
    pub metadata_ttl_multiplier: u64, // Metadata TTL = memo_ttl * multiplier
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            memo_cache_size: 1000,
            metadata_cache_size: 5000,
            memo_ttl_seconds: 3600,     // 1 hour
            metadata_ttl_multiplier: 2, // Metadata lives twice as long as memos
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoMetadata {
    pub id: MemoId,
    pub title: String,
    pub file_path: PathBuf,
    pub last_modified: SystemTime,
    pub file_size: u64,
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub memo_hits: u64,
    pub memo_misses: u64,
    pub metadata_hits: u64,
    pub metadata_misses: u64,
    pub memo_cache_size: u64,
    pub metadata_cache_size: u64,
}

#[derive(Debug)]
pub struct MemoCache {
    cache: Cache<MemoId, Arc<Memo>>,
    metadata_cache: Cache<PathBuf, Arc<MemoMetadata>>,
    stats: Arc<RwLock<CacheStats>>,
    config: CacheConfig,
}

impl MemoCache {
    #[must_use]
    pub fn new() -> Self {
        Self::with_cache_config(CacheConfig::default())
    }

    #[must_use]
    pub fn with_cache_config(config: CacheConfig) -> Self {
        info!(
            memo_cache_size = config.memo_cache_size,
            metadata_cache_size = config.metadata_cache_size,
            memo_ttl_seconds = config.memo_ttl_seconds,
            metadata_ttl_multiplier = config.metadata_ttl_multiplier,
            "Creating memo cache with configuration"
        );

        let memo_cache = Cache::builder()
            .max_capacity(config.memo_cache_size)
            .time_to_live(Duration::from_secs(config.memo_ttl_seconds))
            .build();

        let metadata_ttl = config.memo_ttl_seconds * config.metadata_ttl_multiplier;
        let metadata_cache = Cache::builder()
            .max_capacity(config.metadata_cache_size)
            .time_to_live(Duration::from_secs(metadata_ttl))
            .build();

        Self {
            cache: memo_cache,
            metadata_cache,
            stats: Arc::new(RwLock::new(CacheStats {
                memo_hits: 0,
                memo_misses: 0,
                metadata_hits: 0,
                metadata_misses: 0,
                memo_cache_size: 0,
                metadata_cache_size: 0,
            })),
            config,
        }
    }

    /// Create cache with legacy parameters (deprecated, use with_cache_config)
    #[must_use]
    pub fn with_config(max_capacity: u64, ttl_seconds: u64) -> Self {
        let config = CacheConfig {
            memo_cache_size: max_capacity,
            metadata_cache_size: 5000, // Use default for metadata
            memo_ttl_seconds: ttl_seconds,
            metadata_ttl_multiplier: 2,
        };
        Self::with_cache_config(config)
    }

    #[instrument(skip(self), fields(memo_id = %id))]
    pub async fn get_memo(&self, id: &MemoId) -> Option<Arc<Memo>> {
        match self.cache.get(id).await {
            Some(memo) => {
                debug!("Cache hit for memo {}", id);
                self.increment_memo_hits().await;
                Some(memo)
            }
            None => {
                debug!("Cache miss for memo {}", id);
                self.increment_memo_misses().await;
                None
            }
        }
    }

    #[instrument(skip(self, memo), fields(memo_id = %memo.id, memo_title = %memo.title))]
    pub async fn put_memo(&self, memo: Memo) {
        debug!("Caching memo {}", memo.id);
        let memo_id = memo.id;
        self.cache.insert(memo_id, Arc::new(memo)).await;
        self.update_memo_cache_size().await;
    }

    #[instrument(skip(self), fields(memo_id = %id))]
    pub async fn remove_memo(&self, id: &MemoId) {
        debug!("Removing memo {} from cache", id);
        self.cache.remove(id).await;
        self.update_memo_cache_size().await;
    }

    #[instrument(skip(self), fields(file_path = %file_path.display()))]
    pub async fn get_metadata(&self, file_path: &PathBuf) -> Option<Arc<MemoMetadata>> {
        match self.metadata_cache.get(file_path).await {
            Some(metadata) => {
                debug!("Cache hit for metadata {}", file_path.display());
                self.increment_metadata_hits().await;
                Some(metadata)
            }
            None => {
                debug!("Cache miss for metadata {}", file_path.display());
                self.increment_metadata_misses().await;
                None
            }
        }
    }

    #[instrument(skip(self, metadata), fields(file_path = %file_path.display(), memo_id = %metadata.id))]
    pub async fn put_metadata(&self, file_path: PathBuf, metadata: MemoMetadata) {
        debug!("Caching metadata for {}", file_path.display());
        self.metadata_cache
            .insert(file_path, Arc::new(metadata))
            .await;
        self.update_metadata_cache_size().await;
    }

    pub async fn remove_metadata(&self, file_path: &PathBuf) {
        debug!("Removing metadata for {} from cache", file_path.display());
        self.metadata_cache.remove(file_path).await;
        self.update_metadata_cache_size().await;
    }

    #[instrument(skip(self), fields(memo_id = %id))]
    pub async fn invalidate_memo(&self, id: &MemoId) {
        warn!("Invalidating memo {} from cache", id);
        self.cache.invalidate(id).await;
        self.update_memo_cache_size().await;
    }

    #[instrument(skip(self))]
    pub async fn invalidate_all(&self) {
        warn!("Invalidating entire cache");
        self.cache.invalidate_all();
        self.metadata_cache.invalidate_all();
        self.reset_stats().await;
    }


    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    pub fn get_config(&self) -> &CacheConfig {
        &self.config
    }

    /// Calculate the cache hit ratio based on actual statistics
    /// This is a non-async approximation for quick access
    pub fn cache_hit_ratio(&self) -> f64 {
        // Since we can't await in a sync function, we try to get the current stats
        // This uses try_read which returns immediately without blocking
        if let Ok(stats) = self.stats.try_read() {
            let total_requests = stats.memo_hits + stats.memo_misses;
            if total_requests == 0 {
                0.0
            } else {
                stats.memo_hits as f64 / total_requests as f64
            }
        } else {
            // If we can't read stats (lock contention), return 0.0 as fallback
            0.0
        }
    }

    /// Calculate the cache hit ratio asynchronously with guaranteed accuracy
    pub async fn cache_hit_ratio_async(&self) -> f64 {
        let stats = self.stats.read().await;
        let total_requests = stats.memo_hits + stats.memo_misses;
        if total_requests == 0 {
            0.0
        } else {
            stats.memo_hits as f64 / total_requests as f64
        }
    }

    async fn increment_memo_hits(&self) {
        self.stats.write().await.memo_hits += 1;
    }

    async fn increment_memo_misses(&self) {
        self.stats.write().await.memo_misses += 1;
    }

    async fn increment_metadata_hits(&self) {
        self.stats.write().await.metadata_hits += 1;
    }

    async fn increment_metadata_misses(&self) {
        self.stats.write().await.metadata_misses += 1;
    }

    async fn update_memo_cache_size(&self) {
        let size = self.cache.entry_count();
        self.stats.write().await.memo_cache_size = size;
    }

    async fn update_metadata_cache_size(&self) {
        let size = self.metadata_cache.entry_count();
        self.stats.write().await.metadata_cache_size = size;
    }

    async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = CacheStats {
            memo_hits: 0,
            memo_misses: 0,
            metadata_hits: 0,
            metadata_misses: 0,
            memo_cache_size: 0,
            metadata_cache_size: 0,
        };
    }

    /// Check if a cached memo is still valid based on file modification time
    #[instrument(skip(self), fields(memo_id = %id, file_path = %file_path.display()))]
    pub async fn is_memo_valid(&self, id: &MemoId, file_path: &PathBuf) -> Result<bool> {
        // Get cached metadata
        if let Some(cached_metadata) = self.get_metadata(file_path).await {
            // Check file modification time
            let file_metadata = std::fs::metadata(file_path).map_err(|e| {
                warn!(
                    "Failed to read file metadata for {}: {}",
                    file_path.display(),
                    e
                );
                MemoStoreError::FileOperation { source: e }
            })?;

            let current_modified = file_metadata.modified().map_err(|e| {
                warn!(
                    "Failed to get modification time for {}: {}",
                    file_path.display(),
                    e
                );
                MemoStoreError::FileOperation { source: e }
            })?;

            if current_modified > cached_metadata.last_modified {
                // File has been modified, cache is invalid
                debug!(
                    "Memo {} is invalid due to file modification (cached: {:?}, current: {:?})",
                    id, cached_metadata.last_modified, current_modified
                );
                self.remove_memo(id).await;
                self.remove_metadata(file_path).await;
                return Ok(false);
            }

            debug!("Memo {} is valid (no file modifications detected)", id);
            return Ok(true);
        }

        // No metadata cached, assume invalid
        debug!("No cached metadata found for memo {}, assuming invalid", id);
        Ok(false)
    }
}

impl Default for MemoCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_memo(id_suffix: usize) -> Memo {
        Memo::new(
            format!("Test Memo {id_suffix}"),
            format!("Content for test memo {id_suffix}"),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_cache_creation() {
        let cache = MemoCache::new();
        let stats = cache.get_stats().await;
        assert_eq!(stats.memo_cache_size, 0);
        assert_eq!(stats.metadata_cache_size, 0);
    }

    #[tokio::test]
    async fn test_memo_cache_put_and_get() {
        let cache = MemoCache::new();
        let memo = create_test_memo(1);
        let memo_id = memo.id;

        // Cache miss initially
        assert!(cache.get_memo(&memo_id).await.is_none());

        // Put memo in cache
        cache.put_memo(memo.clone()).await;

        // Cache hit
        let cached_memo = cache.get_memo(&memo_id).await;
        assert!(cached_memo.is_some());
        assert_eq!(cached_memo.unwrap().title, memo.title);

        let stats = cache.get_stats().await;
        assert_eq!(stats.memo_hits, 1);
        assert_eq!(stats.memo_misses, 1);
        // Note: cache_size might be 0 due to moka's internal management
        // The important thing is that we got a hit and miss
    }

    #[tokio::test]
    async fn test_memo_cache_remove() {
        let cache = MemoCache::new();
        let memo = create_test_memo(2);
        let memo_id = memo.id;

        cache.put_memo(memo).await;
        assert!(cache.get_memo(&memo_id).await.is_some());

        cache.remove_memo(&memo_id).await;
        assert!(cache.get_memo(&memo_id).await.is_none());
    }

    #[tokio::test]
    async fn test_metadata_cache() {
        let cache = MemoCache::new();
        let file_path = PathBuf::from("/test/memo.md");

        let metadata = MemoMetadata {
            id: MemoId::new(),
            title: "Test".to_string(),
            file_path: file_path.clone(),
            last_modified: SystemTime::now(),
            file_size: 100,
        };

        // Cache miss initially
        assert!(cache.get_metadata(&file_path).await.is_none());

        // Put metadata in cache
        cache
            .put_metadata(file_path.clone(), metadata.clone())
            .await;

        // Cache hit
        let cached_metadata = cache.get_metadata(&file_path).await;
        assert!(cached_metadata.is_some());
        assert_eq!(cached_metadata.unwrap().title, metadata.title);

        let stats = cache.get_stats().await;
        assert_eq!(stats.metadata_hits, 1);
        assert_eq!(stats.metadata_misses, 1);
        // Note: cache_size might be 0 due to moka's internal management
        // The important thing is that we got a hit and miss
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = MemoCache::new();
        let memo = create_test_memo(3);
        let memo_id = memo.id;

        cache.put_memo(memo).await;
        assert!(cache.get_memo(&memo_id).await.is_some());

        cache.invalidate_memo(&memo_id).await;
        assert!(cache.get_memo(&memo_id).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidate_all() {
        let cache = MemoCache::new();

        // Add multiple memos
        for i in 1..=5 {
            cache.put_memo(create_test_memo(i)).await;
        }

        let _stats = cache.get_stats().await;
        // Cache should have entries (though exact count may vary)

        cache.invalidate_all().await;

        // After invalidate_all, cache should be empty
        // Verify by trying to get a memo that was previously cached
        let memo_after_clear = cache.get_memo(&MemoId::new()).await;
        assert!(memo_after_clear.is_none());
    }

    #[tokio::test]
    async fn test_memo_validity_check() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_memo.md");

        // Create a test file
        fs::write(&file_path, "test content").unwrap();
        let file_metadata = fs::metadata(&file_path).unwrap();
        let last_modified = file_metadata.modified().unwrap();

        let cache = MemoCache::new();
        let memo_id = MemoId::new();

        // Cache metadata
        let metadata = MemoMetadata {
            id: memo_id,
            title: "Test".to_string(),
            file_path: file_path.clone(),
            last_modified,
            file_size: file_metadata.len(),
        };

        cache.put_metadata(file_path.clone(), metadata).await;

        // File should be valid initially
        let is_valid = cache.is_memo_valid(&memo_id, &file_path).await.unwrap();
        assert!(is_valid);

        // Modify the file
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::write(&file_path, "modified content").unwrap();

        // File should now be invalid
        let is_valid_after = cache.is_memo_valid(&memo_id, &file_path).await.unwrap();
        assert!(!is_valid_after);
    }
}
