//! Google Cloud Storage cache backend implementation
//!
//! Provides a cache backend using Google Cloud Storage for persistent, distributed caching.

#[cfg(feature = "s3")]
mod implementation {
    use async_trait::async_trait;
    use object_store::ObjectStore;
    use object_store::gcp::GoogleCloudStorageBuilder;
    use object_store::path::Path;
    use serde::{Serialize, de::DeserializeOwned};
    use std::sync::Arc;
    use std::time::Duration;
    use tracing::{debug, error, warn};

    use crate::core::cache::cloud::{CacheMetadata, CloudCache, CloudCacheConfig};
    use crate::core::cache::types::CacheKey;
    use crate::utils::error::error::{GatewayError, Result};

    /// GCS cache configuration
    #[derive(Debug, Clone, Default)]
    pub struct GcsCacheConfig {
        /// Base cloud cache config
        pub base: CloudCacheConfig,
        /// Service account key path
        pub service_account_path: Option<String>,
        /// Service account key JSON content
        pub service_account_key: Option<String>,
    }

    impl GcsCacheConfig {
        /// Create a new GCS cache configuration
        pub fn new(bucket: impl Into<String>) -> Self {
            Self {
                base: CloudCacheConfig::new(bucket),
                ..Default::default()
            }
        }

        /// Set the service account key path
        pub fn service_account_path(mut self, path: impl Into<String>) -> Self {
            self.service_account_path = Some(path.into());
            self
        }

        /// Set the service account key JSON content
        pub fn service_account_key(mut self, key: impl Into<String>) -> Self {
            self.service_account_key = Some(key.into());
            self
        }

        /// Set the key prefix
        pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
            self.base.prefix = prefix.into();
            self
        }

        /// Set the default TTL
        pub fn default_ttl(mut self, ttl: Duration) -> Self {
            self.base.default_ttl = ttl;
            self
        }

        /// Create from environment variables
        pub fn from_env() -> Option<Self> {
            let bucket = std::env::var("GCS_CACHE_BUCKET").ok()?;
            Some(Self {
                base: CloudCacheConfig::new(bucket),
                service_account_path: std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok(),
                service_account_key: std::env::var("GCS_SERVICE_ACCOUNT_KEY").ok(),
            })
        }
    }

    /// Google Cloud Storage cache backend
    pub struct GcsCache {
        store: Arc<dyn ObjectStore>,
        config: GcsCacheConfig,
    }

    impl GcsCache {
        /// Create a new GCS cache
        pub async fn new(config: GcsCacheConfig) -> Result<Self> {
            let mut builder =
                GoogleCloudStorageBuilder::new().with_bucket_name(&config.base.bucket);

            if let Some(ref key_path) = config.service_account_path {
                builder = builder.with_service_account_path(key_path);
            }

            if let Some(ref key) = config.service_account_key {
                builder = builder.with_service_account_key(key);
            }

            let store = builder.build().map_err(|e| {
                GatewayError::FileStorage(format!("Failed to create GCS client: {}", e))
            })?;

            Ok(Self {
                store: Arc::new(store),
                config,
            })
        }

        /// Create from environment variables
        pub async fn from_env() -> Result<Self> {
            let config = GcsCacheConfig::from_env().ok_or_else(|| {
                GatewayError::Config("GCS_CACHE_BUCKET not set in environment".to_string())
            })?;
            Self::new(config).await
        }

        /// Build the full GCS path for a cache key
        fn build_path(&self, key: &CacheKey) -> Path {
            Path::from(format!("{}{}", self.config.base.prefix, key.as_str()))
        }

        /// Build the metadata path for a cache key
        fn build_metadata_path(&self, key: &CacheKey) -> Path {
            Path::from(format!("{}{}._meta", self.config.base.prefix, key.as_str()))
        }
    }

    #[async_trait]
    impl CloudCache for GcsCache {
        async fn get<T: DeserializeOwned + Send>(&self, key: &CacheKey) -> Result<Option<T>> {
            let path = self.build_path(key);
            let meta_path = self.build_metadata_path(key);
            debug!(key = %path, "Reading from GCS cache");

            // First check metadata for expiration
            match self.store.get(&meta_path).await {
                Ok(meta_result) => {
                    let meta_bytes = meta_result.bytes().await.map_err(|e| {
                        GatewayError::FileStorage(format!("Failed to read metadata: {}", e))
                    })?;

                    let metadata: CacheMetadata =
                        serde_json::from_slice(&meta_bytes).map_err(|e| {
                            GatewayError::FileStorage(format!("Failed to parse metadata: {}", e))
                        })?;

                    if metadata.is_expired() {
                        debug!(key = %path, "Cache entry expired");
                        let _ = self.delete(key).await;
                        return Ok(None);
                    }
                }
                Err(object_store::Error::NotFound { .. }) => {
                    debug!(key = %path, "Cache miss - no metadata");
                    return Ok(None);
                }
                Err(err) => {
                    warn!(key = %path, error = %err, "Failed to read metadata");
                    return Ok(None);
                }
            }

            // Read the actual value
            match self.store.get(&path).await {
                Ok(result) => {
                    let bytes = result.bytes().await.map_err(|e| {
                        GatewayError::FileStorage(format!("Failed to read body: {}", e))
                    })?;

                    let value: T = serde_json::from_slice(&bytes).map_err(|e| {
                        GatewayError::FileStorage(format!("Failed to deserialize: {}", e))
                    })?;

                    debug!(key = %path, "GCS cache hit");
                    Ok(Some(value))
                }
                Err(object_store::Error::NotFound { .. }) => {
                    debug!(key = %path, "Cache miss");
                    Ok(None)
                }
                Err(err) => {
                    error!(key = %path, error = %err, "Failed to read from GCS");
                    Err(GatewayError::FileStorage(format!(
                        "GCS read error: {}",
                        err
                    )))
                }
            }
        }

        async fn set<T: Serialize + Send + Sync>(
            &self,
            key: &CacheKey,
            value: &T,
            ttl: Duration,
        ) -> Result<()> {
            let path = self.build_path(key);
            let meta_path = self.build_metadata_path(key);
            debug!(key = %path, ttl_secs = ttl.as_secs(), "Writing to GCS cache");

            // Serialize the value
            let bytes = serde_json::to_vec(value)
                .map_err(|e| GatewayError::FileStorage(format!("Failed to serialize: {}", e)))?;

            // Create metadata
            let metadata = CacheMetadata::new(ttl, bytes.len(), false);
            let meta_bytes = serde_json::to_vec(&metadata).map_err(|e| {
                GatewayError::FileStorage(format!("Failed to serialize metadata: {}", e))
            })?;

            // Write the value
            self.store
                .put(&path, bytes.into())
                .await
                .map_err(|e| GatewayError::FileStorage(format!("Failed to write to GCS: {}", e)))?;

            // Write metadata
            self.store
                .put(&meta_path, meta_bytes.into())
                .await
                .map_err(|e| {
                    GatewayError::FileStorage(format!("Failed to write metadata: {}", e))
                })?;

            debug!(key = %path, "GCS cache write successful");
            Ok(())
        }

        async fn delete(&self, key: &CacheKey) -> Result<bool> {
            let path = self.build_path(key);
            let meta_path = self.build_metadata_path(key);
            debug!(key = %path, "Deleting from GCS cache");

            let value_result = self.store.delete(&path).await;
            let meta_result = self.store.delete(&meta_path).await;

            // Check if at least one deletion succeeded
            let deleted = value_result.is_ok() || meta_result.is_ok();

            if deleted {
                debug!(key = %path, "GCS cache delete successful");
            }

            Ok(deleted)
        }

        async fn exists(&self, key: &CacheKey) -> Result<bool> {
            let path = self.build_path(key);

            match self.store.head(&path).await {
                Ok(_) => Ok(true),
                Err(object_store::Error::NotFound { .. }) => Ok(false),
                Err(err) => Err(GatewayError::FileStorage(format!(
                    "GCS head error: {}",
                    err
                ))),
            }
        }

        async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
            use futures::StreamExt;

            let full_prefix = Path::from(format!("{}{}", self.config.base.prefix, prefix));

            let mut keys = Vec::new();
            let mut stream = self.store.list(Some(&full_prefix));

            while let Some(result) = stream.next().await {
                match result {
                    Ok(meta) => {
                        let key = meta.location.to_string();
                        // Remove prefix and filter out metadata keys
                        if !key.ends_with("._meta") {
                            if let Some(stripped) = key.strip_prefix(&self.config.base.prefix) {
                                keys.push(stripped.to_string());
                            } else {
                                keys.push(key);
                            }
                        }
                    }
                    Err(err) => {
                        warn!(error = %err, "Error listing GCS objects");
                    }
                }
            }

            Ok(keys)
        }

        async fn clear(&self) -> Result<()> {
            warn!("Clearing all GCS cache entries");

            let keys = self.list_keys("").await?;

            for key in keys {
                let cache_key = CacheKey::new(key);
                let _ = self.delete(&cache_key).await;
            }

            Ok(())
        }

        fn name(&self) -> &'static str {
            "gcs"
        }
    }

    impl std::fmt::Debug for GcsCache {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("GcsCache")
                .field("bucket", &self.config.base.bucket)
                .field("prefix", &self.config.base.prefix)
                .finish()
        }
    }
}

#[cfg(feature = "s3")]
pub use implementation::*;

// Stub implementation when feature is disabled
#[cfg(not(feature = "s3"))]
mod stub {
    use crate::utils::error::error::{GatewayError, Result};

    /// GCS cache configuration (stub)
    #[derive(Debug, Clone, Default)]
    pub struct GcsCacheConfig {
        pub bucket: String,
    }

    impl GcsCacheConfig {
        pub fn new(bucket: impl Into<String>) -> Self {
            Self {
                bucket: bucket.into(),
            }
        }

        pub fn from_env() -> Option<Self> {
            None
        }
    }

    /// GCS cache (stub)
    #[derive(Debug)]
    pub struct GcsCache;

    impl GcsCache {
        pub async fn new(_config: GcsCacheConfig) -> Result<Self> {
            Err(GatewayError::Config(
                "GCS cache support not enabled. Enable the 's3' feature.".to_string(),
            ))
        }

        pub async fn from_env() -> Result<Self> {
            Err(GatewayError::Config(
                "GCS cache support not enabled. Enable the 's3' feature.".to_string(),
            ))
        }
    }
}

#[cfg(not(feature = "s3"))]
pub use stub::*;

#[cfg(all(test, feature = "s3"))]
mod tests {
    use super::*;

    #[test]
    fn test_gcs_cache_config_builder() {
        let config = GcsCacheConfig::new("my-bucket")
            .service_account_path("/path/to/key.json")
            .prefix("cache/");

        assert_eq!(config.base.bucket, "my-bucket");
        assert_eq!(
            config.service_account_path,
            Some("/path/to/key.json".to_string())
        );
        assert_eq!(config.base.prefix, "cache/");
    }
}
