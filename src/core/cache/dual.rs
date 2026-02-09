//! Dual cache implementation
//!
//! This module provides a two-tier cache system combining in-memory and Redis caches
//! for optimal performance and distributed consistency.

use super::memory::InMemoryCache;
use super::redis_cache::RedisCache;
use super::types::{
    AtomicCacheStats, CacheEntry, CacheKey, CacheMode, CacheStatsSnapshot, DualCacheConfig,
};
use crate::storage::redis::RedisPool;
use crate::utils::error::error::Result;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, trace, warn};

/// Dual-layer cache combining in-memory and Redis caches
///
/// Read strategy:
/// 1. Check memory cache first (sub-millisecond)
/// 2. On memory miss, check Redis cache
/// 3. On Redis hit, populate memory cache for future reads
///
/// Write strategy:
/// - Write to both memory and Redis caches
///
/// Invalidation strategy:
/// - Invalidate both caches
pub struct DualCache<T> {
    /// In-memory cache layer (L1)
    memory: Arc<InMemoryCache<T>>,
    /// Redis cache layer (L2)
    redis: Option<RedisCache<T>>,
    /// Configuration
    config: DualCacheConfig,
    /// Shared statistics
    stats: Arc<AtomicCacheStats>,
}

impl<T> DualCache<T>
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    /// Create a new dual cache with the given configuration
    pub fn new(config: DualCacheConfig, redis_pool: Option<Arc<RedisPool>>) -> Self {
        let stats = Arc::new(AtomicCacheStats::new());
        let memory = Arc::new(InMemoryCache::with_stats(
            config.clone(),
            Arc::clone(&stats),
        ));

        let redis = match (&config.mode, redis_pool) {
            (CacheMode::MemoryOnly, _) => None,
            (_, Some(pool)) => Some(RedisCache::with_stats(
                pool,
                config.clone(),
                Arc::clone(&stats),
            )),
            (CacheMode::RedisOnly, None) => {
                warn!(
                    "Redis-only mode requested but no Redis pool provided, falling back to memory-only"
                );
                None
            }
            (CacheMode::Dual, None) => {
                debug!("No Redis pool provided, using memory-only cache");
                None
            }
        };

        Self {
            memory,
            redis,
            config,
            stats,
        }
    }

    /// Create a memory-only cache
    pub fn memory_only(config: DualCacheConfig) -> Self {
        let mut config = config;
        config.mode = CacheMode::MemoryOnly;
        Self::new(config, None)
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(DualCacheConfig::default(), None)
    }

    /// Start the background cleanup task for the memory cache
    pub fn start_cleanup_task(&self) {
        self.memory.start_cleanup_task();
    }

    /// Get a value from the cache
    ///
    /// Checks memory cache first, then Redis. On Redis hit,
    /// populates memory cache for future reads.
    pub async fn get(&self, key: &CacheKey) -> Result<Option<T>> {
        match self.config.mode {
            CacheMode::MemoryOnly => Ok(self.memory.get(key)),
            CacheMode::RedisOnly => {
                if let Some(ref redis) = self.redis {
                    redis.get(key).await
                } else {
                    Ok(None)
                }
            }
            CacheMode::Dual => self.get_dual(key).await,
        }
    }

    /// Get from dual cache with read-through pattern
    async fn get_dual(&self, key: &CacheKey) -> Result<Option<T>> {
        // L1: Check memory cache first (fastest path)
        if let Some(value) = self.memory.get(key) {
            trace!(key = %key, "Dual cache L1 hit");
            return Ok(Some(value));
        }

        // L2: Check Redis cache
        if let Some(ref redis) = self.redis {
            if let Some(value) = redis.get(key).await? {
                // Populate memory cache with the value from Redis
                self.memory.set(key.clone(), value.clone());
                trace!(key = %key, "Dual cache L2 hit, populated L1");
                return Ok(Some(value));
            }
        }

        trace!(key = %key, "Dual cache miss");
        Ok(None)
    }

    /// Get an entry with metadata from the cache
    pub async fn get_entry(&self, key: &CacheKey) -> Result<Option<CacheEntry<T>>> {
        match self.config.mode {
            CacheMode::MemoryOnly => Ok(self.memory.get_entry(key)),
            CacheMode::RedisOnly => {
                if let Some(ref redis) = self.redis {
                    redis.get_entry(key).await
                } else {
                    Ok(None)
                }
            }
            CacheMode::Dual => {
                // Check memory first
                if let Some(entry) = self.memory.get_entry(key) {
                    return Ok(Some(entry));
                }

                // Check Redis
                if let Some(ref redis) = self.redis {
                    if let Some(entry) = redis.get_entry(key).await? {
                        // Populate memory cache
                        self.memory.set_with_size(
                            key.clone(),
                            entry.value.clone(),
                            entry.ttl,
                            entry.size_bytes,
                        );
                        return Ok(Some(entry));
                    }
                }

                Ok(None)
            }
        }
    }

    /// Set a value in the cache with the default TTL
    pub async fn set(&self, key: CacheKey, value: T) -> Result<()> {
        self.set_with_ttl(key, value, self.config.default_ttl).await
    }

    /// Set a value in the cache with a specific TTL
    pub async fn set_with_ttl(&self, key: CacheKey, value: T, ttl: Duration) -> Result<()> {
        match self.config.mode {
            CacheMode::MemoryOnly => {
                self.memory.set_with_ttl(key, value, ttl);
                Ok(())
            }
            CacheMode::RedisOnly => {
                if let Some(ref redis) = self.redis {
                    redis.set_with_ttl(key, value, ttl).await
                } else {
                    Ok(())
                }
            }
            CacheMode::Dual => self.set_dual(key, value, ttl).await,
        }
    }

    /// Set in both cache layers
    async fn set_dual(&self, key: CacheKey, value: T, ttl: Duration) -> Result<()> {
        // Write to memory cache (synchronous, fast)
        self.memory.set_with_ttl(key.clone(), value.clone(), ttl);

        // Write to Redis cache (asynchronous)
        if let Some(ref redis) = self.redis {
            if let Err(e) = redis.set_with_ttl(key.clone(), value, ttl).await {
                warn!(key = %key, error = %e, "Failed to write to Redis cache");
                // Don't fail the operation if Redis write fails
            }
        }

        trace!(key = %key, ttl_secs = ttl.as_secs(), "Dual cache set");
        Ok(())
    }

    /// Set a value with size tracking
    pub async fn set_with_size(
        &self,
        key: CacheKey,
        value: T,
        ttl: Duration,
        size_bytes: usize,
    ) -> Result<()> {
        match self.config.mode {
            CacheMode::MemoryOnly => {
                self.memory.set_with_size(key, value, ttl, size_bytes);
                Ok(())
            }
            CacheMode::RedisOnly => {
                if let Some(ref redis) = self.redis {
                    redis.set_with_size(key, value, ttl, size_bytes).await
                } else {
                    Ok(())
                }
            }
            CacheMode::Dual => {
                self.memory
                    .set_with_size(key.clone(), value.clone(), ttl, size_bytes);
                if let Some(ref redis) = self.redis {
                    let _ = redis.set_with_size(key, value, ttl, size_bytes).await;
                }
                Ok(())
            }
        }
    }

    /// Delete a value from both cache layers
    pub async fn delete(&self, key: &CacheKey) -> Result<bool> {
        let mut deleted = false;

        match self.config.mode {
            CacheMode::MemoryOnly => {
                deleted = self.memory.delete(key);
            }
            CacheMode::RedisOnly => {
                if let Some(ref redis) = self.redis {
                    deleted = redis.delete(key).await?;
                }
            }
            CacheMode::Dual => {
                // Delete from memory
                if self.memory.delete(key) {
                    deleted = true;
                }

                // Delete from Redis
                if let Some(ref redis) = self.redis {
                    if redis.delete(key).await.unwrap_or(false) {
                        deleted = true;
                    }
                }
            }
        }

        trace!(key = %key, deleted = deleted, "Dual cache delete");
        Ok(deleted)
    }

    /// Check if a key exists in either cache layer
    pub async fn exists(&self, key: &CacheKey) -> Result<bool> {
        match self.config.mode {
            CacheMode::MemoryOnly => Ok(self.memory.exists(key)),
            CacheMode::RedisOnly => {
                if let Some(ref redis) = self.redis {
                    redis.exists(key).await
                } else {
                    Ok(false)
                }
            }
            CacheMode::Dual => {
                // Check memory first
                if self.memory.exists(key) {
                    return Ok(true);
                }

                // Check Redis
                if let Some(ref redis) = self.redis {
                    return redis.exists(key).await;
                }

                Ok(false)
            }
        }
    }

    /// Get the remaining TTL for a key
    pub async fn ttl(&self, key: &CacheKey) -> Result<Option<Duration>> {
        match self.config.mode {
            CacheMode::MemoryOnly => Ok(self.memory.ttl(key)),
            CacheMode::RedisOnly => {
                if let Some(ref redis) = self.redis {
                    redis.ttl(key).await
                } else {
                    Ok(None)
                }
            }
            CacheMode::Dual => {
                // Prefer memory TTL
                if let Some(ttl) = self.memory.ttl(key) {
                    return Ok(Some(ttl));
                }

                // Fall back to Redis
                if let Some(ref redis) = self.redis {
                    return redis.ttl(key).await;
                }

                Ok(None)
            }
        }
    }

    /// Clear all entries from both cache layers
    pub async fn clear(&self) -> Result<()> {
        // Clear memory cache
        self.memory.clear();

        // Note: We don't clear Redis cache as it may be shared across instances
        // Use delete_by_prefix for Redis if needed

        debug!("Dual cache cleared (memory only)");
        Ok(())
    }

    /// Get the number of entries in the memory cache
    pub fn memory_len(&self) -> usize {
        self.memory.len()
    }

    /// Check if the memory cache is empty
    pub fn is_memory_empty(&self) -> bool {
        self.memory.is_empty()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStatsSnapshot {
        self.stats.snapshot()
    }

    /// Get the raw statistics for atomic updates
    pub fn atomic_stats(&self) -> Arc<AtomicCacheStats> {
        Arc::clone(&self.stats)
    }

    /// Check if Redis is available
    pub async fn is_redis_available(&self) -> bool {
        if let Some(ref redis) = self.redis {
            redis.is_available().await
        } else {
            false
        }
    }

    /// Get the current cache mode
    pub fn mode(&self) -> CacheMode {
        self.config.mode
    }

    /// Get the configuration
    pub fn config(&self) -> &DualCacheConfig {
        &self.config
    }

    /// Shutdown the cache
    pub fn shutdown(&self) {
        self.memory.shutdown();
    }
}

/// Batch operations for dual cache
impl<T> DualCache<T>
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    /// Get multiple values from the cache
    pub async fn get_many(&self, keys: &[CacheKey]) -> Result<Vec<Option<T>>> {
        let mut results = Vec::with_capacity(keys.len());

        for key in keys {
            results.push(self.get(key).await?);
        }

        Ok(results)
    }

    /// Set multiple values in the cache
    pub async fn set_many(&self, entries: &[(CacheKey, T, Duration)]) -> Result<()> {
        for (key, value, ttl) in entries {
            self.set_with_ttl(key.clone(), value.clone(), *ttl).await?;
        }
        Ok(())
    }

    /// Delete multiple keys from the cache
    pub async fn delete_many(&self, keys: &[CacheKey]) -> Result<usize> {
        let mut deleted = 0;
        for key in keys {
            if self.delete(key).await? {
                deleted += 1;
            }
        }
        Ok(deleted)
    }
}

/// Cache warming operations
impl<T> DualCache<T>
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    /// Warm the memory cache from Redis
    ///
    /// Loads entries from Redis into memory for specified keys
    pub async fn warm_from_redis(&self, keys: &[CacheKey]) -> Result<usize> {
        if self.redis.is_none() || self.config.mode == CacheMode::MemoryOnly {
            return Ok(0);
        }

        let Some(redis) = self.redis.as_ref() else {
            return Ok(0);
        };
        let mut warmed = 0;

        for key in keys {
            // Skip if already in memory
            if self.memory.exists(key) {
                continue;
            }

            // Try to load from Redis
            if let Ok(Some(entry)) = redis.get_entry(key).await {
                self.memory
                    .set_with_size(key.clone(), entry.value, entry.ttl, entry.size_bytes);
                warmed += 1;
            }
        }

        debug!(count = warmed, "Warmed memory cache from Redis");
        Ok(warmed)
    }

    /// Warm the memory cache with provided entries
    pub fn warm_with_entries(&self, entries: &[(CacheKey, T, Duration)]) -> usize {
        let mut warmed = 0;

        for (key, value, ttl) in entries {
            if !self.memory.exists(key) {
                self.memory.set_with_ttl(key.clone(), value.clone(), *ttl);
                warmed += 1;
            }
        }

        debug!(count = warmed, "Warmed memory cache with entries");
        warmed
    }
}


#[cfg(test)]
#[path = "dual_tests.rs"]
mod tests;
