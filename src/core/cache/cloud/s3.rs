//! AWS S3 cache backend implementation
//!
//! Provides a cache backend using AWS S3 for persistent, distributed caching.

#[cfg(feature = "s3")]
mod implementation {
    use async_trait::async_trait;
    use aws_sdk_s3::Client;
    use aws_sdk_s3::primitives::ByteStream;
    use aws_sdk_s3::types::Object as S3Object;
    use serde::{Serialize, de::DeserializeOwned};
    use std::time::Duration;
    use tracing::{debug, error, warn};

    use crate::core::cache::cloud::{CacheMetadata, CloudCache, CloudCacheConfig};
    use crate::core::cache::types::CacheKey;
    use crate::utils::error::gateway_error::{GatewayError, Result};

    /// S3 cache configuration
    #[derive(Debug, Clone, Default)]
    pub struct S3CacheConfig {
        /// Base cloud cache config
        pub base: CloudCacheConfig,
        /// AWS region
        pub region: Option<String>,
        /// Custom endpoint URL (for LocalStack, MinIO, etc.)
        pub endpoint_url: Option<String>,
        /// Storage class for cache objects
        pub storage_class: S3StorageClass,
    }

    /// S3 storage class options
    #[derive(Debug, Clone, Copy, Default)]
    pub enum S3StorageClass {
        #[default]
        Standard,
        StandardIa,
        OnezoneIa,
        IntelligentTiering,
        Glacier,
        GlacierIr,
    }

    impl S3StorageClass {
        /// Get the S3 storage class string
        pub fn as_str(&self) -> &'static str {
            match self {
                S3StorageClass::Standard => "STANDARD",
                S3StorageClass::StandardIa => "STANDARD_IA",
                S3StorageClass::OnezoneIa => "ONEZONE_IA",
                S3StorageClass::IntelligentTiering => "INTELLIGENT_TIERING",
                S3StorageClass::Glacier => "GLACIER",
                S3StorageClass::GlacierIr => "GLACIER_IR",
            }
        }
    }

    impl S3CacheConfig {
        /// Create a new S3 cache configuration
        pub fn new(bucket: impl Into<String>) -> Self {
            Self {
                base: CloudCacheConfig::new(bucket),
                ..Default::default()
            }
        }

        /// Set the AWS region
        pub fn region(mut self, region: impl Into<String>) -> Self {
            self.region = Some(region.into());
            self
        }

        /// Set a custom endpoint URL
        pub fn endpoint_url(mut self, url: impl Into<String>) -> Self {
            self.endpoint_url = Some(url.into());
            self
        }

        /// Set the storage class
        pub fn storage_class(mut self, class: S3StorageClass) -> Self {
            self.storage_class = class;
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
            let bucket = std::env::var("S3_CACHE_BUCKET").ok()?;
            Some(Self {
                base: CloudCacheConfig::new(bucket),
                region: std::env::var("AWS_REGION").ok(),
                endpoint_url: std::env::var("S3_ENDPOINT_URL").ok(),
                storage_class: S3StorageClass::default(),
            })
        }
    }

    /// AWS S3 cache backend
    pub struct S3Cache {
        client: Client,
        config: S3CacheConfig,
    }

    impl S3Cache {
        /// Create a new S3 cache
        pub async fn new(config: S3CacheConfig) -> Result<Self> {
            let mut aws_config_builder =
                aws_config::defaults(aws_config::BehaviorVersion::latest());

            if let Some(ref region) = config.region {
                aws_config_builder =
                    aws_config_builder.region(aws_config::Region::new(region.clone()));
            }

            let aws_config = aws_config_builder.load().await;

            let mut s3_config = aws_sdk_s3::config::Builder::from(&aws_config);

            if let Some(ref endpoint) = config.endpoint_url {
                s3_config = s3_config.endpoint_url(endpoint).force_path_style(true);
            }

            let client = Client::from_conf(s3_config.build());

            Ok(Self { client, config })
        }

        /// Create from environment variables
        pub async fn from_env() -> Result<Self> {
            let config = S3CacheConfig::from_env().ok_or_else(|| {
                GatewayError::Config("S3_CACHE_BUCKET not set in environment".to_string())
            })?;
            Self::new(config).await
        }

        /// Build the full S3 key for a cache key
        fn build_key(&self, key: &CacheKey) -> String {
            format!("{}{}", self.config.base.prefix, key.as_str())
        }

        /// Build the metadata key for a cache key
        fn build_metadata_key(&self, key: &CacheKey) -> String {
            format!("{}{}._meta", self.config.base.prefix, key.as_str())
        }
    }

    #[async_trait]
    impl CloudCache for S3Cache {
        async fn get<T: DeserializeOwned + Send>(&self, key: &CacheKey) -> Result<Option<T>> {
            let s3_key = self.build_key(key);
            let meta_key = self.build_metadata_key(key);
            debug!(key = %s3_key, "Reading from S3 cache");

            // First check metadata for expiration
            match self
                .client
                .get_object()
                .bucket(&self.config.base.bucket)
                .key(&meta_key)
                .send()
                .await
            {
                Ok(meta_response) => {
                    let meta_bytes = meta_response
                        .body
                        .collect()
                        .await
                        .map_err(|e| {
                            GatewayError::FileStorage(format!("Failed to read metadata: {}", e))
                        })?
                        .into_bytes();

                    let metadata: CacheMetadata =
                        serde_json::from_slice(&meta_bytes).map_err(|e| {
                            GatewayError::FileStorage(format!("Failed to parse metadata: {}", e))
                        })?;

                    if metadata.is_expired() {
                        debug!(key = %s3_key, "Cache entry expired");
                        // Clean up expired entry in background
                        let _ = self.delete(key).await;
                        return Ok(None);
                    }
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("NoSuchKey") || err_str.contains("404") {
                        debug!(key = %s3_key, "Cache miss - no metadata");
                        return Ok(None);
                    }
                    warn!(key = %s3_key, error = %err, "Failed to read metadata");
                    return Ok(None);
                }
            }

            // Read the actual value
            match self
                .client
                .get_object()
                .bucket(&self.config.base.bucket)
                .key(&s3_key)
                .send()
                .await
            {
                Ok(response) => {
                    let bytes = response
                        .body
                        .collect()
                        .await
                        .map_err(|e| {
                            GatewayError::FileStorage(format!("Failed to read body: {}", e))
                        })?
                        .into_bytes();

                    let value: T = serde_json::from_slice(&bytes).map_err(|e| {
                        GatewayError::FileStorage(format!("Failed to deserialize: {}", e))
                    })?;

                    debug!(key = %s3_key, "S3 cache hit");
                    Ok(Some(value))
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("NoSuchKey") || err_str.contains("404") {
                        debug!(key = %s3_key, "Cache miss");
                        Ok(None)
                    } else {
                        error!(key = %s3_key, error = %err, "Failed to read from S3");
                        Err(GatewayError::FileStorage(format!("S3 read error: {}", err)))
                    }
                }
            }
        }

        async fn set<T: Serialize + Send + Sync>(
            &self,
            key: &CacheKey,
            value: &T,
            ttl: Duration,
        ) -> Result<()> {
            let s3_key = self.build_key(key);
            let meta_key = self.build_metadata_key(key);
            debug!(key = %s3_key, ttl_secs = ttl.as_secs(), "Writing to S3 cache");

            // Serialize the value
            let bytes = serde_json::to_vec(value)
                .map_err(|e| GatewayError::FileStorage(format!("Failed to serialize: {}", e)))?;

            // Create metadata
            let metadata = CacheMetadata::new(ttl, bytes.len(), false);
            let meta_bytes = serde_json::to_vec(&metadata).map_err(|e| {
                GatewayError::FileStorage(format!("Failed to serialize metadata: {}", e))
            })?;

            // Write the value
            self.client
                .put_object()
                .bucket(&self.config.base.bucket)
                .key(&s3_key)
                .body(ByteStream::from(bytes))
                .storage_class(self.config.storage_class.as_str().parse().map_err(|e| {
                    GatewayError::Config(format!("Invalid S3 storage class: {}", e))
                })?)
                .content_type("application/json")
                .send()
                .await
                .map_err(|e| GatewayError::FileStorage(format!("Failed to write to S3: {}", e)))?;

            // Write metadata
            self.client
                .put_object()
                .bucket(&self.config.base.bucket)
                .key(&meta_key)
                .body(ByteStream::from(meta_bytes))
                .content_type("application/json")
                .send()
                .await
                .map_err(|e| {
                    GatewayError::FileStorage(format!("Failed to write metadata: {}", e))
                })?;

            debug!(key = %s3_key, "S3 cache write successful");
            Ok(())
        }

        async fn delete(&self, key: &CacheKey) -> Result<bool> {
            let s3_key = self.build_key(key);
            let meta_key = self.build_metadata_key(key);
            debug!(key = %s3_key, "Deleting from S3 cache");

            // Delete both the value and metadata
            let value_result = self
                .client
                .delete_object()
                .bucket(&self.config.base.bucket)
                .key(&s3_key)
                .send()
                .await;

            let meta_result = self
                .client
                .delete_object()
                .bucket(&self.config.base.bucket)
                .key(&meta_key)
                .send()
                .await;

            // S3 delete doesn't error if key doesn't exist
            if value_result.is_err() && meta_result.is_err() {
                return Ok(false);
            }

            debug!(key = %s3_key, "S3 cache delete successful");
            Ok(true)
        }

        async fn exists(&self, key: &CacheKey) -> Result<bool> {
            let s3_key = self.build_key(key);

            match self
                .client
                .head_object()
                .bucket(&self.config.base.bucket)
                .key(&s3_key)
                .send()
                .await
            {
                Ok(_) => Ok(true),
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("NotFound") || err_str.contains("404") {
                        Ok(false)
                    } else {
                        Err(GatewayError::FileStorage(format!("S3 head error: {}", err)))
                    }
                }
            }
        }

        async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
            let full_prefix = format!("{}{}", self.config.base.prefix, prefix);

            let response = self
                .client
                .list_objects_v2()
                .bucket(&self.config.base.bucket)
                .prefix(&full_prefix)
                .send()
                .await
                .map_err(|e| GatewayError::FileStorage(format!("Failed to list objects: {}", e)))?;

            let mut keys = Vec::new();
            for obj in response.contents().iter() {
                let obj: &S3Object = obj;
                if let Some(k) = obj.key() {
                    let key = k
                        .strip_prefix(&self.config.base.prefix)
                        .unwrap_or(k)
                        .to_string();
                    if !key.ends_with("._meta") {
                        keys.push(key);
                    }
                }
            }

            Ok(keys)
        }

        async fn clear(&self) -> Result<()> {
            warn!("Clearing all S3 cache entries");

            let keys = self.list_keys("").await?;

            for key in keys {
                let cache_key = CacheKey::new(key);
                let _ = self.delete(&cache_key).await;
            }

            Ok(())
        }

        fn name(&self) -> &'static str {
            "s3"
        }
    }

    impl std::fmt::Debug for S3Cache {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("S3Cache")
                .field("bucket", &self.config.base.bucket)
                .field("prefix", &self.config.base.prefix)
                .field("region", &self.config.region)
                .finish()
        }
    }
}

#[cfg(feature = "s3")]
pub use implementation::*;

// Stub implementation when feature is disabled
#[cfg(not(feature = "s3"))]
mod stub {
    use crate::utils::error::gateway_error::{GatewayError, Result};

    /// S3 cache configuration (stub)
    #[derive(Debug, Clone, Default)]
    pub struct S3CacheConfig {
        pub bucket: String,
    }

    impl S3CacheConfig {
        pub fn new(bucket: impl Into<String>) -> Self {
            Self {
                bucket: bucket.into(),
            }
        }

        pub fn from_env() -> Option<Self> {
            None
        }
    }

    /// S3 cache (stub)
    #[derive(Debug)]
    pub struct S3Cache;

    impl S3Cache {
        pub async fn new(_config: S3CacheConfig) -> Result<Self> {
            Err(GatewayError::Config(
                "S3 cache support not enabled. Enable the 's3' feature.".to_string(),
            ))
        }

        pub async fn from_env() -> Result<Self> {
            Err(GatewayError::Config(
                "S3 cache support not enabled. Enable the 's3' feature.".to_string(),
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
    fn test_s3_cache_config_builder() {
        let config = S3CacheConfig::new("my-bucket")
            .region("us-east-1")
            .prefix("cache/")
            .storage_class(S3StorageClass::StandardIa);

        assert_eq!(config.base.bucket, "my-bucket");
        assert_eq!(config.region, Some("us-east-1".to_string()));
        assert_eq!(config.base.prefix, "cache/");
    }

    #[test]
    fn test_s3_storage_class() {
        assert_eq!(S3StorageClass::Standard.as_str(), "STANDARD");
        assert_eq!(S3StorageClass::StandardIa.as_str(), "STANDARD_IA");
        assert_eq!(S3StorageClass::Glacier.as_str(), "GLACIER");
    }
}
