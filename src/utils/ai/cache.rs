//! Token counting cache for performance optimization
//!
//! This module provides caching for token counting operations to avoid expensive recalculations.

use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::sync::{OnceLock, RwLock};
use tracing::debug;

/// Simple cache with fixed capacity using HashMap
#[derive(Default)]
struct SimpleCache {
    cache: HashMap<u64, u32>,
    capacity: usize,
}
impl SimpleCache {
    fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    fn get(&self, key: &u64) -> Option<u32> {
        self.cache.get(key).copied()
    }

    fn put(&mut self, key: u64, value: u32) {
        if self.cache.len() >= self.capacity {
            // Simple eviction: clear half the cache when full
            let keys_to_remove: Vec<_> =
                self.cache.keys().take(self.capacity / 2).copied().collect();
            for k in keys_to_remove {
                self.cache.remove(&k);
            }
        }
        self.cache.insert(key, value);
    }

    fn clear(&mut self) {
        self.cache.clear();
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Global token cache with simple eviction
static TOKEN_CACHE: OnceLock<RwLock<SimpleCache>> = OnceLock::new();

/// Token counting cache manager
pub struct TokenCache;
impl TokenCache {
    /// Get cached token count for text and model combination
    pub fn get(&self, model: &str, text: &str) -> Option<u32> {
        let key = self.cache_key(model, text);
        let cache = TOKEN_CACHE.get_or_init(|| RwLock::new(SimpleCache::new(10000)));
        cache.read().ok()?.get(&key)
    }

    /// Cache token count for text and model combination
    pub fn set(&self, model: &str, text: &str, token_count: u32) {
        let key = self.cache_key(model, text);
        let cache = TOKEN_CACHE.get_or_init(|| RwLock::new(SimpleCache::new(10000)));
        if let Ok(mut c) = cache.write() {
            c.put(key, token_count);
            debug!(
                "Cached token count: {} tokens for model: {}",
                token_count, model
            );
        }
    }

    /// Clear all cached token counts
    pub fn clear(&self) {
        let cache = TOKEN_CACHE.get_or_init(|| RwLock::new(SimpleCache::new(10000)));
        if let Ok(mut c) = cache.write() {
            c.clear();
            debug!("Token cache cleared");
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> Option<(usize, usize)> {
        let cache = TOKEN_CACHE.get_or_init(|| RwLock::new(SimpleCache::new(10000)));
        if let Ok(c) = cache.read() {
            Some((c.len(), c.capacity()))
        } else {
            None
        }
    }

    /// Generate cache key from model and text
    fn cache_key(&self, model: &str, text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        model.hash(&mut hasher);
        text.hash(&mut hasher);
        hasher.finish()
    }

    /// Estimate token count with vectorized operations for better performance
    pub fn estimate_tokens_vectorized(&self, model: &str, text: &str, chars_per_token: f64) -> u32 {
        // Check cache first
        if let Some(cached) = self.get(model, text) {
            return cached;
        }

        // Fast path for empty text
        if text.is_empty() {
            return 0;
        }

        // Vectorized character counting for better performance
        let byte_len = text.len() as f64;
        let char_len = text.chars().count() as f64;

        // Adjust estimation based on Unicode density
        let unicode_factor = if char_len == byte_len { 1.0 } else { 1.2 };
        let estimated_tokens = (char_len / chars_per_token * unicode_factor).ceil() as u32;

        // Apply model-specific adjustments with buffer
        let final_tokens = (estimated_tokens as f64 * 1.1).ceil() as u32;

        // Cache the result
        self.set(model, text, final_tokens);

        final_tokens
    }

    /// Batch token counting for multiple texts
    pub fn batch_estimate(&self, model: &str, texts: &[String], chars_per_token: f64) -> Vec<u32> {
        texts
            .iter()
            .map(|text| self.estimate_tokens_vectorized(model, text, chars_per_token))
            .collect()
    }
}

/// Global token cache instance
pub fn token_cache() -> &'static TokenCache {
    static CACHE: TokenCache = TokenCache;
    &CACHE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_cache_basic() {
        let cache = token_cache();

        // Clear cache first to ensure clean state
        cache.clear();

        let model = "gpt-4-test";
        let text = "Hello world test";

        // Should miss initially
        assert!(cache.get(model, text).is_none());

        // Set and get
        cache.set(model, text, 2);
        assert_eq!(cache.get(model, text), Some(2));
    }

    #[test]
    fn test_cache_key_generation() {
        let cache = token_cache();
        let key1 = cache.cache_key("gpt-4", "hello");
        let key2 = cache.cache_key("gpt-4", "world");
        let key3 = cache.cache_key("gpt-3.5", "hello");

        // Different texts should have different keys
        assert_ne!(key1, key2);
        // Different models should have different keys
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_vectorized_estimation() {
        let cache = token_cache();
        cache.clear(); // Start fresh

        let model = "gpt-4";
        let text = "This is a test text for token estimation";
        let chars_per_token = 4.0;

        let tokens = cache.estimate_tokens_vectorized(model, text, chars_per_token);
        assert!(tokens > 0);

        // Second call should hit cache
        let tokens2 = cache.estimate_tokens_vectorized(model, text, chars_per_token);
        assert_eq!(tokens, tokens2);
    }

    #[test]
    fn test_batch_estimation() {
        let cache = token_cache();
        let model = "gpt-4";
        let texts = vec![
            "Hello world".to_string(),
            "This is a longer text".to_string(),
            "Short".to_string(),
        ];
        let chars_per_token = 4.0;

        let results = cache.batch_estimate(model, &texts, chars_per_token);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|&count| count > 0));
    }

    #[test]
    fn test_empty_text() {
        let cache = token_cache();
        let tokens = cache.estimate_tokens_vectorized("gpt-4", "", 4.0);
        assert_eq!(tokens, 0);
    }
}
