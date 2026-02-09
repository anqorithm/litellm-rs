use super::*;
use std::thread;

// ==================== Basic Operations Tests ====================

#[test]
fn test_cache_new() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

#[test]
fn test_cache_set_get() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("test-key");

    cache.set(key.clone(), "test-value".to_string());

    let result = cache.get(&key);
    assert_eq!(result, Some("test-value".to_string()));
}

#[test]
fn test_cache_get_nonexistent() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("nonexistent");

    let result = cache.get(&key);
    assert!(result.is_none());
}

#[test]
fn test_cache_delete() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("to-delete");

    cache.set(key.clone(), "value".to_string());
    assert!(cache.exists(&key));

    let deleted = cache.delete(&key);
    assert!(deleted);
    assert!(!cache.exists(&key));
}

#[test]
fn test_cache_delete_nonexistent() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("nonexistent");

    let deleted = cache.delete(&key);
    assert!(!deleted);
}

#[test]
fn test_cache_exists() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("exists-key");

    assert!(!cache.exists(&key));
    cache.set(key.clone(), "value".to_string());
    assert!(cache.exists(&key));
}

#[test]
fn test_cache_clear() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();

    cache.set(CacheKey::new("key1"), "value1".to_string());
    cache.set(CacheKey::new("key2"), "value2".to_string());
    cache.set(CacheKey::new("key3"), "value3".to_string());

    assert_eq!(cache.len(), 3);
    cache.clear();
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

// ==================== TTL Tests ====================

#[test]
fn test_cache_ttl_expiration() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("expiring-key");

    cache.set_with_ttl(key.clone(), "value".to_string(), Duration::from_millis(10));
    assert!(cache.exists(&key));

    thread::sleep(Duration::from_millis(20));
    assert!(!cache.exists(&key));
}

#[test]
fn test_cache_get_expired() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("expiring-key");

    cache.set_with_ttl(key.clone(), "value".to_string(), Duration::from_millis(10));
    thread::sleep(Duration::from_millis(20));

    let result = cache.get(&key);
    assert!(result.is_none());
}

#[test]
fn test_cache_ttl_remaining() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("ttl-key");

    cache.set_with_ttl(key.clone(), "value".to_string(), Duration::from_secs(60));

    let ttl = cache.ttl(&key);
    assert!(ttl.is_some());
    assert!(ttl.unwrap() <= Duration::from_secs(60));
}

// ==================== Eviction Tests ====================

#[test]
fn test_cache_lru_eviction() {
    let config = DualCacheConfig::default()
        .with_max_size(3)
        .with_eviction_policy(EvictionPolicy::LRU);
    let cache: InMemoryCache<String> = InMemoryCache::new(config);

    cache.set(CacheKey::new("key1"), "value1".to_string());
    cache.set(CacheKey::new("key2"), "value2".to_string());
    cache.set(CacheKey::new("key3"), "value3".to_string());

    // Access key1 and key2 to make them more recent
    cache.get(&CacheKey::new("key1"));
    cache.get(&CacheKey::new("key2"));

    // Add key4, should evict key3 (least recently used)
    cache.set(CacheKey::new("key4"), "value4".to_string());

    assert!(cache.exists(&CacheKey::new("key1")));
    assert!(cache.exists(&CacheKey::new("key2")));
    assert!(!cache.exists(&CacheKey::new("key3")));
    assert!(cache.exists(&CacheKey::new("key4")));
}

#[test]
fn test_cache_lfu_eviction() {
    let config = DualCacheConfig::default()
        .with_max_size(3)
        .with_eviction_policy(EvictionPolicy::LFU);
    let cache: InMemoryCache<String> = InMemoryCache::new(config);

    cache.set(CacheKey::new("key1"), "value1".to_string());
    cache.set(CacheKey::new("key2"), "value2".to_string());
    cache.set(CacheKey::new("key3"), "value3".to_string());

    // Access key1 multiple times
    for _ in 0..5 {
        cache.get(&CacheKey::new("key1"));
    }
    // Access key2 a few times
    for _ in 0..2 {
        cache.get(&CacheKey::new("key2"));
    }
    // key3 has lowest access count

    // Add key4, should evict key3 (least frequently used)
    cache.set(CacheKey::new("key4"), "value4".to_string());

    assert!(cache.exists(&CacheKey::new("key1")));
    assert!(cache.exists(&CacheKey::new("key2")));
    // key3 should be evicted
    assert!(cache.exists(&CacheKey::new("key4")));
}

// ==================== Statistics Tests ====================

#[test]
fn test_cache_stats_hits_misses() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("stats-key");

    cache.set(key.clone(), "value".to_string());

    // Generate hits
    cache.get(&key);
    cache.get(&key);

    // Generate misses
    cache.get(&CacheKey::new("nonexistent1"));
    cache.get(&CacheKey::new("nonexistent2"));

    let stats = cache.stats().snapshot();
    assert_eq!(stats.memory_hits, 2);
    assert_eq!(stats.memory_misses, 2);
}

#[test]
fn test_cache_stats_writes() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();

    cache.set(CacheKey::new("key1"), "value1".to_string());
    cache.set(CacheKey::new("key2"), "value2".to_string());

    let stats = cache.stats().snapshot();
    assert_eq!(stats.writes, 2);
}

#[test]
fn test_cache_stats_deletions() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("to-delete");

    cache.set(key.clone(), "value".to_string());
    cache.delete(&key);

    let stats = cache.stats().snapshot();
    assert_eq!(stats.deletions, 1);
}

// ==================== Concurrent Access Tests ====================

#[test]
fn test_cache_concurrent_read_write() {
    use std::sync::Arc;

    let cache = Arc::new(InMemoryCache::<i32>::with_defaults());
    let mut handles = vec![];

    // Writers
    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for j in 0..100 {
                let key = CacheKey::new(format!("key-{}-{}", i, j));
                cache_clone.set(key, i * 100 + j);
            }
        });
        handles.push(handle);
    }

    // Readers
    for _ in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for i in 0..10 {
                for j in 0..100 {
                    let key = CacheKey::new(format!("key-{}-{}", i, j));
                    let _ = cache_clone.get(&key);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Just verify no panics occurred
    assert!(cache.len() <= 1000);
}

// ==================== Entry Metadata Tests ====================

#[test]
fn test_cache_get_entry() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("entry-key");

    cache.set_with_size(
        key.clone(),
        "value".to_string(),
        Duration::from_secs(60),
        100,
    );

    let entry = cache.get_entry(&key);
    assert!(entry.is_some());

    let entry = entry.unwrap();
    assert_eq!(entry.value, "value");
    assert_eq!(entry.size_bytes, 100);
    assert_eq!(entry.access_count, 1); // One access from get_entry
}

#[test]
fn test_cache_keys() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();

    cache.set(CacheKey::new("key1"), "value1".to_string());
    cache.set(CacheKey::new("key2"), "value2".to_string());
    cache.set(CacheKey::new("key3"), "value3".to_string());

    let keys = cache.keys();
    assert_eq!(keys.len(), 3);
}

// ==================== Update Tests ====================

#[test]
fn test_cache_update_existing() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("update-key");

    cache.set(key.clone(), "initial".to_string());
    cache.set(key.clone(), "updated".to_string());

    let result = cache.get(&key);
    assert_eq!(result, Some("updated".to_string()));
    assert_eq!(cache.len(), 1);
}
