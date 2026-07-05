//! In-memory cache implementation
//!
//! This module provides a high-performance in-memory cache using DashMap
//! for lock-free concurrent access with LRU eviction support.
//!
//! Hot-path eviction metadata is maintained with per-entry atomics in sharded
//! indexes so cache hits and writes do not serialize on a global LRU lock.

use super::types::{AtomicCacheStats, CacheEntry, CacheKey, DualCacheConfig, EvictionPolicy};
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tracing::{debug, trace};

const EVICTION_SAMPLE_SIZE: usize = 64;
const MIN_ACCESS_SHARDS: usize = 4;
const MAX_ACCESS_SHARDS: usize = 64;

#[derive(Debug)]
struct CacheAccessMeta {
    last_access_tick: AtomicU64,
    access_count: AtomicU64,
}

impl CacheAccessMeta {
    fn new(insert_tick: u64) -> Self {
        Self {
            last_access_tick: AtomicU64::new(insert_tick),
            access_count: AtomicU64::new(0),
        }
    }

    fn record_access(&self, tick: u64) -> u64 {
        self.last_access_tick.store(tick, Ordering::Relaxed);
        self.access_count.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn reset_for_insert(&self, tick: u64) {
        self.last_access_tick.store(tick, Ordering::Relaxed);
        self.access_count.store(0, Ordering::Relaxed);
    }

    fn snapshot(&self) -> (u64, u64) {
        (
            self.last_access_tick.load(Ordering::Relaxed),
            self.access_count.load(Ordering::Relaxed),
        )
    }
}

#[derive(Debug)]
struct EvictionCandidate {
    key: CacheKey,
    last_access_tick: u64,
    access_count: u64,
    created_at: Instant,
    remaining_ttl: Option<Duration>,
}

/// In-memory cache with LRU eviction and TTL expiration
pub struct InMemoryCache<T> {
    /// Main cache storage using DashMap for lock-free access
    cache: Arc<DashMap<CacheKey, CacheEntry<T>>>,
    /// Sharded eviction metadata. Shard count defaults to available CPU
    /// parallelism rounded to a bounded power of two.
    access_meta: Arc<Vec<DashMap<CacheKey, CacheAccessMeta>>>,
    /// Monotonic logical clock for access ordering.
    access_clock: AtomicU64,
    /// Rotates the first shard inspected by sampled eviction.
    eviction_cursor: AtomicUsize,
    /// Configuration
    config: DualCacheConfig,
    /// Statistics
    stats: Arc<AtomicCacheStats>,
    /// Shutdown signal
    shutdown: Arc<AtomicBool>,
    /// Notify for shutdown
    shutdown_notify: Arc<Notify>,
}

impl<T: Clone + Send + Sync + 'static> InMemoryCache<T> {
    /// Create a new in-memory cache with the given configuration
    pub fn new(config: DualCacheConfig) -> Self {
        Self::with_stats(config, Arc::new(AtomicCacheStats::new()))
    }

    /// Create a new in-memory cache with shared statistics
    pub fn with_stats(config: DualCacheConfig, stats: Arc<AtomicCacheStats>) -> Self {
        let cache = Arc::new(DashMap::with_capacity(config.max_size));
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_notify = Arc::new(Notify::new());
        let access_meta = Arc::new(
            (0..default_access_shard_count())
                .map(|_| DashMap::new())
                .collect(),
        );

        Self {
            cache,
            access_meta,
            access_clock: AtomicU64::new(0),
            eviction_cursor: AtomicUsize::new(0),
            config,
            stats,
            shutdown,
            shutdown_notify,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(DualCacheConfig::memory_only())
    }

    /// Start the background cleanup task
    pub fn start_cleanup_task(self: &Arc<Self>) {
        let cache = Arc::clone(self);
        let interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(interval) => {
                        cache.cleanup_expired().await;
                    }
                    _ = cache.shutdown_notify.notified() => {
                        debug!("In-memory cache cleanup task shutting down");
                        break;
                    }
                }
            }
        });
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &CacheKey) -> Option<T> {
        // Atomically remove expired entries to avoid TOCTOU race
        if let Some((_, removed)) = self.cache.remove_if(key, |_k, v| v.is_expired()) {
            self.remove_access_meta(key);
            self.stats.sub_total_size(removed.size_bytes);
            self.stats.set_entry_count(self.cache.len());
            self.stats.record_memory_miss();
            trace!(key = %key, "Cache entry expired");
            return None;
        }

        if let Some(entry) = self.cache.get(key) {
            let value = entry.value.clone();
            drop(entry);
            self.record_access(key);
            self.stats.record_memory_hit();
            trace!(key = %key, "Cache hit");
            Some(value)
        } else {
            self.stats.record_memory_miss();
            trace!(key = %key, "Cache miss");
            None
        }
    }

    /// Get an entry with metadata from the cache
    pub async fn get_entry(&self, key: &CacheKey) -> Option<CacheEntry<T>> {
        // Atomically remove expired entries to avoid TOCTOU race
        if let Some((_, removed)) = self.cache.remove_if(key, |_k, v| v.is_expired()) {
            self.remove_access_meta(key);
            self.stats.sub_total_size(removed.size_bytes);
            self.stats.set_entry_count(self.cache.len());
            self.stats.record_memory_miss();
            return None;
        }

        if let Some(entry) = self.cache.get(key) {
            let mut snapshot = entry.clone();
            drop(entry);
            let access_count = self.record_access(key);
            snapshot.access_count = access_count;
            snapshot.last_accessed = Instant::now();
            self.stats.record_memory_hit();
            Some(snapshot)
        } else {
            self.stats.record_memory_miss();
            None
        }
    }

    /// Set a value in the cache with the default TTL
    pub async fn set(&self, key: CacheKey, value: T) {
        self.set_with_ttl(key, value, self.config.default_ttl).await;
    }

    /// Set a value in the cache with a specific TTL
    pub async fn set_with_ttl(&self, key: CacheKey, value: T, ttl: Duration) {
        // Check if we need to evict entries
        if self.cache.len() >= self.config.max_size {
            self.evict_one().await;
        }

        let entry = CacheEntry::new(value, ttl);
        let new_size = entry.size_bytes;
        self.reset_access_meta_for_insert(&key);
        // Atomic insert returns the old entry if key existed (no TOCTOU gap)
        let old = self.cache.insert(key.clone(), entry);
        self.stats.record_write();

        if let Some(old_entry) = old {
            self.stats.sub_total_size(old_entry.size_bytes);
        }

        self.stats.add_total_size(new_size);
        self.stats.set_entry_count(self.cache.len());
        trace!(key = %key, ttl_secs = ttl.as_secs(), "Cache set");
    }

    /// Set a value with size tracking
    pub async fn set_with_size(&self, key: CacheKey, value: T, ttl: Duration, size_bytes: usize) {
        if self.cache.len() >= self.config.max_size {
            self.evict_one().await;
        }

        let entry = CacheEntry::with_size(value, ttl, size_bytes);
        let new_size = entry.size_bytes;
        self.reset_access_meta_for_insert(&key);
        // Atomic insert returns the old entry if key existed (no TOCTOU gap)
        let old = self.cache.insert(key.clone(), entry);
        self.stats.record_write();

        if let Some(old_entry) = old {
            self.stats.sub_total_size(old_entry.size_bytes);
        }

        self.stats.add_total_size(new_size);
        self.stats.set_entry_count(self.cache.len());
    }

    /// Delete a value from the cache
    pub async fn delete(&self, key: &CacheKey) -> bool {
        if let Some((_, removed)) = self.cache.remove(key) {
            self.remove_access_meta(key);
            self.stats.record_deletion();
            self.stats.sub_total_size(removed.size_bytes);
            self.stats.set_entry_count(self.cache.len());
            trace!(key = %key, "Cache delete");
            true
        } else {
            false
        }
    }

    /// Check if a key exists in the cache
    pub async fn exists(&self, key: &CacheKey) -> bool {
        // Atomically remove expired entries to avoid TOCTOU race
        if self.cache.remove_if(key, |_k, v| v.is_expired()).is_some() {
            self.remove_access_meta(key);
            self.stats.set_entry_count(self.cache.len());
            return false;
        }
        self.cache.contains_key(key)
    }

    /// Get the remaining TTL for a key
    pub fn ttl(&self, key: &CacheKey) -> Option<Duration> {
        if let Some(entry) = self.cache.get(key) {
            entry.remaining_ttl()
        } else {
            None
        }
    }

    /// Clear all entries from the cache
    pub async fn clear(&self) {
        self.cache.clear();
        for shard in self.access_meta.iter() {
            shard.clear();
        }
        self.access_clock.store(0, Ordering::Relaxed);
        self.eviction_cursor.store(0, Ordering::Relaxed);
        self.stats.reset();
        debug!("Cache cleared");
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get cache statistics
    pub fn stats(&self) -> Arc<AtomicCacheStats> {
        Arc::clone(&self.stats)
    }

    /// Get all keys in the cache
    pub fn keys(&self) -> Vec<CacheKey> {
        self.cache.iter().map(|r| r.key().clone()).collect()
    }

    /// Shutdown the cache and cleanup task
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.shutdown_notify.notify_waiters();
    }

    // ==================== Private Methods ====================

    fn next_access_tick(&self) -> u64 {
        self.access_clock.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn access_shard(&self, key: &CacheKey) -> &DashMap<CacheKey, CacheAccessMeta> {
        let index = key.hash_value() as usize % self.access_meta.len();
        &self.access_meta[index]
    }

    fn reset_access_meta_for_insert(&self, key: &CacheKey) {
        let tick = self.next_access_tick();
        let shard = self.access_shard(key);

        if let Some(meta) = shard.get(key) {
            meta.reset_for_insert(tick);
        } else {
            shard.insert(key.clone(), CacheAccessMeta::new(tick));
        }
    }

    fn record_access(&self, key: &CacheKey) -> u64 {
        let tick = self.next_access_tick();
        let shard = self.access_shard(key);

        if let Some(meta) = shard.get(key) {
            meta.record_access(tick)
        } else {
            let meta = CacheAccessMeta::new(tick);
            let count = meta.record_access(tick);
            shard.insert(key.clone(), meta);
            count
        }
    }

    fn remove_access_meta(&self, key: &CacheKey) {
        self.access_shard(key).remove(key);
    }

    fn eviction_candidates(&self) -> Vec<EvictionCandidate> {
        let target = self.cache.len().min(EVICTION_SAMPLE_SIZE);
        let mut candidates = Vec::with_capacity(target);
        if target == 0 {
            return candidates;
        }

        let shard_count = self.access_meta.len();
        let start = self.eviction_cursor.fetch_add(1, Ordering::Relaxed) % shard_count;

        for offset in 0..shard_count {
            if candidates.len() >= target {
                break;
            }

            let shard = &self.access_meta[(start + offset) % shard_count];
            let remaining = target - candidates.len();
            let shard_budget = remaining.min(EVICTION_SAMPLE_SIZE);
            let mut stale_keys = Vec::new();

            for meta_ref in shard.iter().take(shard_budget) {
                let key = meta_ref.key().clone();
                if let Some(entry) = self.cache.get(&key) {
                    let (last_access_tick, access_count) = meta_ref.value().snapshot();
                    candidates.push(EvictionCandidate {
                        key,
                        last_access_tick,
                        access_count,
                        created_at: entry.created_at,
                        remaining_ttl: entry.remaining_ttl(),
                    });
                } else {
                    stale_keys.push(key);
                }
            }

            for key in stale_keys {
                shard.remove(&key);
            }
        }

        candidates
    }

    /// Evict one entry based on the eviction policy
    async fn evict_one(&self) {
        match self.config.eviction_policy {
            EvictionPolicy::LRU => self.evict_lru().await,
            EvictionPolicy::LFU => self.evict_lfu().await,
            EvictionPolicy::TTL => self.evict_ttl().await,
            EvictionPolicy::FIFO => self.evict_fifo().await,
        }
    }

    /// Evict the least recently used entry
    async fn evict_lru(&self) {
        let candidate = self
            .eviction_candidates()
            .into_iter()
            .min_by_key(|candidate| candidate.last_access_tick);

        if let Some(candidate) = candidate {
            self.evict_candidate(candidate, "LRU");
        }
    }

    /// Evict the least frequently used entry
    async fn evict_lfu(&self) {
        let candidate = self
            .eviction_candidates()
            .into_iter()
            .min_by_key(|candidate| (candidate.access_count, candidate.last_access_tick));

        if let Some(candidate) = candidate {
            self.evict_candidate(candidate, "LFU");
        }
    }

    /// Evict entry with shortest remaining TTL
    async fn evict_ttl(&self) {
        let candidate = self
            .eviction_candidates()
            .into_iter()
            .min_by_key(|candidate| candidate.remaining_ttl.unwrap_or(Duration::ZERO));

        if let Some(candidate) = candidate {
            self.evict_candidate(candidate, "TTL");
        }
    }

    /// Evict the oldest entry (FIFO)
    async fn evict_fifo(&self) {
        let candidate = self
            .eviction_candidates()
            .into_iter()
            .min_by_key(|candidate| candidate.created_at);

        if let Some(candidate) = candidate {
            self.evict_candidate(candidate, "FIFO");
        }
    }

    fn evict_candidate(&self, candidate: EvictionCandidate, policy: &'static str) {
        if let Some((_, removed)) = self.cache.remove(&candidate.key) {
            self.stats.sub_total_size(removed.size_bytes);
            self.stats.record_eviction();
            self.stats.set_entry_count(self.cache.len());
            trace!(key = %candidate.key, policy = policy, "Cache eviction");
        }
        self.remove_access_meta(&candidate.key);
    }

    /// Clean up expired entries
    async fn cleanup_expired(&self) {
        let mut expired_keys = Vec::new();

        for entry in self.cache.iter() {
            if entry.value().is_expired() {
                expired_keys.push(entry.key().clone());
            }
        }

        let count = expired_keys.len();
        for key in expired_keys {
            if let Some((_, removed)) = self.cache.remove(&key) {
                self.stats.sub_total_size(removed.size_bytes);
            }
            self.remove_access_meta(&key);
            self.stats.record_eviction();
        }

        if count > 0 {
            debug!(count = count, "Cleaned up expired entries");
            self.stats.set_entry_count(self.cache.len());
        }
    }
}

fn default_access_shard_count() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(MIN_ACCESS_SHARDS)
        .next_power_of_two()
        .clamp(MIN_ACCESS_SHARDS, MAX_ACCESS_SHARDS)
}

impl<T> Drop for InMemoryCache<T> {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.shutdown_notify.notify_waiters();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Basic Operations Tests ====================

    #[test]
    fn test_cache_new() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        let key = CacheKey::new("test-key");

        cache.set(key.clone(), "test-value".to_string()).await;

        let result = cache.get(&key).await;
        assert_eq!(result, Some("test-value".to_string()));
    }

    #[tokio::test]
    async fn test_cache_get_nonexistent() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        let key = CacheKey::new("nonexistent");

        let result = cache.get(&key).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_delete() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        let key = CacheKey::new("to-delete");

        cache.set(key.clone(), "value".to_string()).await;
        assert!(cache.exists(&key).await);

        let deleted = cache.delete(&key).await;
        assert!(deleted);
        assert!(!cache.exists(&key).await);
    }

    #[tokio::test]
    async fn test_cache_delete_nonexistent() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        let key = CacheKey::new("nonexistent");

        let deleted = cache.delete(&key).await;
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_cache_exists() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        let key = CacheKey::new("exists-key");

        assert!(!cache.exists(&key).await);
        cache.set(key.clone(), "value".to_string()).await;
        assert!(cache.exists(&key).await);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();

        cache.set(CacheKey::new("key1"), "value1".to_string()).await;
        cache.set(CacheKey::new("key2"), "value2".to_string()).await;
        cache.set(CacheKey::new("key3"), "value3".to_string()).await;

        assert_eq!(cache.len(), 3);
        cache.clear().await;
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    // ==================== TTL Tests ====================

    #[tokio::test]
    async fn test_cache_ttl_expiration() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        let key = CacheKey::new("expiring-key");

        cache
            .set_with_ttl(key.clone(), "value".to_string(), Duration::from_millis(10))
            .await;
        assert!(cache.exists(&key).await);

        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(!cache.exists(&key).await);
    }

    #[tokio::test]
    async fn test_cache_get_expired() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        let key = CacheKey::new("expiring-key");

        cache
            .set_with_ttl(key.clone(), "value".to_string(), Duration::from_millis(10))
            .await;
        tokio::time::sleep(Duration::from_millis(20)).await;

        let result = cache.get(&key).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_ttl_remaining() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        let key = CacheKey::new("ttl-key");

        cache
            .set_with_ttl(key.clone(), "value".to_string(), Duration::from_secs(60))
            .await;

        let ttl = cache.ttl(&key);
        assert!(ttl.is_some());
        assert!(ttl.unwrap() <= Duration::from_secs(60));
    }

    // ==================== Eviction Tests ====================

    #[tokio::test]
    async fn test_cache_lru_eviction() {
        let config = DualCacheConfig::default()
            .with_max_size(3)
            .with_eviction_policy(EvictionPolicy::LRU);
        let cache: InMemoryCache<String> = InMemoryCache::new(config);

        cache.set(CacheKey::new("key1"), "value1".to_string()).await;
        cache.set(CacheKey::new("key2"), "value2".to_string()).await;
        cache.set(CacheKey::new("key3"), "value3".to_string()).await;

        // Access key1 and key2 to make them more recent
        cache.get(&CacheKey::new("key1")).await;
        cache.get(&CacheKey::new("key2")).await;

        // Add key4, should evict key3 (least recently used)
        cache.set(CacheKey::new("key4"), "value4".to_string()).await;

        assert!(cache.exists(&CacheKey::new("key1")).await);
        assert!(cache.exists(&CacheKey::new("key2")).await);
        assert!(!cache.exists(&CacheKey::new("key3")).await);
        assert!(cache.exists(&CacheKey::new("key4")).await);
    }

    #[tokio::test]
    async fn test_cache_lfu_eviction() {
        let config = DualCacheConfig::default()
            .with_max_size(3)
            .with_eviction_policy(EvictionPolicy::LFU);
        let cache: InMemoryCache<String> = InMemoryCache::new(config);

        cache.set(CacheKey::new("key1"), "value1".to_string()).await;
        cache.set(CacheKey::new("key2"), "value2".to_string()).await;
        cache.set(CacheKey::new("key3"), "value3".to_string()).await;

        // Access key1 multiple times
        for _ in 0..5 {
            cache.get(&CacheKey::new("key1")).await;
        }
        // Access key2 a few times
        for _ in 0..2 {
            cache.get(&CacheKey::new("key2")).await;
        }
        // key3 has lowest access count

        // Add key4, should evict key3 (least frequently used)
        cache.set(CacheKey::new("key4"), "value4".to_string()).await;

        assert!(cache.exists(&CacheKey::new("key1")).await);
        assert!(cache.exists(&CacheKey::new("key2")).await);
        assert!(!cache.exists(&CacheKey::new("key3")).await);
        // key3 should be evicted
        assert!(cache.exists(&CacheKey::new("key4")).await);
    }

    #[tokio::test]
    async fn test_cache_ttl_eviction_prefers_expired_entry() {
        let config = DualCacheConfig::default()
            .with_max_size(3)
            .with_eviction_policy(EvictionPolicy::TTL);
        let cache: InMemoryCache<String> = InMemoryCache::new(config);

        cache
            .set_with_ttl(
                CacheKey::new("short"),
                "value1".to_string(),
                Duration::from_millis(10),
            )
            .await;
        cache
            .set_with_ttl(
                CacheKey::new("long1"),
                "value2".to_string(),
                Duration::from_secs(60),
            )
            .await;
        cache
            .set_with_ttl(
                CacheKey::new("long2"),
                "value3".to_string(),
                Duration::from_secs(60),
            )
            .await;

        tokio::time::sleep(Duration::from_millis(20)).await;
        cache.set(CacheKey::new("new"), "value4".to_string()).await;

        assert!(!cache.exists(&CacheKey::new("short")).await);
        assert!(cache.exists(&CacheKey::new("long1")).await);
        assert!(cache.exists(&CacheKey::new("long2")).await);
        assert!(cache.exists(&CacheKey::new("new")).await);
    }

    // ==================== Statistics Tests ====================

    #[tokio::test]
    async fn test_cache_stats_hits_misses() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        let key = CacheKey::new("stats-key");

        cache.set(key.clone(), "value".to_string()).await;

        // Generate hits
        cache.get(&key).await;
        cache.get(&key).await;

        // Generate misses
        cache.get(&CacheKey::new("nonexistent1")).await;
        cache.get(&CacheKey::new("nonexistent2")).await;

        let stats = cache.stats().snapshot();
        assert_eq!(stats.memory_hits, 2);
        assert_eq!(stats.memory_misses, 2);
    }

    #[tokio::test]
    async fn test_cache_stats_writes() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();

        cache.set(CacheKey::new("key1"), "value1".to_string()).await;
        cache.set(CacheKey::new("key2"), "value2".to_string()).await;

        let stats = cache.stats().snapshot();
        assert_eq!(stats.writes, 2);
    }

    #[tokio::test]
    async fn test_cache_stats_deletions() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        let key = CacheKey::new("to-delete");

        cache.set(key.clone(), "value".to_string()).await;
        cache.delete(&key).await;

        let stats = cache.stats().snapshot();
        assert_eq!(stats.deletions, 1);
    }

    // ==================== Concurrent Access Tests ====================

    #[tokio::test]
    async fn test_cache_concurrent_read_write() {
        use std::sync::Arc;

        let cache = Arc::new(InMemoryCache::<i32>::with_defaults());
        let mut handles = vec![];

        // Writers
        for i in 0..4 {
            let cache_clone = Arc::clone(&cache);
            let handle = tokio::spawn(async move {
                for j in 0..25 {
                    let key = CacheKey::new(format!("key-{}-{}", i, j));
                    cache_clone.set(key, i * 25 + j).await;
                }
            });
            handles.push(handle);
        }

        // Readers
        for _ in 0..4 {
            let cache_clone = Arc::clone(&cache);
            let handle = tokio::spawn(async move {
                for i in 0..4 {
                    for j in 0..25 {
                        let key = CacheKey::new(format!("key-{}-{}", i, j));
                        let _ = cache_clone.get(&key).await;
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Just verify no panics occurred
        assert!(cache.len() <= 100);
    }

    // ==================== Entry Metadata Tests ====================

    #[tokio::test]
    async fn test_cache_get_entry() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        let key = CacheKey::new("entry-key");

        cache
            .set_with_size(
                key.clone(),
                "value".to_string(),
                Duration::from_secs(60),
                100,
            )
            .await;

        let entry = cache.get_entry(&key).await;
        assert!(entry.is_some());

        let entry = entry.unwrap();
        assert_eq!(entry.value, "value");
        assert_eq!(entry.size_bytes, 100);
        assert_eq!(entry.access_count, 1); // One access from get_entry
    }

    #[tokio::test]
    async fn test_cache_get_entry_includes_prior_get_accesses() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        let key = CacheKey::new("entry-access-key");

        cache.set(key.clone(), "value".to_string()).await;
        cache.get(&key).await;
        cache.get(&key).await;

        let entry = cache.get_entry(&key).await.unwrap();
        assert_eq!(entry.access_count, 3);
    }

    #[tokio::test]
    async fn test_cache_keys() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();

        cache.set(CacheKey::new("key1"), "value1".to_string()).await;
        cache.set(CacheKey::new("key2"), "value2".to_string()).await;
        cache.set(CacheKey::new("key3"), "value3".to_string()).await;

        let keys = cache.keys();
        assert_eq!(keys.len(), 3);
    }

    // ==================== Update Tests ====================

    #[tokio::test]
    async fn test_cache_update_existing() {
        let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
        let key = CacheKey::new("update-key");

        cache.set(key.clone(), "initial".to_string()).await;
        cache.set(key.clone(), "updated".to_string()).await;

        let result = cache.get(&key).await;
        assert_eq!(result, Some("updated".to_string()));
        assert_eq!(cache.len(), 1);
    }
}
