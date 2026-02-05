//! HashiCorp Vault secret manager implementation
//!
//! Provides integration with HashiCorp Vault for secure secret storage.

#[cfg(feature = "vault-secrets")]
mod implementation {
    use async_trait::async_trait;
    use std::collections::HashMap;
    use tracing::{debug, error, warn};
    use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
    use vaultrs::kv2;

    use crate::core::traits::secret_manager::{
        ListSecretsOptions, ListSecretsResult, SecretError, SecretManager, SecretMetadata,
        SecretResult,
    };

    /// HashiCorp Vault configuration
    #[derive(Debug, Clone)]
    pub struct VaultConfig {
        /// Vault server address
        pub address: String,
        /// Authentication token
        pub token: Option<String>,
        /// KV secrets engine mount path
        pub mount: String,
        /// Secret path prefix
        pub prefix: Option<String>,
        /// Namespace (for Vault Enterprise)
        pub namespace: Option<String>,
        /// Skip TLS verification (not recommended for production)
        pub skip_tls_verify: bool,
    }

    impl Default for VaultConfig {
        fn default() -> Self {
            Self {
                address: "http://127.0.0.1:8200".to_string(),
                token: None,
                mount: "secret".to_string(),
                prefix: None,
                namespace: None,
                skip_tls_verify: false,
            }
        }
    }

    impl VaultConfig {
        pub fn new(address: impl Into<String>) -> Self {
            Self {
                address: address.into(),
                ..Default::default()
            }
        }

        pub fn token(mut self, token: impl Into<String>) -> Self {
            self.token = Some(token.into());
            self
        }

        pub fn mount(mut self, mount: impl Into<String>) -> Self {
            self.mount = mount.into();
            self
        }

        pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
            self.prefix = Some(prefix.into());
            self
        }

        pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
            self.namespace = Some(namespace.into());
            self
        }

        pub fn skip_tls_verify(mut self, skip: bool) -> Self {
            self.skip_tls_verify = skip;
            self
        }

        /// Create from environment variables
        pub fn from_env() -> Self {
            Self {
                address: std::env::var("VAULT_ADDR")
                    .unwrap_or_else(|_| "http://127.0.0.1:8200".to_string()),
                token: std::env::var("VAULT_TOKEN").ok(),
                mount: std::env::var("VAULT_MOUNT").unwrap_or_else(|_| "secret".to_string()),
                prefix: std::env::var("VAULT_PREFIX").ok(),
                namespace: std::env::var("VAULT_NAMESPACE").ok(),
                skip_tls_verify: std::env::var("VAULT_SKIP_VERIFY")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false),
            }
        }
    }

    /// HashiCorp Vault secret manager
    pub struct VaultSecretManager {
        client: VaultClient,
        config: VaultConfig,
    }

    impl VaultSecretManager {
        /// Create a new Vault secret manager
        pub fn new(config: VaultConfig) -> SecretResult<Self> {
            let token = config
                .token
                .clone()
                .ok_or_else(|| SecretError::auth("Vault token is required"))?;

            let mut settings_builder = VaultClientSettingsBuilder::default();
            settings_builder.address(&config.address);
            settings_builder.token(&token);

            if let Some(ref ns) = config.namespace {
                settings_builder.namespace(Some(ns.clone()));
            }

            // Note: vaultrs handles TLS verification through its own settings
            let settings = settings_builder.build().map_err(|e| {
                SecretError::config(format!("Failed to build Vault client settings: {}", e))
            })?;

            let client = VaultClient::new(settings).map_err(|e| {
                SecretError::connection(format!("Failed to create Vault client: {}", e))
            })?;

            Ok(Self { client, config })
        }

        /// Create from environment variables
        pub fn from_env() -> SecretResult<Self> {
            Self::new(VaultConfig::from_env())
        }

        /// Get the full secret path with prefix
        fn get_secret_path(&self, name: &str) -> String {
            match &self.config.prefix {
                Some(prefix) => format!("{}{}", prefix, name),
                None => name.to_string(),
            }
        }
    }

    #[async_trait]
    impl SecretManager for VaultSecretManager {
        fn name(&self) -> &'static str {
            "vault"
        }

        async fn read_secret(&self, name: &str) -> SecretResult<Option<String>> {
            let path = self.get_secret_path(name);
            debug!(path = %path, mount = %self.config.mount, "Reading secret from Vault");

            match kv2::read::<HashMap<String, String>>(&self.client, &self.config.mount, &path)
                .await
            {
                Ok(secret_data) => {
                    // Try to get "value" key first, then fall back to first value
                    let value = secret_data
                        .get("value")
                        .or_else(|| secret_data.values().next())
                        .cloned();
                    Ok(value)
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("404") || err_str.contains("not found") {
                        debug!(path = %path, "Secret not found in Vault");
                        Ok(None)
                    } else if err_str.contains("403") || err_str.contains("permission denied") {
                        warn!(path = %path, "Access denied to Vault secret");
                        Err(SecretError::access_denied(&path))
                    } else if err_str.contains("401") || err_str.contains("invalid token") {
                        error!(path = %path, "Vault authentication failed");
                        Err(SecretError::auth(err_str))
                    } else {
                        error!(path = %path, error = %err, "Failed to read secret from Vault");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }

        async fn write_secret(&self, name: &str, value: &str) -> SecretResult<()> {
            let path = self.get_secret_path(name);
            debug!(path = %path, mount = %self.config.mount, "Writing secret to Vault");

            let mut data = HashMap::new();
            data.insert("value".to_string(), value.to_string());

            match kv2::set(&self.client, &self.config.mount, &path, &data).await {
                Ok(_) => {
                    debug!(path = %path, "Secret written to Vault successfully");
                    Ok(())
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("403") || err_str.contains("permission denied") {
                        Err(SecretError::access_denied(&path))
                    } else if err_str.contains("401") || err_str.contains("invalid token") {
                        Err(SecretError::auth(err_str))
                    } else {
                        error!(path = %path, error = %err, "Failed to write secret to Vault");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }

        async fn delete_secret(&self, name: &str) -> SecretResult<()> {
            let path = self.get_secret_path(name);
            debug!(path = %path, mount = %self.config.mount, "Deleting secret from Vault");

            match kv2::delete_latest(&self.client, &self.config.mount, &path).await {
                Ok(_) => {
                    debug!(path = %path, "Secret deleted from Vault successfully");
                    Ok(())
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("404") || err_str.contains("not found") {
                        // Already deleted
                        Ok(())
                    } else if err_str.contains("403") || err_str.contains("permission denied") {
                        Err(SecretError::access_denied(&path))
                    } else if err_str.contains("401") || err_str.contains("invalid token") {
                        Err(SecretError::auth(err_str))
                    } else {
                        error!(path = %path, error = %err, "Failed to delete secret from Vault");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }

        async fn list_secrets(
            &self,
            options: &ListSecretsOptions,
        ) -> SecretResult<ListSecretsResult> {
            let path = match (&self.config.prefix, &options.prefix) {
                (Some(config_prefix), Some(opt_prefix)) => {
                    format!("{}{}", config_prefix, opt_prefix)
                }
                (Some(config_prefix), None) => config_prefix.clone(),
                (None, Some(opt_prefix)) => opt_prefix.clone(),
                (None, None) => String::new(),
            };

            debug!(path = %path, mount = %self.config.mount, "Listing secrets from Vault");

            match kv2::list(&self.client, &self.config.mount, &path).await {
                Ok(keys) => {
                    let mut secrets: Vec<SecretMetadata> =
                        keys.into_iter().map(SecretMetadata::new).collect();

                    // Apply max_results limit
                    if let Some(max) = options.max_results {
                        secrets.truncate(max);
                    }

                    Ok(ListSecretsResult {
                        secrets,
                        next_token: None, // Vault KV2 doesn't support pagination
                    })
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("404") || err_str.contains("not found") {
                        // Empty path
                        Ok(ListSecretsResult {
                            secrets: vec![],
                            next_token: None,
                        })
                    } else if err_str.contains("403") || err_str.contains("permission denied") {
                        Err(SecretError::access_denied(&path))
                    } else if err_str.contains("401") || err_str.contains("invalid token") {
                        Err(SecretError::auth(err_str))
                    } else {
                        error!(path = %path, error = %err, "Failed to list secrets from Vault");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }
    }

    impl std::fmt::Debug for VaultSecretManager {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("VaultSecretManager")
                .field("config", &self.config)
                .finish()
        }
    }
}

#[cfg(feature = "vault-secrets")]
pub use implementation::*;

// Stub implementation when feature is disabled
#[cfg(not(feature = "vault-secrets"))]
mod stub {
    use crate::core::traits::secret_manager::SecretError;

    /// HashiCorp Vault configuration (stub)
    #[derive(Debug, Clone, Default)]
    pub struct VaultConfig {
        pub address: String,
    }

    impl VaultConfig {
        pub fn new(address: impl Into<String>) -> Self {
            Self {
                address: address.into(),
            }
        }

        pub fn from_env() -> Self {
            Self::default()
        }
    }

    /// HashiCorp Vault secret manager (stub)
    #[derive(Debug)]
    pub struct VaultSecretManager;

    impl VaultSecretManager {
        pub fn new(_config: VaultConfig) -> Result<Self, SecretError> {
            Err(SecretError::config(
                "HashiCorp Vault support not enabled. Enable the 'vault-secrets' feature.",
            ))
        }

        pub fn from_env() -> Result<Self, SecretError> {
            Self::new(VaultConfig::default())
        }
    }
}

#[cfg(not(feature = "vault-secrets"))]
pub use stub::*;

#[cfg(all(test, feature = "vault-secrets"))]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = VaultConfig::new("https://vault.example.com:8200")
            .token("s.abcdef123456")
            .mount("kv")
            .prefix("prod/")
            .namespace("my-namespace")
            .skip_tls_verify(false);

        assert_eq!(config.address, "https://vault.example.com:8200");
        assert_eq!(config.token, Some("s.abcdef123456".to_string()));
        assert_eq!(config.mount, "kv");
        assert_eq!(config.prefix, Some("prod/".to_string()));
        assert_eq!(config.namespace, Some("my-namespace".to_string()));
        assert!(!config.skip_tls_verify);
    }

    #[test]
    fn test_config_default() {
        let config = VaultConfig::default();
        assert_eq!(config.address, "http://127.0.0.1:8200");
        assert_eq!(config.mount, "secret");
        assert!(config.token.is_none());
        assert!(config.prefix.is_none());
    }
}
