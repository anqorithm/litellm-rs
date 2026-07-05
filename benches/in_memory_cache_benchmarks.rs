//! In-memory cache hot-path benchmarks.
//!
//! The `legacy_global_mutex_lru` case is a benchmark-local reference matching
//! the old single-LRU-mutex behavior so PRs can report comparable throughput
//! without keeping the old production implementation.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use dashmap::DashMap;
use litellm_rs::core::cache::{
    AtomicCacheStats, CacheEntry, CacheKey, DualCacheConfig, EvictionPolicy, InMemoryCache,
};
use lru::LruCache;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::Mutex;

const KEY_COUNT: usize = 4096;
const OPS_PER_TASK: usize = 512;
const VALUE: &str = "cached-value";
const TASK_COUNT: usize = 4;

struct LegacyGlobalMutexLruCache {
    cache: DashMap<CacheKey, CacheEntry<String>>,
    lru_order: Mutex<LruCache<CacheKey, ()>>,
    stats: AtomicCacheStats,
    max_size: usize,
}

impl LegacyGlobalMutexLruCache {
    fn new(max_size: usize) -> Self {
        Self {
            cache: DashMap::with_capacity(max_size),
            lru_order: Mutex::new(LruCache::new(
                NonZeroUsize::new(max_size).unwrap_or(NonZeroUsize::MIN),
            )),
            stats: AtomicCacheStats::new(),
            max_size,
        }
    }

    async fn get(&self, key: &CacheKey) -> Option<String> {
        if let Some(mut entry) = self.cache.get_mut(key) {
            entry.touch();

            let mut lru = self.lru_order.lock().await;
            if !lru.promote(key) {
                lru.push(key.clone(), ());
            }

            self.stats.record_memory_hit();
            Some(entry.value.clone())
        } else {
            self.stats.record_memory_miss();
            None
        }
    }

    async fn set(&self, key: CacheKey, value: String) {
        if self.cache.len() >= self.max_size {
            let evicted = self.lru_order.lock().await.pop_lru().map(|(key, _)| key);
            if let Some(evicted) = evicted {
                self.cache.remove(&evicted);
            }
        }

        let entry = CacheEntry::new(value, Duration::from_secs(60));
        let new_size = entry.size_bytes;
        let old = self.cache.insert(key.clone(), entry);
        self.stats.record_write();

        if let Some(old_entry) = old {
            self.stats.sub_total_size(old_entry.size_bytes);
        }

        let mut lru = self.lru_order.lock().await;
        if !lru.promote(&key) {
            lru.push(key, ());
        }

        self.stats.add_total_size(new_size);
        self.stats.set_entry_count(self.cache.len());
    }
}

fn runtime() -> Runtime {
    Builder::new_multi_thread()
        .worker_threads(8)
        .enable_time()
        .build()
        .expect("benchmark runtime should start")
}

fn cache_config() -> DualCacheConfig {
    DualCacheConfig::default()
        .with_max_size(KEY_COUNT * 2)
        .with_eviction_policy(EvictionPolicy::LRU)
}

fn keys() -> Arc<Vec<CacheKey>> {
    Arc::new(
        (0..KEY_COUNT)
            .map(|index| CacheKey::new(format!("key-{index}")))
            .collect(),
    )
}

async fn load_current(cache: &InMemoryCache<String>, keys: &[CacheKey]) {
    for key in keys {
        cache.set(key.clone(), VALUE.to_string()).await;
    }
}

async fn load_legacy(cache: &LegacyGlobalMutexLruCache, keys: &[CacheKey]) {
    for key in keys {
        cache.set(key.clone(), VALUE.to_string()).await;
    }
}

async fn current_get_hits(
    cache: Arc<InMemoryCache<String>>,
    keys: Arc<Vec<CacheKey>>,
    task_count: usize,
) {
    let mut handles = Vec::with_capacity(task_count);

    for task_index in 0..task_count {
        let cache = Arc::clone(&cache);
        let keys = Arc::clone(&keys);
        handles.push(tokio::spawn(async move {
            for op_index in 0..OPS_PER_TASK {
                let index = (op_index + task_index * 97) % KEY_COUNT;
                black_box(cache.get(&keys[index]).await);
            }
        }));
    }

    for handle in handles {
        handle.await.expect("get task should complete");
    }
}

async fn legacy_get_hits(
    cache: Arc<LegacyGlobalMutexLruCache>,
    keys: Arc<Vec<CacheKey>>,
    task_count: usize,
) {
    let mut handles = Vec::with_capacity(task_count);

    for task_index in 0..task_count {
        let cache = Arc::clone(&cache);
        let keys = Arc::clone(&keys);
        handles.push(tokio::spawn(async move {
            for op_index in 0..OPS_PER_TASK {
                let index = (op_index + task_index * 97) % KEY_COUNT;
                black_box(cache.get(&keys[index]).await);
            }
        }));
    }

    for handle in handles {
        handle.await.expect("get task should complete");
    }
}

fn bench_in_memory_cache_hot_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("in_memory_cache_hot_path");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    let operations = (TASK_COUNT * OPS_PER_TASK) as u64;

    group.throughput(Throughput::Elements(operations));
    group.bench_with_input(
        BenchmarkId::new("current_atomic_sampled_get_hits", TASK_COUNT),
        &TASK_COUNT,
        |b, &task_count| {
            let rt = runtime();
            let keys = keys();
            let cache = Arc::new(InMemoryCache::new(cache_config()));
            rt.block_on(load_current(&cache, &keys));

            b.iter(|| {
                rt.block_on(current_get_hits(
                    Arc::clone(&cache),
                    Arc::clone(&keys),
                    task_count,
                ));
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("legacy_global_mutex_lru_get_hits", TASK_COUNT),
        &TASK_COUNT,
        |b, &task_count| {
            let rt = runtime();
            let keys = keys();
            let cache = Arc::new(LegacyGlobalMutexLruCache::new(KEY_COUNT * 2));
            rt.block_on(load_legacy(&cache, &keys));

            b.iter(|| {
                rt.block_on(legacy_get_hits(
                    Arc::clone(&cache),
                    Arc::clone(&keys),
                    task_count,
                ));
            });
        },
    );

    group.finish();
}

criterion_group!(benches, bench_in_memory_cache_hot_path);
criterion_main!(benches);
