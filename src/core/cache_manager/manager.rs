//! Legacy cache manager implementation.
//!
//! This module is retained for compatibility. Prefer `core::cache` for key-based
//! caching and `core::semantic_cache` for similarity-based caching.

use super::types::{
    AtomicCacheStats, CacheConfig, CacheEntry, CacheKey, CacheStats, SemanticCacheMap,
};
use crate::core::models::openai::ChatCompletionResponse;
use crate::utils::error::error::Result;
use dashmap::DashMap;
use lru::LruCache;
use parking_lot::RwLock;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::{debug, info};

/// Multi-tier cache manager
pub struct CacheManager {
    /// L1 cache: In-memory LRU cache for hot data
    l1_cache: Arc<RwLock<LruCache<CacheKey, CacheEntry<ChatCompletionResponse>>>>,
    /// L2 cache: Larger capacity with TTL
    l2_cache: Arc<DashMap<CacheKey, CacheEntry<ChatCompletionResponse>>>,
    /// Semantic cache for similar queries
    semantic_cache: Arc<RwLock<SemanticCacheMap>>,
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics (lock-free atomics for hot path)
    stats: Arc<AtomicCacheStats>,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(config: CacheConfig) -> Result<Self> {
        // Ensure we have a reasonable minimum capacity
        let l1_capacity = NonZeroUsize::new(config.max_entries / 10)
            .or_else(|| NonZeroUsize::new(100))
            .ok_or_else(|| {
                crate::utils::error::error::GatewayError::Config(
                    "Invalid cache configuration: max_entries must be greater than 0".to_string(),
                )
            })?;

        Ok(Self {
            l1_cache: Arc::new(RwLock::new(LruCache::new(l1_capacity))),
            l2_cache: Arc::new(DashMap::new()),
            semantic_cache: Arc::new(RwLock::new(SemanticCacheMap::default())),
            config,
            stats: Arc::new(AtomicCacheStats::default()),
        })
    }

    /// Get a cached response
    pub async fn get(&self, key: &CacheKey) -> Result<Option<ChatCompletionResponse>> {
        // Try L1 cache first
        {
            let mut l1 = self.l1_cache.write();
            if let Some(entry) = l1.get_mut(key) {
                if !entry.is_expired() {
                    entry.mark_accessed();
                    self.stats.l1_hits.fetch_add(1, Ordering::Relaxed);
                    debug!("L1 cache hit for key: {:?}", key);
                    return Ok(Some(entry.value.clone()));
                } else {
                    l1.pop(key);
                }
            }
        }

        self.stats.l1_misses.fetch_add(1, Ordering::Relaxed);

        // Try L2 cache
        if let Some(mut entry) = self.l2_cache.get_mut(key) {
            if !entry.is_expired() {
                entry.mark_accessed();

                // Promote to L1 cache
                let mut l1 = self.l1_cache.write();
                l1.put(key.clone(), entry.clone());

                self.stats.l2_hits.fetch_add(1, Ordering::Relaxed);
                debug!("L2 cache hit for key: {:?}", key);
                return Ok(Some(entry.value.clone()));
            } else {
                self.l2_cache.remove(key);
            }
        }

        self.stats.l2_misses.fetch_add(1, Ordering::Relaxed);

        // Try semantic cache if enabled
        if self.config.enable_semantic {
            if let Some(response) = self.semantic_lookup(key).await? {
                self.stats.semantic_hits.fetch_add(1, Ordering::Relaxed);
                debug!("Semantic cache hit for key: {:?}", key);
                return Ok(Some(response));
            }
        }

        self.stats.semantic_misses.fetch_add(1, Ordering::Relaxed);
        Ok(None)
    }

    /// Store a response in the cache
    pub async fn put(&self, key: CacheKey, response: ChatCompletionResponse) -> Result<()> {
        let size_bytes = self.estimate_size(&response);
        let entry = CacheEntry::new(response, self.config.default_ttl, size_bytes);

        // Store in L2 cache
        self.l2_cache.insert(key.clone(), entry);

        // Update semantic cache if enabled
        if self.config.enable_semantic {
            self.update_semantic_cache(&key).await?;
        }

        // Update statistics (lock-free)
        self.stats
            .total_size_bytes
            .fetch_add(size_bytes, Ordering::Relaxed);

        // Cleanup expired entries periodically
        if self.l2_cache.len().is_multiple_of(1000) {
            self.cleanup_expired().await;
        }

        debug!("Cached response for key: {:?}", key);
        Ok(())
    }

    /// Semantic cache lookup
    async fn semantic_lookup(&self, _key: &CacheKey) -> Result<Option<ChatCompletionResponse>> {
        // TODO: Implement semantic similarity search
        // This would involve:
        // 1. Extract embeddings from the request
        // 2. Compare with cached embeddings
        // 3. Return similar cached responses if similarity > threshold
        Ok(None)
    }

    /// Update semantic cache
    async fn update_semantic_cache(&self, _key: &CacheKey) -> Result<()> {
        // TODO: Implement semantic cache updates
        // This would involve:
        // 1. Generate embeddings for the request
        // 2. Store in semantic index
        Ok(())
    }

    /// Estimate the size of a response in bytes
    fn estimate_size(&self, response: &ChatCompletionResponse) -> usize {
        // Rough estimation based on JSON serialization
        serde_json::to_string(response)
            .map(|s| s.len())
            .unwrap_or(1024) // Default estimate
    }

    /// Clean up expired entries
    async fn cleanup_expired(&self) {
        let mut removed_count = 0u64;
        let mut removed_size = 0usize;

        // Clean L2 cache
        self.l2_cache.retain(|_, entry| {
            if entry.is_expired() {
                removed_count += 1;
                removed_size += entry.size_bytes;
                false
            } else {
                true
            }
        });

        // Update statistics (lock-free)
        if removed_count > 0 {
            self.stats
                .evictions
                .fetch_add(removed_count, Ordering::Relaxed);
            // Use saturating subtraction for size
            let current_size = self.stats.total_size_bytes.load(Ordering::Relaxed);
            self.stats
                .total_size_bytes
                .store(current_size.saturating_sub(removed_size), Ordering::Relaxed);

            info!(
                "Cleaned up {} expired cache entries, freed {} bytes",
                removed_count, removed_size
            );
        }
    }

    /// Get cache statistics (lock-free snapshot)
    pub fn stats(&self) -> CacheStats {
        self.stats.snapshot()
    }

    /// Clear all caches
    pub async fn clear(&self) {
        self.l1_cache.write().clear();
        self.l2_cache.clear();
        self.semantic_cache.write().clear();

        // Reset atomic stats
        self.stats.reset();

        info!("All caches cleared");
    }
}
