//! Cache system trait definitions
//!
//! Provides unified cache interface supporting multiple cache backends

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Core cache trait
///
/// Defines unified cache operation interface
#[allow(async_fn_in_trait)]
pub trait Cache<K, V>: Send + Sync
where
    K: Send + Sync,
    V: Send + Sync,
{
    /// Error
    type Error: std::error::Error + Send + Sync + 'static;

    /// Get
    async fn get(&self, key: &K) -> Result<Option<V>, Self::Error>;

    /// Settings
    async fn set(&self, key: &K, value: V, ttl: Duration) -> Result<(), Self::Error>;

    /// Delete
    async fn delete(&self, key: &K) -> Result<bool, Self::Error>;

    /// Check
    async fn exists(&self, key: &K) -> Result<bool, Self::Error>;

    /// Settings
    async fn expire(&self, key: &K, ttl: Duration) -> Result<bool, Self::Error>;

    /// Get
    async fn ttl(&self, key: &K) -> Result<Option<Duration>, Self::Error>;

    /// Clear all cache
    async fn clear(&self) -> Result<(), Self::Error>;

    /// Get
    async fn size(&self) -> Result<usize, Self::Error>;

    /// Get
    async fn get_many(&self, keys: &[K]) -> Result<Vec<Option<V>>, Self::Error> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.get(key).await?);
        }
        Ok(results)
    }

    /// Settings
    async fn set_many(&self, items: &[(K, V, Duration)]) -> Result<(), Self::Error>
    where
        K: Clone,
        V: Clone,
    {
        for (key, value, ttl) in items {
            self.set(key, value.clone(), *ttl).await?;
        }
        Ok(())
    }
}

/// Cache key trait
///
/// Defines operations that cache keys must support
pub trait CacheKey: Send + Sync + Clone + std::fmt::Debug + std::hash::Hash + Eq {
    /// Serialize key to string
    fn to_cache_key(&self) -> String;

    /// Deserialize key from string
    fn from_cache_key(s: &str) -> Result<Self, CacheError>
    where
        Self: Sized;
}

/// Cache value trait
///
/// Defines operations that cache values must support
pub trait CacheValue: Send + Sync + Clone + std::fmt::Debug {
    /// Serialize to bytes
    fn to_bytes(&self) -> Result<Vec<u8>, CacheError>;

    /// Deserialize from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, CacheError>
    where
        Self: Sized;
}

/// Implementation of CacheKey for String
impl CacheKey for String {
    fn to_cache_key(&self) -> String {
        self.clone()
    }

    fn from_cache_key(s: &str) -> Result<Self, CacheError> {
        Ok(s.to_string())
    }
}

/// Implementation of CacheValue for all types that implement Serialize + DeserializeOwned
impl<T> CacheValue for T
where
    T: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + std::fmt::Debug,
{
    fn to_bytes(&self) -> Result<Vec<u8>, CacheError> {
        rmp_serde::to_vec(self).map_err(CacheError::Serialization)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, CacheError> {
        rmp_serde::from_slice(bytes).map_err(CacheError::Deserialization)
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Cache hit count
    pub hits: u64,
    /// Cache miss count
    pub misses: u64,
    /// Current key count
    pub key_count: usize,
    /// Used memory amount (bytes)
    pub memory_usage: usize,
    /// Hit rate
    pub hit_rate: f64,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            key_count: 0,
            memory_usage: 0,
            hit_rate: 0.0,
        }
    }

    pub fn calculate_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hit_rate = self.hits as f64 / total as f64;
        }
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache trait with statistics functionality
#[allow(async_fn_in_trait)]
pub trait CacheWithStats<K, V>: Cache<K, V>
where
    K: Send + Sync,
    V: Send + Sync,
{
    /// Get
    async fn stats(&self) -> Result<CacheStats, Self::Error>;

    /// Reset statistics
    async fn reset_stats(&self) -> Result<(), Self::Error>;
}

/// Cache event types
#[derive(Debug, Clone)]
pub enum CacheEvent<K, V> {
    /// Cache hit
    Hit { key: K },
    /// Cache miss
    Miss { key: K },
    /// Settings
    Set { key: K, value: V },
    /// Delete
    Delete { key: K },
    /// Cache expiration
    Expire { key: K },
    /// Cache clear
    Clear,
}

/// Cache event listener
#[allow(async_fn_in_trait)]
pub trait CacheEventListener<K, V>: Send + Sync
where
    K: Send + Sync,
    V: Send + Sync,
{
    /// Handle
    async fn on_event(&self, event: CacheEvent<K, V>);
}

/// Error
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("Serialization failed: {0}")]
    Serialization(#[from] rmp_serde::encode::Error),

    #[error("Deserialization failed: {0}")]
    Deserialization(rmp_serde::decode::Error),

    #[error("Key not found: {key}")]
    KeyNotFound { key: String },

    #[error("Cache is full")]
    CacheFull,

    #[error("Invalid TTL: {ttl_ms}ms")]
    InvalidTTL { ttl_ms: u64 },

    #[error("Cache operation timeout")]
    Timeout,

    #[error("Cache backend error: {0}")]
    Backend(String),

    #[error("Other cache error: {0}")]
    Other(String),
}

impl CacheError {
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    pub fn key_not_found(key: impl Into<String>) -> Self {
        Self::KeyNotFound { key: key.into() }
    }

    pub fn backend(msg: impl Into<String>) -> Self {
        Self::Backend(msg.into())
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== CacheKey Tests ====================

    #[test]
    fn test_string_cache_key_to_cache_key() {
        let key = "my-cache-key".to_string();
        assert_eq!(key.to_cache_key(), "my-cache-key");
    }

    #[test]
    fn test_string_cache_key_from_cache_key() {
        let key = String::from_cache_key("restored-key").unwrap();
        assert_eq!(key, "restored-key");
    }

    #[test]
    fn test_string_cache_key_roundtrip() {
        let original = "test-key-123".to_string();
        let serialized = original.to_cache_key();
        let restored = String::from_cache_key(&serialized).unwrap();
        assert_eq!(original, restored);
    }

    // ==================== CacheValue Tests ====================

    #[test]
    fn test_cache_value_to_bytes_string() {
        let value = "hello world".to_string();
        let bytes = value.to_bytes();
        assert!(bytes.is_ok());
        assert!(!bytes.unwrap().is_empty());
    }

    #[test]
    fn test_cache_value_from_bytes_string() {
        let value = "test value".to_string();
        let bytes = value.to_bytes().unwrap();
        let restored = String::from_bytes(&bytes).unwrap();
        assert_eq!(value, restored);
    }

    #[test]
    fn test_cache_value_roundtrip_integer() {
        let value: i32 = 42;
        let bytes = value.to_bytes().unwrap();
        let restored = i32::from_bytes(&bytes).unwrap();
        assert_eq!(value, restored);
    }

    #[test]
    fn test_cache_value_roundtrip_complex() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct TestData {
            id: u64,
            name: String,
            active: bool,
        }

        let value = TestData {
            id: 123,
            name: "test".to_string(),
            active: true,
        };
        let bytes = value.to_bytes().unwrap();
        let restored = TestData::from_bytes(&bytes).unwrap();
        assert_eq!(value, restored);
    }

    // ==================== CacheStats Tests ====================

    #[test]
    fn test_cache_stats_new() {
        let stats = CacheStats::new();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.key_count, 0);
        assert_eq!(stats.memory_usage, 0);
        assert!((stats.hit_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_stats_default() {
        let stats = CacheStats::default();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_stats_calculate_hit_rate_zero_total() {
        let mut stats = CacheStats::new();
        stats.calculate_hit_rate();
        assert!((stats.hit_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_stats_calculate_hit_rate_all_hits() {
        let mut stats = CacheStats::new();
        stats.hits = 100;
        stats.misses = 0;
        stats.calculate_hit_rate();
        assert!((stats.hit_rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_stats_calculate_hit_rate_all_misses() {
        let mut stats = CacheStats::new();
        stats.hits = 0;
        stats.misses = 100;
        stats.calculate_hit_rate();
        assert!((stats.hit_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_stats_calculate_hit_rate_mixed() {
        let mut stats = CacheStats::new();
        stats.hits = 75;
        stats.misses = 25;
        stats.calculate_hit_rate();
        assert!((stats.hit_rate - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_cache_stats_clone() {
        let mut stats = CacheStats::new();
        stats.hits = 10;
        stats.key_count = 5;
        let cloned = stats.clone();
        assert_eq!(stats.hits, cloned.hits);
        assert_eq!(stats.key_count, cloned.key_count);
    }

    #[test]
    fn test_cache_stats_debug() {
        let stats = CacheStats::new();
        let debug = format!("{:?}", stats);
        assert!(debug.contains("CacheStats"));
    }

    // ==================== CacheEvent Tests ====================

    #[test]
    fn test_cache_event_hit() {
        let event: CacheEvent<String, i32> = CacheEvent::Hit {
            key: "key1".to_string(),
        };
        assert!(matches!(event, CacheEvent::Hit { key } if key == "key1"));
    }

    #[test]
    fn test_cache_event_miss() {
        let event: CacheEvent<String, i32> = CacheEvent::Miss {
            key: "key2".to_string(),
        };
        assert!(matches!(event, CacheEvent::Miss { key } if key == "key2"));
    }

    #[test]
    fn test_cache_event_set() {
        let event = CacheEvent::Set {
            key: "key3".to_string(),
            value: 42,
        };
        assert!(matches!(event, CacheEvent::Set { key, value } if key == "key3" && value == 42));
    }

    #[test]
    fn test_cache_event_delete() {
        let event: CacheEvent<String, i32> = CacheEvent::Delete {
            key: "key4".to_string(),
        };
        assert!(matches!(event, CacheEvent::Delete { key } if key == "key4"));
    }

    #[test]
    fn test_cache_event_expire() {
        let event: CacheEvent<String, i32> = CacheEvent::Expire {
            key: "key5".to_string(),
        };
        assert!(matches!(event, CacheEvent::Expire { key } if key == "key5"));
    }

    #[test]
    fn test_cache_event_clear() {
        let event: CacheEvent<String, i32> = CacheEvent::Clear;
        assert!(matches!(event, CacheEvent::Clear));
    }

    #[test]
    fn test_cache_event_clone() {
        let event = CacheEvent::Set {
            key: "key".to_string(),
            value: 100,
        };
        let cloned = event.clone();
        assert!(matches!(cloned, CacheEvent::Set { key, value } if key == "key" && value == 100));
    }

    // ==================== CacheError Tests ====================

    #[test]
    fn test_cache_error_connection() {
        let err = CacheError::connection("Redis connection failed");
        assert!(matches!(err, CacheError::Connection(_)));
        assert!(err.to_string().contains("Connection failed"));
    }

    #[test]
    fn test_cache_error_key_not_found() {
        let err = CacheError::key_not_found("missing-key");
        assert!(matches!(err, CacheError::KeyNotFound { .. }));
        assert!(err.to_string().contains("Key not found"));
        assert!(err.to_string().contains("missing-key"));
    }

    #[test]
    fn test_cache_error_cache_full() {
        let err = CacheError::CacheFull;
        assert!(err.to_string().contains("Cache is full"));
    }

    #[test]
    fn test_cache_error_invalid_ttl() {
        let err = CacheError::InvalidTTL { ttl_ms: 0 };
        assert!(err.to_string().contains("Invalid TTL"));
    }

    #[test]
    fn test_cache_error_timeout() {
        let err = CacheError::Timeout;
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn test_cache_error_backend() {
        let err = CacheError::backend("Backend failure");
        assert!(matches!(err, CacheError::Backend(_)));
        assert!(err.to_string().contains("Backend"));
    }

    #[test]
    fn test_cache_error_other() {
        let err = CacheError::other("Some other error");
        assert!(matches!(err, CacheError::Other(_)));
    }

    #[test]
    fn test_cache_error_display() {
        let err = CacheError::connection("test error");
        let display = format!("{}", err);
        assert!(!display.is_empty());
    }

    #[test]
    fn test_cache_error_debug() {
        let err = CacheError::CacheFull;
        let debug = format!("{:?}", err);
        assert!(debug.contains("CacheFull"));
    }
}
