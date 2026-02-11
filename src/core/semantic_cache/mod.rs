//! Semantic caching for AI responses
//!
//! This module provides intelligent caching based on semantic similarity of prompts.
//!
//! Boundary:
//! - `crate::core::cache` handles deterministic key-based cache.
//! - This module handles vector-similarity semantic cache.

mod cache;
mod types;
mod utils;
mod validation;

#[cfg(test)]
mod tests;

// Re-export main types and structs for backward compatibility
pub use cache::SemanticCache;
pub use types::{CacheStats, EmbeddingProvider, SemanticCacheConfig, SemanticCacheEntry};
