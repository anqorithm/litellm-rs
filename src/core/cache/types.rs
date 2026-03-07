//! Cache type definitions
//!
//! This module contains core type definitions for the dual cache system,
//! including cache keys, entries, configuration, and eviction policies.

use crate::core::types::config::defaults::default_true;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Cache key with hash generation for efficient lookups
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// The unique key string
    key: String,
    /// Pre-computed hash for fast lookups
    hash: u64,
}

impl CacheKey {
    /// Create a new cache key from a string
    pub fn new(key: impl Into<String>) -> Self {
        let key = key.into();
        let hash = Self::compute_hash(&key);
        Self { key, hash }
    }

    /// Create a cache key from parts
    pub fn from_parts(prefix: &str, parts: &[&str]) -> Self {
        let key = std::iter::once(prefix)
            .chain(parts.iter().copied())
            .collect::<Vec<_>>()
            .join(":");
        Self::new(key)
    }

    /// Compute the hash of a key
    fn compute_hash(key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// Get the key string
    pub fn as_str(&self) -> &str {
        &self.key
    }

    /// Get the pre-computed hash
    pub fn hash_value(&self) -> u64 {
        self.hash
    }

    /// Convert to Redis-compatible key string
    pub fn to_redis_key(&self) -> String {
        format!("litellm:cache:{}", self.key)
    }
}

impl std::fmt::Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}

impl From<String> for CacheKey {
    fn from(key: String) -> Self {
        Self::new(key)
    }
}

impl From<&str> for CacheKey {
    fn from(key: &str) -> Self {
        Self::new(key)
    }
}

/// Cache entry with value, TTL, and metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// The cached value
    pub value: T,
    /// Time-to-live duration
    pub ttl: Duration,
    /// When the entry was created
    pub created_at: Instant,
    /// Absolute expiration time (for Redis compatibility)
    pub expires_at_unix: u64,
    /// Access count for LFU eviction
    pub access_count: u64,
    /// Last access time
    pub last_accessed: Instant,
    /// Estimated size in bytes
    pub size_bytes: usize,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry
    pub fn new(value: T, ttl: Duration) -> Self {
        let now = Instant::now();
        let expires_at_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + ttl.as_secs();

        Self {
            value,
            ttl,
            created_at: now,
            expires_at_unix,
            access_count: 0,
            last_accessed: now,
            size_bytes: 0,
        }
    }

    /// Create a new cache entry with estimated size
    pub fn with_size(value: T, ttl: Duration, size_bytes: usize) -> Self {
        let mut entry = Self::new(value, ttl);
        entry.size_bytes = size_bytes;
        entry
    }

    /// Check if the entry has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    /// Get remaining TTL
    pub fn remaining_ttl(&self) -> Option<Duration> {
        if self.is_expired() {
            None
        } else {
            Some(self.ttl.saturating_sub(self.created_at.elapsed()))
        }
    }

    /// Mark the entry as accessed (for LRU/LFU tracking)
    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = Instant::now();
    }

    /// Get the age of the entry
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// Serializable cache entry for Redis storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableCacheEntry<T> {
    /// The cached value
    pub value: T,
    /// TTL in seconds
    pub ttl_secs: u64,
    /// Unix timestamp when created
    pub created_at_unix: u64,
    /// Unix timestamp when expires
    pub expires_at_unix: u64,
    /// Access count
    pub access_count: u64,
    /// Size in bytes
    pub size_bytes: usize,
}

impl<T: Clone> From<&CacheEntry<T>> for SerializableCacheEntry<T> {
    fn from(entry: &CacheEntry<T>) -> Self {
        let created_at_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(entry.created_at.elapsed().as_secs());

        Self {
            value: entry.value.clone(),
            ttl_secs: entry.ttl.as_secs(),
            created_at_unix,
            expires_at_unix: entry.expires_at_unix,
            access_count: entry.access_count,
            size_bytes: entry.size_bytes,
        }
    }
}

impl<T> SerializableCacheEntry<T> {
    /// Convert back to a CacheEntry
    pub fn into_cache_entry(self) -> CacheEntry<T> {
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let age_secs = now_unix.saturating_sub(self.created_at_unix);
        let _remaining_ttl = self.ttl_secs.saturating_sub(age_secs);

        CacheEntry {
            value: self.value,
            ttl: Duration::from_secs(self.ttl_secs),
            created_at: Instant::now() - Duration::from_secs(age_secs),
            expires_at_unix: self.expires_at_unix,
            access_count: self.access_count,
            last_accessed: Instant::now(),
            size_bytes: self.size_bytes,
        }
    }

    /// Check if expired based on Unix timestamp
    pub fn is_expired(&self) -> bool {
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now_unix >= self.expires_at_unix
    }
}

/// Eviction policy for cache management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EvictionPolicy {
    /// Least Recently Used - evicts entries that haven't been accessed recently
    #[default]
    LRU,
    /// Least Frequently Used - evicts entries with lowest access count
    LFU,
    /// Time-To-Live based - only evicts expired entries
    TTL,
    /// First In First Out - evicts oldest entries first
    FIFO,
}

impl std::fmt::Display for EvictionPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvictionPolicy::LRU => write!(f, "lru"),
            EvictionPolicy::LFU => write!(f, "lfu"),
            EvictionPolicy::TTL => write!(f, "ttl"),
            EvictionPolicy::FIFO => write!(f, "fifo"),
        }
    }
}

/// Cache mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CacheMode {
    /// Use only in-memory cache
    MemoryOnly,
    /// Use only Redis cache
    RedisOnly,
    /// Use both memory and Redis (default)
    #[default]
    Dual,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualCacheConfig {
    /// Maximum number of entries in memory cache
    #[serde(default = "default_max_size")]
    pub max_size: usize,
    /// Default TTL for cache entries
    #[serde(default = "default_ttl")]
    pub default_ttl: Duration,
    /// Eviction policy
    #[serde(default)]
    pub eviction_policy: EvictionPolicy,
    /// Cache mode
    #[serde(default)]
    pub mode: CacheMode,
    /// Enable cache statistics
    #[serde(default = "default_true")]
    pub enable_stats: bool,
    /// Background cleanup interval
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval: Duration,
    /// Redis key prefix
    #[serde(default = "default_key_prefix")]
    pub key_prefix: String,
    /// Enable compression for large values
    #[serde(default)]
    pub enable_compression: bool,
    /// Compression threshold in bytes
    #[serde(default = "default_compression_threshold")]
    pub compression_threshold: usize,
}

fn default_max_size() -> usize {
    10000
}

fn default_ttl() -> Duration {
    Duration::from_secs(3600) // 1 hour
}

fn default_cleanup_interval() -> Duration {
    Duration::from_secs(60) // 1 minute
}

fn default_key_prefix() -> String {
    "litellm:cache".to_string()
}

fn default_compression_threshold() -> usize {
    1024 // 1KB
}

impl Default for DualCacheConfig {
    fn default() -> Self {
        Self {
            max_size: default_max_size(),
            default_ttl: default_ttl(),
            eviction_policy: EvictionPolicy::default(),
            mode: CacheMode::default(),
            enable_stats: default_true(),
            cleanup_interval: default_cleanup_interval(),
            key_prefix: default_key_prefix(),
            enable_compression: false,
            compression_threshold: default_compression_threshold(),
        }
    }
}

impl DualCacheConfig {
    /// Create a memory-only configuration
    pub fn memory_only() -> Self {
        Self {
            mode: CacheMode::MemoryOnly,
            ..Default::default()
        }
    }

    /// Create a Redis-only configuration
    pub fn redis_only() -> Self {
        Self {
            mode: CacheMode::RedisOnly,
            ..Default::default()
        }
    }

    /// Set the maximum cache size
    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }

    /// Set the default TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// Set the eviction policy
    pub fn with_eviction_policy(mut self, policy: EvictionPolicy) -> Self {
        self.eviction_policy = policy;
        self
    }
}

/// Atomic cache statistics for lock-free updates
#[derive(Debug, Default)]
pub struct AtomicCacheStats {
    /// Memory cache hits
    pub memory_hits: AtomicU64,
    /// Memory cache misses
    pub memory_misses: AtomicU64,
    /// Redis cache hits
    pub redis_hits: AtomicU64,
    /// Redis cache misses
    pub redis_misses: AtomicU64,
    /// Cache writes
    pub writes: AtomicU64,
    /// Cache deletions
    pub deletions: AtomicU64,
    /// Cache evictions
    pub evictions: AtomicU64,
    /// Current entry count
    pub entry_count: AtomicUsize,
    /// Total size in bytes
    pub total_size_bytes: AtomicUsize,
}

impl AtomicCacheStats {
    /// Create a new stats instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a memory cache hit
    pub fn record_memory_hit(&self) {
        self.memory_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a memory cache miss
    pub fn record_memory_miss(&self) {
        self.memory_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a Redis cache hit
    pub fn record_redis_hit(&self) {
        self.redis_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a Redis cache miss
    pub fn record_redis_miss(&self) {
        self.redis_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache write
    pub fn record_write(&self) {
        self.writes.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache deletion
    pub fn record_deletion(&self) {
        self.deletions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an eviction
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Update entry count
    pub fn set_entry_count(&self, count: usize) {
        self.entry_count.store(count, Ordering::Relaxed);
    }

    /// Update total size
    pub fn set_total_size(&self, size: usize) {
        self.total_size_bytes.store(size, Ordering::Relaxed);
    }

    /// Atomically add to total size
    pub fn add_total_size(&self, size: usize) {
        self.total_size_bytes.fetch_add(size, Ordering::Relaxed);
    }

    /// Atomically subtract from total size (saturating)
    pub fn sub_total_size(&self, size: usize) {
        let _ =
            self.total_size_bytes
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                    Some(current.saturating_sub(size))
                });
    }

    /// Get a snapshot of current statistics
    pub fn snapshot(&self) -> CacheStatsSnapshot {
        CacheStatsSnapshot {
            memory_hits: self.memory_hits.load(Ordering::Relaxed),
            memory_misses: self.memory_misses.load(Ordering::Relaxed),
            redis_hits: self.redis_hits.load(Ordering::Relaxed),
            redis_misses: self.redis_misses.load(Ordering::Relaxed),
            writes: self.writes.load(Ordering::Relaxed),
            deletions: self.deletions.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            entry_count: self.entry_count.load(Ordering::Relaxed),
            total_size_bytes: self.total_size_bytes.load(Ordering::Relaxed),
        }
    }

    /// Reset all statistics
    pub fn reset(&self) {
        self.memory_hits.store(0, Ordering::Relaxed);
        self.memory_misses.store(0, Ordering::Relaxed);
        self.redis_hits.store(0, Ordering::Relaxed);
        self.redis_misses.store(0, Ordering::Relaxed);
        self.writes.store(0, Ordering::Relaxed);
        self.deletions.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.entry_count.store(0, Ordering::Relaxed);
        self.total_size_bytes.store(0, Ordering::Relaxed);
    }
}

/// Snapshot of cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStatsSnapshot {
    /// Memory cache hits
    pub memory_hits: u64,
    /// Memory cache misses
    pub memory_misses: u64,
    /// Redis cache hits
    pub redis_hits: u64,
    /// Redis cache misses
    pub redis_misses: u64,
    /// Cache writes
    pub writes: u64,
    /// Cache deletions
    pub deletions: u64,
    /// Cache evictions
    pub evictions: u64,
    /// Current entry count
    pub entry_count: usize,
    /// Total size in bytes
    pub total_size_bytes: usize,
}

impl CacheStatsSnapshot {
    /// Calculate total hit count
    pub fn total_hits(&self) -> u64 {
        self.memory_hits + self.redis_hits
    }

    /// Calculate total miss count
    pub fn total_misses(&self) -> u64 {
        self.memory_misses + self.redis_misses
    }

    /// Calculate total request count
    pub fn total_requests(&self) -> u64 {
        self.total_hits() + self.total_misses()
    }

    /// Calculate overall hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            0.0
        } else {
            self.total_hits() as f64 / total as f64
        }
    }

    /// Calculate memory hit rate
    pub fn memory_hit_rate(&self) -> f64 {
        let total = self.memory_hits + self.memory_misses;
        if total == 0 {
            0.0
        } else {
            self.memory_hits as f64 / total as f64
        }
    }

    /// Calculate Redis hit rate
    pub fn redis_hit_rate(&self) -> f64 {
        let total = self.redis_hits + self.redis_misses;
        if total == 0 {
            0.0
        } else {
            self.redis_hits as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== CacheKey Tests ====================

    #[test]
    fn test_cache_key_new() {
        let key = CacheKey::new("test-key");
        assert_eq!(key.as_str(), "test-key");
        assert!(key.hash_value() > 0);
    }

    #[test]
    fn test_cache_key_from_parts() {
        let key = CacheKey::from_parts("chat", &["gpt-4", "user-123"]);
        assert_eq!(key.as_str(), "chat:gpt-4:user-123");
    }

    #[test]
    fn test_cache_key_to_redis_key() {
        let key = CacheKey::new("my-key");
        assert_eq!(key.to_redis_key(), "litellm:cache:my-key");
    }

    #[test]
    fn test_cache_key_equality() {
        let key1 = CacheKey::new("same-key");
        let key2 = CacheKey::new("same-key");
        assert_eq!(key1, key2);
        assert_eq!(key1.hash_value(), key2.hash_value());
    }

    #[test]
    fn test_cache_key_inequality() {
        let key1 = CacheKey::new("key-1");
        let key2 = CacheKey::new("key-2");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_display() {
        let key = CacheKey::new("display-key");
        assert_eq!(format!("{}", key), "display-key");
    }

    #[test]
    fn test_cache_key_from_string() {
        let key: CacheKey = "from-string".into();
        assert_eq!(key.as_str(), "from-string");
    }

    // ==================== CacheEntry Tests ====================

    #[test]
    fn test_cache_entry_new() {
        let entry = CacheEntry::new("value", Duration::from_secs(60));
        assert_eq!(entry.value, "value");
        assert_eq!(entry.ttl, Duration::from_secs(60));
        assert_eq!(entry.access_count, 0);
    }

    #[test]
    fn test_cache_entry_with_size() {
        let entry = CacheEntry::with_size("value", Duration::from_secs(60), 100);
        assert_eq!(entry.size_bytes, 100);
    }

    #[test]
    fn test_cache_entry_not_expired() {
        let entry = CacheEntry::new("value", Duration::from_secs(3600));
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_expired() {
        let entry = CacheEntry::new("value", Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(10));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_entry_remaining_ttl() {
        let entry = CacheEntry::new("value", Duration::from_secs(60));
        let remaining = entry.remaining_ttl();
        assert!(remaining.is_some());
        assert!(remaining.unwrap() <= Duration::from_secs(60));
    }

    #[test]
    fn test_cache_entry_touch() {
        let mut entry = CacheEntry::new("value", Duration::from_secs(60));
        assert_eq!(entry.access_count, 0);
        entry.touch();
        assert_eq!(entry.access_count, 1);
        entry.touch();
        entry.touch();
        assert_eq!(entry.access_count, 3);
    }

    #[test]
    fn test_cache_entry_age() {
        let entry = CacheEntry::new("value", Duration::from_secs(60));
        std::thread::sleep(Duration::from_millis(10));
        let age = entry.age();
        assert!(age >= Duration::from_millis(10));
    }

    // ==================== SerializableCacheEntry Tests ====================

    #[test]
    fn test_serializable_entry_conversion() {
        let entry = CacheEntry::new("test-value".to_string(), Duration::from_secs(300));
        let serializable: SerializableCacheEntry<String> = (&entry).into();
        assert_eq!(serializable.value, "test-value");
        assert_eq!(serializable.ttl_secs, 300);
    }

    #[test]
    fn test_serializable_entry_roundtrip() {
        let original = CacheEntry::with_size("roundtrip".to_string(), Duration::from_secs(120), 50);
        let serializable: SerializableCacheEntry<String> = (&original).into();
        let restored = serializable.into_cache_entry();
        assert_eq!(restored.value, "roundtrip");
        assert_eq!(restored.ttl.as_secs(), 120);
        assert_eq!(restored.size_bytes, 50);
    }

    // ==================== EvictionPolicy Tests ====================

    #[test]
    fn test_eviction_policy_default() {
        let policy = EvictionPolicy::default();
        assert_eq!(policy, EvictionPolicy::LRU);
    }

    #[test]
    fn test_eviction_policy_display() {
        assert_eq!(format!("{}", EvictionPolicy::LRU), "lru");
        assert_eq!(format!("{}", EvictionPolicy::LFU), "lfu");
        assert_eq!(format!("{}", EvictionPolicy::TTL), "ttl");
        assert_eq!(format!("{}", EvictionPolicy::FIFO), "fifo");
    }

    #[test]
    fn test_eviction_policy_serialize() {
        let policy = EvictionPolicy::LRU;
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, "\"lru\"");
    }

    #[test]
    fn test_eviction_policy_deserialize() {
        let policy: EvictionPolicy = serde_json::from_str("\"lfu\"").unwrap();
        assert_eq!(policy, EvictionPolicy::LFU);
    }

    // ==================== CacheMode Tests ====================

    #[test]
    fn test_cache_mode_default() {
        let mode = CacheMode::default();
        assert_eq!(mode, CacheMode::Dual);
    }

    #[test]
    fn test_cache_mode_serialize() {
        let mode = CacheMode::MemoryOnly;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"memory_only\"");
    }

    // ==================== DualCacheConfig Tests ====================

    #[test]
    fn test_dual_cache_config_default() {
        let config = DualCacheConfig::default();
        assert_eq!(config.max_size, 10000);
        assert_eq!(config.default_ttl, Duration::from_secs(3600));
        assert_eq!(config.eviction_policy, EvictionPolicy::LRU);
        assert_eq!(config.mode, CacheMode::Dual);
        assert!(config.enable_stats);
    }

    #[test]
    fn test_dual_cache_config_memory_only() {
        let config = DualCacheConfig::memory_only();
        assert_eq!(config.mode, CacheMode::MemoryOnly);
    }

    #[test]
    fn test_dual_cache_config_redis_only() {
        let config = DualCacheConfig::redis_only();
        assert_eq!(config.mode, CacheMode::RedisOnly);
    }

    #[test]
    fn test_dual_cache_config_builder() {
        let config = DualCacheConfig::default()
            .with_max_size(5000)
            .with_ttl(Duration::from_secs(1800))
            .with_eviction_policy(EvictionPolicy::LFU);

        assert_eq!(config.max_size, 5000);
        assert_eq!(config.default_ttl, Duration::from_secs(1800));
        assert_eq!(config.eviction_policy, EvictionPolicy::LFU);
    }

    // ==================== AtomicCacheStats Tests ====================

    #[test]
    fn test_atomic_cache_stats_default() {
        let stats = AtomicCacheStats::default();
        let snapshot = stats.snapshot();
        assert_eq!(snapshot.memory_hits, 0);
        assert_eq!(snapshot.redis_hits, 0);
    }

    #[test]
    fn test_atomic_cache_stats_record() {
        let stats = AtomicCacheStats::new();
        stats.record_memory_hit();
        stats.record_memory_hit();
        stats.record_memory_miss();
        stats.record_redis_hit();
        stats.record_write();
        stats.record_eviction();

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.memory_hits, 2);
        assert_eq!(snapshot.memory_misses, 1);
        assert_eq!(snapshot.redis_hits, 1);
        assert_eq!(snapshot.writes, 1);
        assert_eq!(snapshot.evictions, 1);
    }

    #[test]
    fn test_atomic_cache_stats_reset() {
        let stats = AtomicCacheStats::new();
        stats.record_memory_hit();
        stats.record_redis_hit();
        stats.reset();

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.memory_hits, 0);
        assert_eq!(snapshot.redis_hits, 0);
    }

    #[test]
    fn test_atomic_cache_stats_concurrent() {
        use std::sync::Arc;
        use std::thread;

        let stats = Arc::new(AtomicCacheStats::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let stats_clone = Arc::clone(&stats);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    stats_clone.record_memory_hit();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(stats.snapshot().memory_hits, 1000);
    }

    // ==================== CacheStatsSnapshot Tests ====================

    #[test]
    fn test_cache_stats_snapshot_hit_rate() {
        let snapshot = CacheStatsSnapshot {
            memory_hits: 80,
            memory_misses: 20,
            redis_hits: 0,
            redis_misses: 0,
            ..Default::default()
        };

        assert!((snapshot.hit_rate() - 0.8).abs() < 0.001);
        assert!((snapshot.memory_hit_rate() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_cache_stats_snapshot_zero_requests() {
        let snapshot = CacheStatsSnapshot::default();
        assert_eq!(snapshot.hit_rate(), 0.0);
        assert_eq!(snapshot.total_requests(), 0);
    }

    #[test]
    fn test_cache_stats_snapshot_combined() {
        let snapshot = CacheStatsSnapshot {
            memory_hits: 50,
            memory_misses: 20,
            redis_hits: 30,
            redis_misses: 10,
            ..Default::default()
        };

        assert_eq!(snapshot.total_hits(), 80);
        assert_eq!(snapshot.total_misses(), 30);
        assert_eq!(snapshot.total_requests(), 110);
    }
}
