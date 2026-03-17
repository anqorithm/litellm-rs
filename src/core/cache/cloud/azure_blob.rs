//! Azure Blob Storage cache backend implementation
//!
//! Provides a cache backend using Azure Blob Storage for persistent, distributed caching.

#[cfg(feature = "s3")]
mod implementation {
    use async_trait::async_trait;
    use object_store::ObjectStore;
    use object_store::azure::{AzureConfigKey, MicrosoftAzureBuilder};
    use object_store::path::Path;
    use serde::{Serialize, de::DeserializeOwned};
    use std::sync::Arc;
    use std::time::Duration;
    use tracing::{debug, error, warn};

    use crate::core::cache::cloud::{CacheMetadata, CloudCache, CloudCacheConfig};
    use crate::core::cache::types::CacheKey;
    use crate::utils::error::gateway_error::{GatewayError, Result};

    /// Azure Blob cache configuration
    #[derive(Debug, Clone, Default)]
    pub struct AzureBlobCacheConfig {
        /// Base cloud cache config (bucket = container name)
        pub base: CloudCacheConfig,
        /// Azure storage account name
        pub account_name: Option<String>,
        /// Azure storage account key
        pub account_key: Option<String>,
        /// Azure SAS token
        pub sas_token: Option<String>,
        /// Use Azure CLI credentials
        pub use_cli_credentials: bool,
    }

    impl AzureBlobCacheConfig {
        /// Create a new Azure Blob cache configuration
        pub fn new(container: impl Into<String>) -> Self {
            Self {
                base: CloudCacheConfig::new(container),
                ..Default::default()
            }
        }

        /// Set the storage account name
        pub fn account_name(mut self, name: impl Into<String>) -> Self {
            self.account_name = Some(name.into());
            self
        }

        /// Set the storage account key
        pub fn account_key(mut self, key: impl Into<String>) -> Self {
            self.account_key = Some(key.into());
            self
        }

        /// Set the SAS token
        pub fn sas_token(mut self, token: impl Into<String>) -> Self {
            self.sas_token = Some(token.into());
            self
        }

        /// Use Azure CLI credentials for authentication
        pub fn use_cli_credentials(mut self, use_cli: bool) -> Self {
            self.use_cli_credentials = use_cli;
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
            let container = std::env::var("AZURE_CACHE_CONTAINER").ok()?;
            Some(Self {
                base: CloudCacheConfig::new(container),
                account_name: std::env::var("AZURE_STORAGE_ACCOUNT").ok(),
                account_key: std::env::var("AZURE_STORAGE_KEY").ok(),
                sas_token: std::env::var("AZURE_STORAGE_SAS_TOKEN").ok(),
                use_cli_credentials: std::env::var("AZURE_USE_CLI_CREDENTIALS")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false),
            })
        }
    }

    /// Azure Blob Storage cache backend
    pub struct AzureBlobCache {
        store: Arc<dyn ObjectStore>,
        config: AzureBlobCacheConfig,
    }

    impl AzureBlobCache {
        /// Create a new Azure Blob cache
        pub async fn new(config: AzureBlobCacheConfig) -> Result<Self> {
            let mut builder = MicrosoftAzureBuilder::new().with_container_name(&config.base.bucket);

            // Configure authentication
            if let Some(ref account_name) = config.account_name {
                builder = builder.with_config(AzureConfigKey::AccountName, account_name);
            }
            if let Some(ref account_key) = config.account_key {
                builder = builder.with_config(AzureConfigKey::AccessKey, account_key);
            }
            if let Some(ref sas_token) = config.sas_token {
                builder = builder.with_config(AzureConfigKey::SasKey, sas_token);
            }
            if config.use_cli_credentials {
                builder = builder.with_config(AzureConfigKey::UseAzureCli, "true");
            }

            let store = builder.build().map_err(|e| {
                GatewayError::Internal(format!("Failed to create Azure Blob client: {}", e))
            })?;

            Ok(Self {
                store: Arc::new(store),
                config,
            })
        }

        /// Create from environment variables
        pub async fn from_env() -> Result<Self> {
            let config = AzureBlobCacheConfig::from_env().ok_or_else(|| {
                GatewayError::Config("AZURE_CACHE_CONTAINER not set in environment".to_string())
            })?;
            Self::new(config).await
        }

        /// Build the full Azure Blob path for a cache key
        fn build_path(&self, key: &CacheKey) -> Path {
            Path::from(format!("{}{}", self.config.base.prefix, key.as_str()))
        }

        /// Build the metadata path for a cache key
        fn build_metadata_path(&self, key: &CacheKey) -> Path {
            Path::from(format!("{}{}._meta", self.config.base.prefix, key.as_str()))
        }
    }

    #[async_trait]
    impl CloudCache for AzureBlobCache {
        async fn get<T: DeserializeOwned + Send>(&self, key: &CacheKey) -> Result<Option<T>> {
            let path = self.build_path(key);
            let meta_path = self.build_metadata_path(key);
            debug!(key = %path, "Reading from Azure Blob cache");

            // First check metadata for expiration
            match self.store.get(&meta_path).await {
                Ok(meta_result) => {
                    let meta_bytes = meta_result.bytes().await.map_err(|e| {
                        GatewayError::Internal(format!("Failed to read metadata: {}", e))
                    })?;

                    let metadata: CacheMetadata =
                        serde_json::from_slice(&meta_bytes).map_err(|e| {
                            GatewayError::Internal(format!("Failed to parse metadata: {}", e))
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
                        GatewayError::Internal(format!("Failed to read body: {}", e))
                    })?;

                    let value: T = serde_json::from_slice(&bytes).map_err(|e| {
                        GatewayError::Internal(format!("Failed to deserialize: {}", e))
                    })?;

                    debug!(key = %path, "Azure Blob cache hit");
                    Ok(Some(value))
                }
                Err(object_store::Error::NotFound { .. }) => {
                    debug!(key = %path, "Cache miss");
                    Ok(None)
                }
                Err(err) => {
                    error!(key = %path, error = %err, "Failed to read from Azure Blob");
                    Err(GatewayError::Internal(format!(
                        "Azure Blob read error: {}",
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
            debug!(key = %path, ttl_secs = ttl.as_secs(), "Writing to Azure Blob cache");

            // Serialize the value
            let bytes = serde_json::to_vec(value)
                .map_err(|e| GatewayError::Internal(format!("Failed to serialize: {}", e)))?;

            // Create metadata
            let metadata = CacheMetadata::new(ttl, bytes.len(), false);
            let meta_bytes = serde_json::to_vec(&metadata).map_err(|e| {
                GatewayError::Internal(format!("Failed to serialize metadata: {}", e))
            })?;

            // Write the value
            self.store.put(&path, bytes.into()).await.map_err(|e| {
                GatewayError::Internal(format!("Failed to write to Azure Blob: {}", e))
            })?;

            // Write metadata
            self.store
                .put(&meta_path, meta_bytes.into())
                .await
                .map_err(|e| GatewayError::Internal(format!("Failed to write metadata: {}", e)))?;

            debug!(key = %path, "Azure Blob cache write successful");
            Ok(())
        }

        async fn delete(&self, key: &CacheKey) -> Result<bool> {
            let path = self.build_path(key);
            let meta_path = self.build_metadata_path(key);
            debug!(key = %path, "Deleting from Azure Blob cache");

            let value_result = self.store.delete(&path).await;
            let meta_result = self.store.delete(&meta_path).await;

            let deleted = value_result.is_ok() || meta_result.is_ok();

            if deleted {
                debug!(key = %path, "Azure Blob cache delete successful");
            }

            Ok(deleted)
        }

        async fn exists(&self, key: &CacheKey) -> Result<bool> {
            let path = self.build_path(key);

            match self.store.head(&path).await {
                Ok(_) => Ok(true),
                Err(object_store::Error::NotFound { .. }) => Ok(false),
                Err(err) => Err(GatewayError::Internal(format!(
                    "Azure Blob head error: {}",
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
                        warn!(error = %err, "Error listing Azure Blob objects");
                    }
                }
            }

            Ok(keys)
        }

        async fn clear(&self) -> Result<()> {
            warn!("Clearing all Azure Blob cache entries");

            let keys = self.list_keys("").await?;

            for key in keys {
                let cache_key = CacheKey::new(key);
                let _ = self.delete(&cache_key).await;
            }

            Ok(())
        }

        fn name(&self) -> &'static str {
            "azure_blob"
        }
    }

    impl std::fmt::Debug for AzureBlobCache {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("AzureBlobCache")
                .field("container", &self.config.base.bucket)
                .field("prefix", &self.config.base.prefix)
                .field("account_name", &self.config.account_name)
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

    /// Azure Blob cache configuration (stub)
    #[derive(Debug, Clone, Default)]
    pub struct AzureBlobCacheConfig {
        pub container: String,
    }

    impl AzureBlobCacheConfig {
        pub fn new(container: impl Into<String>) -> Self {
            Self {
                container: container.into(),
            }
        }

        pub fn from_env() -> Option<Self> {
            None
        }
    }

    /// Azure Blob cache (stub)
    #[derive(Debug)]
    pub struct AzureBlobCache;

    impl AzureBlobCache {
        pub async fn new(_config: AzureBlobCacheConfig) -> Result<Self> {
            Err(GatewayError::Config(
                "Azure Blob cache support not enabled. Enable the 's3' feature.".to_string(),
            ))
        }

        pub async fn from_env() -> Result<Self> {
            Err(GatewayError::Config(
                "Azure Blob cache support not enabled. Enable the 's3' feature.".to_string(),
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
    fn test_azure_blob_cache_config_builder() {
        let config = AzureBlobCacheConfig::new("my-container")
            .account_name("myaccount")
            .account_key("mykey")
            .prefix("cache/");

        assert_eq!(config.base.bucket, "my-container");
        assert_eq!(config.account_name, Some("myaccount".to_string()));
        assert_eq!(config.account_key, Some("mykey".to_string()));
        assert_eq!(config.base.prefix, "cache/");
    }

    #[test]
    fn test_azure_blob_cache_config_sas_token() {
        let config =
            AzureBlobCacheConfig::new("my-container").sas_token("sv=2021-06-08&ss=b&srt=sco...");

        assert!(config.sas_token.is_some());
    }

    #[test]
    fn test_azure_blob_cache_config_cli_credentials() {
        let config = AzureBlobCacheConfig::new("my-container").use_cli_credentials(true);

        assert!(config.use_cli_credentials);
    }
}
