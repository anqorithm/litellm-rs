//! Type definitions for semantic caching

use crate::core::models::openai::ChatCompletionResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semantic cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCacheEntry {
    /// Unique cache entry ID
    pub id: String,
    /// Original prompt/messages hash
    pub prompt_hash: String,
    /// Prompt embedding vector
    pub embedding: Vec<f32>,
    /// Cached response
    pub response: ChatCompletionResponse,
    /// Model used for the response
    pub model: String,
    /// Cache creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last access timestamp
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    /// Access count
    pub access_count: u64,
    /// TTL in seconds
    pub ttl_seconds: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Semantic cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCacheConfig {
    /// Similarity threshold (0.0 to 1.0)
    pub similarity_threshold: f64,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Default TTL in seconds
    pub default_ttl_seconds: u64,
    /// Embedding model to use
    pub embedding_model: String,
    /// Enable cache for streaming responses
    pub enable_streaming_cache: bool,
    /// Minimum prompt length to cache
    pub min_prompt_length: usize,
    /// Cache hit boost factor
    pub cache_hit_boost: f64,
}

impl Default for SemanticCacheConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,
            max_cache_size: 10000,
            default_ttl_seconds: 3600, // 1 hour
            embedding_model: "text-embedding-ada-002".to_string(),
            enable_streaming_cache: false,
            min_prompt_length: 10,
            cache_hit_boost: 1.1,
        }
    }
}

/// Cache statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total cache entries
    pub total_entries: u64,
    /// Average similarity score for hits
    pub avg_hit_similarity: f64,
    /// Cache size in bytes (approximate)
    pub cache_size_bytes: u64,
}

/// Consolidated cache data - single lock for cache entries and statistics
#[derive(Debug, Default)]
pub(super) struct CacheData {
    /// In-memory cache for recent entries
    pub entries: HashMap<String, SemanticCacheEntry>,
    /// Cache statistics
    pub stats: CacheStats,
}

/// Trait for embedding providers
#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding for text
    async fn generate_embedding(&self, text: &str) -> crate::utils::error::error::Result<Vec<f32>>;

    /// Get embedding dimension
    fn embedding_dimension(&self) -> usize;
}


#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
