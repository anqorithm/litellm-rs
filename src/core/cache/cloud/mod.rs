//! Cloud storage cache backends
//!
//! This module provides cache implementations backed by cloud object storage services:
//! - AWS S3
//! - Google Cloud Storage (GCS)
//! - Azure Blob Storage
//!
//! These backends are suitable for persistent caching across distributed systems
//! where durability and cross-region access are important.

pub mod azure_blob;
pub mod gcs;
pub mod s3;

#[cfg(feature = "s3")]
pub use s3::{S3Cache, S3CacheConfig, S3StorageClass};

#[cfg(feature = "s3")]
pub use gcs::{GcsCache, GcsCacheConfig};

#[cfg(feature = "s3")]
pub use azure_blob::{AzureBlobCache, AzureBlobCacheConfig};

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

use super::types::CacheKey;
use crate::utils::error::error::Result;

/// Trait for cloud storage cache backends
#[async_trait]
pub trait CloudCache: Send + Sync {
    /// Get a value from the cache
    async fn get<T: DeserializeOwned + Send>(&self, key: &CacheKey) -> Result<Option<T>>;

    /// Set a value in the cache with TTL
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &CacheKey,
        value: &T,
        ttl: Duration,
    ) -> Result<()>;

    /// Delete a value from the cache
    async fn delete(&self, key: &CacheKey) -> Result<bool>;

    /// Check if a key exists
    async fn exists(&self, key: &CacheKey) -> Result<bool>;

    /// List keys with a prefix
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;

    /// Clear all cache entries (use with caution)
    async fn clear(&self) -> Result<()>;

    /// Get the backend name
    fn name(&self) -> &'static str;
}

/// Cloud cache configuration
#[derive(Debug, Clone)]
pub struct CloudCacheConfig {
    /// Bucket/container name
    pub bucket: String,
    /// Key prefix for all cache entries
    pub prefix: String,
    /// Default TTL for cache entries
    pub default_ttl: Duration,
    /// Enable compression for large values
    pub enable_compression: bool,
    /// Compression threshold in bytes
    pub compression_threshold: usize,
}

impl Default for CloudCacheConfig {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            prefix: "litellm-cache/".to_string(),
            default_ttl: Duration::from_secs(3600),
            enable_compression: true,
            compression_threshold: 1024,
        }
    }
}

impl CloudCacheConfig {
    /// Create a new configuration with the specified bucket
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            ..Default::default()
        }
    }

    /// Set the key prefix
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Set the default TTL
    pub fn default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// Enable or disable compression
    pub fn compression(mut self, enabled: bool) -> Self {
        self.enable_compression = enabled;
        self
    }

    /// Set the compression threshold
    pub fn compression_threshold(mut self, threshold: usize) -> Self {
        self.compression_threshold = threshold;
        self
    }
}

/// Cache entry metadata stored alongside the value
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct CacheMetadata {
    /// Unix timestamp when the entry expires
    pub expires_at: u64,
    /// Original size before compression
    pub original_size: usize,
    /// Whether the value is compressed
    pub compressed: bool,
    /// Content type
    pub content_type: String,
}

impl CacheMetadata {
    /// Create new metadata
    pub fn new(ttl: Duration, original_size: usize, compressed: bool) -> Self {
        let expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + ttl.as_secs();

        Self {
            expires_at,
            original_size,
            compressed,
            content_type: "application/json".to_string(),
        }
    }

    /// Check if the entry has expired
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now >= self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_cache_config_default() {
        let config = CloudCacheConfig::default();
        assert!(config.bucket.is_empty());
        assert_eq!(config.prefix, "litellm-cache/");
        assert_eq!(config.default_ttl, Duration::from_secs(3600));
        assert!(config.enable_compression);
    }

    #[test]
    fn test_cloud_cache_config_builder() {
        let config = CloudCacheConfig::new("my-bucket")
            .prefix("cache/")
            .default_ttl(Duration::from_secs(7200))
            .compression(false);

        assert_eq!(config.bucket, "my-bucket");
        assert_eq!(config.prefix, "cache/");
        assert_eq!(config.default_ttl, Duration::from_secs(7200));
        assert!(!config.enable_compression);
    }

    #[test]
    fn test_cache_metadata_not_expired() {
        let metadata = CacheMetadata::new(Duration::from_secs(3600), 100, false);
        assert!(!metadata.is_expired());
    }

    #[test]
    fn test_cache_metadata_expired() {
        let mut metadata = CacheMetadata::new(Duration::from_secs(0), 100, false);
        metadata.expires_at = 0; // Force expiration
        assert!(metadata.is_expired());
    }
}
