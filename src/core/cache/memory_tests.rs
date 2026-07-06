use super::*;

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
    assert!(matches!(
        ttl,
        Some(remaining) if remaining <= Duration::from_secs(60)
    ));
}

#[tokio::test]
async fn test_cache_lru_eviction() {
    let config = DualCacheConfig::default()
        .with_max_size(3)
        .with_eviction_policy(EvictionPolicy::LRU);
    let cache: InMemoryCache<String> = InMemoryCache::new(config);

    cache.set(CacheKey::new("key1"), "value1".to_string()).await;
    cache.set(CacheKey::new("key2"), "value2".to_string()).await;
    cache.set(CacheKey::new("key3"), "value3".to_string()).await;

    cache.get(&CacheKey::new("key1")).await;
    cache.get(&CacheKey::new("key2")).await;
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

    for _ in 0..5 {
        cache.get(&CacheKey::new("key1")).await;
    }
    for _ in 0..2 {
        cache.get(&CacheKey::new("key2")).await;
    }
    cache.set(CacheKey::new("key4"), "value4".to_string()).await;

    assert!(cache.exists(&CacheKey::new("key1")).await);
    assert!(cache.exists(&CacheKey::new("key2")).await);
    assert!(!cache.exists(&CacheKey::new("key3")).await);
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

#[tokio::test]
async fn test_cache_ttl_eviction_prefers_expired_entry_outside_sample() {
    let config = DualCacheConfig::default()
        .with_max_size(EVICTION_SAMPLE_SIZE + 1)
        .with_eviction_policy(EvictionPolicy::TTL);
    let cache: InMemoryCache<String> = InMemoryCache::new(config);
    let shard_count = cache.access_meta.len();
    let sampled_shard = 0;
    let expired_shard = 1;

    let mut valid_keys = Vec::with_capacity(EVICTION_SAMPLE_SIZE);
    let mut attempt = 0;
    while valid_keys.len() < EVICTION_SAMPLE_SIZE {
        let key = CacheKey::new(format!("sampled-valid-{attempt}"));
        if key.hash_value() as usize % shard_count == sampled_shard {
            valid_keys.push(key);
        }
        attempt += 1;
    }

    for key in &valid_keys {
        cache
            .set_with_ttl(key.clone(), "valid".to_string(), Duration::from_secs(60))
            .await;
    }

    let expired_key = loop {
        let key = CacheKey::new(format!("expired-outside-sample-{attempt}"));
        attempt += 1;
        if key.hash_value() as usize % shard_count == expired_shard {
            break key;
        }
    };

    cache
        .set_with_ttl(
            expired_key.clone(),
            "expired".to_string(),
            Duration::from_millis(10),
        )
        .await;
    tokio::time::sleep(Duration::from_millis(20)).await;

    cache
        .eviction_cursor
        .store(sampled_shard, Ordering::Relaxed);
    cache
        .set_with_ttl(
            CacheKey::new("new-after-expired"),
            "new".to_string(),
            Duration::from_secs(60),
        )
        .await;

    assert!(!cache.exists(&expired_key).await);
    for key in valid_keys {
        assert!(cache.exists(&key).await, "valid key {key} was evicted");
    }
}

#[tokio::test]
async fn test_cache_lru_eviction_considers_entries_past_sample_size() {
    let config = DualCacheConfig::default()
        .with_max_size(EVICTION_SAMPLE_SIZE + 1)
        .with_eviction_policy(EvictionPolicy::LRU);
    let cache: InMemoryCache<String> = InMemoryCache::new(config);
    let shard_count = cache.access_meta.len();
    let target_shard = 0;

    let mut keys = Vec::with_capacity(EVICTION_SAMPLE_SIZE + 1);
    let mut attempt = 0;
    while keys.len() < EVICTION_SAMPLE_SIZE + 1 {
        let key = CacheKey::new(format!("lru-same-shard-{attempt}"));
        if key.hash_value() as usize % shard_count == target_shard {
            keys.push(key);
        }
        attempt += 1;
    }

    for key in &keys {
        cache
            .set_with_ttl(key.clone(), "value".to_string(), Duration::from_secs(60))
            .await;
    }

    let Some(true_lru) = keys.last().cloned() else {
        panic!("test should create one key beyond the historical sample");
    };
    for key in keys.iter().take(EVICTION_SAMPLE_SIZE) {
        cache.get(key).await;
    }

    cache.eviction_cursor.store(target_shard, Ordering::Relaxed);
    cache
        .set(CacheKey::new("new-after-lru"), "new".to_string())
        .await;

    assert!(
        !cache.exists(&true_lru).await,
        "LRU eviction should consider entries past the first sample"
    );
}

#[tokio::test]
async fn test_cache_stale_eviction_candidate_keeps_reinserted_metadata() {
    let config = DualCacheConfig::default()
        .with_max_size(2)
        .with_eviction_policy(EvictionPolicy::LRU);
    let cache: InMemoryCache<String> = InMemoryCache::new(config);
    let key = CacheKey::new("stale-candidate");

    cache.set(key.clone(), "old".to_string()).await;
    let Some(old_entry) = cache.cache.get(&key).map(|entry| entry.clone()) else {
        panic!("old entry should exist before stale candidate");
    };
    let Some((last_access_tick, access_count)) = cache
        .access_shard(&key)
        .get(&key)
        .map(|meta| meta.snapshot())
    else {
        panic!("old metadata should exist");
    };
    let stale_candidate = EvictionCandidate {
        key: key.clone(),
        last_access_tick,
        access_count,
        created_at: old_entry.created_at,
        remaining_ttl: old_entry.remaining_ttl(),
    };

    cache.cache.remove(&key);
    cache.set(key.clone(), "new".to_string()).await;
    assert!(
        !cache.evict_candidate(stale_candidate, "LRU"),
        "stale eviction candidate should report that no live entry was removed"
    );

    assert_eq!(cache.get(&key).await, Some("new".to_string()));
    assert!(
        cache.access_shard(&key).contains_key(&key),
        "metadata for the reinserted generation must remain"
    );
}

#[tokio::test]
async fn test_cache_stats_hits_misses() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("stats-key");

    cache.set(key.clone(), "value".to_string()).await;
    cache.get(&key).await;
    cache.get(&key).await;
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

#[tokio::test]
async fn test_cache_concurrent_read_write() {
    use std::sync::Arc;

    let cache = Arc::new(InMemoryCache::<i32>::with_defaults());
    let mut handles = vec![];

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
        match handle.await {
            Ok(()) => {}
            Err(error) => panic!("cache worker failed: {error}"),
        }
    }

    assert!(cache.len() <= 100);
}

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

    let Some(entry) = entry else {
        panic!("expected cache entry");
    };
    assert_eq!(entry.value, "value");
    assert_eq!(entry.size_bytes, 100);
    assert_eq!(entry.access_count, 1);
}

#[tokio::test]
async fn test_cache_get_entry_includes_prior_get_accesses() {
    let cache: InMemoryCache<String> = InMemoryCache::with_defaults();
    let key = CacheKey::new("entry-access-key");

    cache.set(key.clone(), "value".to_string()).await;
    cache.get(&key).await;
    cache.get(&key).await;

    let Some(entry) = cache.get_entry(&key).await else {
        panic!("expected cache entry");
    };
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
