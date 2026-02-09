use super::*;

// ==================== Configuration Tests ====================

#[test]
fn test_dual_cache_default_config() {
    let cache: DualCache<String> = DualCache::with_defaults();
    assert_eq!(cache.mode(), CacheMode::Dual);
    assert!(cache.is_memory_empty());
}

#[test]
fn test_dual_cache_memory_only() {
    let cache: DualCache<String> = DualCache::memory_only(DualCacheConfig::default());
    assert_eq!(cache.mode(), CacheMode::MemoryOnly);
}

// ==================== Memory-Only Tests ====================

#[tokio::test]
async fn test_dual_cache_memory_set_get() {
    let cache: DualCache<String> = DualCache::memory_only(DualCacheConfig::default());
    let key = CacheKey::new("test-key");

    cache
        .set(key.clone(), "test-value".to_string())
        .await
        .unwrap();
    let result = cache.get(&key).await.unwrap();

    assert_eq!(result, Some("test-value".to_string()));
}

#[tokio::test]
async fn test_dual_cache_memory_delete() {
    let cache: DualCache<String> = DualCache::memory_only(DualCacheConfig::default());
    let key = CacheKey::new("to-delete");

    cache.set(key.clone(), "value".to_string()).await.unwrap();
    assert!(cache.exists(&key).await.unwrap());

    let deleted = cache.delete(&key).await.unwrap();
    assert!(deleted);
    assert!(!cache.exists(&key).await.unwrap());
}

#[tokio::test]
async fn test_dual_cache_memory_ttl() {
    let cache: DualCache<String> = DualCache::memory_only(DualCacheConfig::default());
    let key = CacheKey::new("ttl-key");

    cache
        .set_with_ttl(key.clone(), "value".to_string(), Duration::from_secs(60))
        .await
        .unwrap();

    let ttl = cache.ttl(&key).await.unwrap();
    assert!(ttl.is_some());
    assert!(ttl.unwrap() <= Duration::from_secs(60));
}

#[tokio::test]
async fn test_dual_cache_memory_clear() {
    let cache: DualCache<String> = DualCache::memory_only(DualCacheConfig::default());

    cache
        .set(CacheKey::new("key1"), "value1".to_string())
        .await
        .unwrap();
    cache
        .set(CacheKey::new("key2"), "value2".to_string())
        .await
        .unwrap();

    assert_eq!(cache.memory_len(), 2);

    cache.clear().await.unwrap();
    assert!(cache.is_memory_empty());
}

// ==================== Batch Operations Tests ====================

#[tokio::test]
async fn test_dual_cache_get_many() {
    let cache: DualCache<String> = DualCache::memory_only(DualCacheConfig::default());

    cache
        .set(CacheKey::new("key1"), "value1".to_string())
        .await
        .unwrap();
    cache
        .set(CacheKey::new("key2"), "value2".to_string())
        .await
        .unwrap();

    let keys = vec![
        CacheKey::new("key1"),
        CacheKey::new("key2"),
        CacheKey::new("key3"),
    ];

    let results = cache.get_many(&keys).await.unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0], Some("value1".to_string()));
    assert_eq!(results[1], Some("value2".to_string()));
    assert_eq!(results[2], None);
}

#[tokio::test]
async fn test_dual_cache_set_many() {
    let cache: DualCache<String> = DualCache::memory_only(DualCacheConfig::default());

    let entries = vec![
        (
            CacheKey::new("key1"),
            "value1".to_string(),
            Duration::from_secs(60),
        ),
        (
            CacheKey::new("key2"),
            "value2".to_string(),
            Duration::from_secs(60),
        ),
        (
            CacheKey::new("key3"),
            "value3".to_string(),
            Duration::from_secs(60),
        ),
    ];

    cache.set_many(&entries).await.unwrap();

    assert_eq!(cache.memory_len(), 3);
    assert!(cache.exists(&CacheKey::new("key1")).await.unwrap());
    assert!(cache.exists(&CacheKey::new("key2")).await.unwrap());
    assert!(cache.exists(&CacheKey::new("key3")).await.unwrap());
}

#[tokio::test]
async fn test_dual_cache_delete_many() {
    let cache: DualCache<String> = DualCache::memory_only(DualCacheConfig::default());

    cache
        .set(CacheKey::new("key1"), "value1".to_string())
        .await
        .unwrap();
    cache
        .set(CacheKey::new("key2"), "value2".to_string())
        .await
        .unwrap();
    cache
        .set(CacheKey::new("key3"), "value3".to_string())
        .await
        .unwrap();

    let keys = vec![CacheKey::new("key1"), CacheKey::new("key2")];
    let deleted = cache.delete_many(&keys).await.unwrap();

    assert_eq!(deleted, 2);
    assert_eq!(cache.memory_len(), 1);
    assert!(cache.exists(&CacheKey::new("key3")).await.unwrap());
}

// ==================== Cache Warming Tests ====================

#[tokio::test]
async fn test_dual_cache_warm_with_entries() {
    let cache: DualCache<String> = DualCache::memory_only(DualCacheConfig::default());

    let entries = vec![
        (
            CacheKey::new("key1"),
            "value1".to_string(),
            Duration::from_secs(60),
        ),
        (
            CacheKey::new("key2"),
            "value2".to_string(),
            Duration::from_secs(60),
        ),
    ];

    let warmed = cache.warm_with_entries(&entries);
    assert_eq!(warmed, 2);

    // Warming again should not add duplicates
    let warmed_again = cache.warm_with_entries(&entries);
    assert_eq!(warmed_again, 0);
}

#[tokio::test]
async fn test_dual_cache_warm_from_redis_without_redis_returns_zero() {
    let cache: DualCache<String> = DualCache::memory_only(DualCacheConfig::default());
    let keys = vec![CacheKey::new("key1"), CacheKey::new("key2")];

    let warmed = cache.warm_from_redis(&keys).await.unwrap();
    assert_eq!(warmed, 0);
}

// ==================== Statistics Tests ====================

#[tokio::test]
async fn test_dual_cache_stats() {
    let cache: DualCache<String> = DualCache::memory_only(DualCacheConfig::default());
    let key = CacheKey::new("stats-key");

    cache.set(key.clone(), "value".to_string()).await.unwrap();
    cache.get(&key).await.unwrap();
    cache.get(&key).await.unwrap();
    cache.get(&CacheKey::new("miss")).await.unwrap();

    let stats = cache.stats();
    assert_eq!(stats.memory_hits, 2);
    assert_eq!(stats.memory_misses, 1);
}

// ==================== Entry Metadata Tests ====================

#[tokio::test]
async fn test_dual_cache_get_entry() {
    let cache: DualCache<String> = DualCache::memory_only(DualCacheConfig::default());
    let key = CacheKey::new("entry-key");

    cache
        .set_with_size(
            key.clone(),
            "value".to_string(),
            Duration::from_secs(60),
            100,
        )
        .await
        .unwrap();

    let entry = cache.get_entry(&key).await.unwrap();
    assert!(entry.is_some());

    let entry = entry.unwrap();
    assert_eq!(entry.value, "value");
    assert_eq!(entry.size_bytes, 100);
}

// ==================== Expiration Tests ====================

#[tokio::test]
async fn test_dual_cache_expiration() {
    let cache: DualCache<String> = DualCache::memory_only(DualCacheConfig::default());
    let key = CacheKey::new("expiring-key");

    cache
        .set_with_ttl(key.clone(), "value".to_string(), Duration::from_millis(10))
        .await
        .unwrap();

    assert!(cache.exists(&key).await.unwrap());

    tokio::time::sleep(Duration::from_millis(20)).await;

    let result = cache.get(&key).await.unwrap();
    assert!(result.is_none());
}

// ==================== Complex Type Tests ====================

#[tokio::test]
async fn test_dual_cache_complex_type() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct ComplexValue {
        id: u64,
        name: String,
        scores: Vec<f64>,
    }

    let cache: DualCache<ComplexValue> = DualCache::memory_only(DualCacheConfig::default());
    let key = CacheKey::new("complex-key");

    let value = ComplexValue {
        id: 123,
        name: "test".to_string(),
        scores: vec![1.0, 2.5, 3.7],
    };

    cache.set(key.clone(), value.clone()).await.unwrap();
    let result = cache.get(&key).await.unwrap();

    assert_eq!(result, Some(value));
}
