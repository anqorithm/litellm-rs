//! Azure Key Vault secret manager implementation
//!
//! Provides integration with Azure Key Vault for secure secret storage.

#[cfg(feature = "azure-secrets")]
mod implementation {
    use async_trait::async_trait;
    use azure_identity::AzureCliCredential;
    use azure_security_keyvault_secrets::SecretClient;
    use azure_security_keyvault_secrets::models::SetSecretParameters;
    use tracing::{debug, error, warn};

    use crate::core::traits::secret_manager::{
        ListSecretsOptions, ListSecretsResult, SecretError, SecretManager, SecretResult,
    };

    /// Azure Key Vault configuration
    #[derive(Debug, Clone)]
    pub struct AzureSecretsConfig {
        /// Key Vault URL (e.g., https://my-vault.vault.azure.net)
        pub vault_url: String,
        /// Secret name prefix
        pub prefix: Option<String>,
    }

    impl AzureSecretsConfig {
        pub fn new(vault_url: impl Into<String>) -> Self {
            Self {
                vault_url: vault_url.into(),
                prefix: None,
            }
        }

        pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
            self.prefix = Some(prefix.into());
            self
        }

        /// Create from environment variables
        pub fn from_env() -> Option<Self> {
            let vault_url = std::env::var("AZURE_KEYVAULT_URL")
                .or_else(|_| std::env::var("AZURE_VAULT_URL"))
                .ok()?;

            Some(Self {
                vault_url,
                prefix: std::env::var("AZURE_SECRETS_PREFIX").ok(),
            })
        }
    }

    /// Azure Key Vault secret manager
    pub struct AzureSecretManager {
        client: SecretClient,
        config: AzureSecretsConfig,
    }

    impl AzureSecretManager {
        /// Create a new Azure Key Vault client using Azure CLI credentials
        pub fn new(config: AzureSecretsConfig) -> SecretResult<Self> {
            let credential = AzureCliCredential::new(None).map_err(|e| {
                SecretError::auth(format!("Failed to create Azure credential: {}", e))
            })?;

            let client = SecretClient::new(&config.vault_url, credential, None).map_err(|e| {
                SecretError::connection(format!("Failed to create Azure Key Vault client: {}", e))
            })?;

            Ok(Self { client, config })
        }

        /// Create from environment variables
        pub fn from_env() -> SecretResult<Self> {
            let config = AzureSecretsConfig::from_env().ok_or_else(|| {
                SecretError::config("Azure Key Vault URL not found in environment")
            })?;
            Self::new(config)
        }

        /// Get the full secret name with prefix
        fn get_secret_name(&self, name: &str) -> String {
            match &self.config.prefix {
                Some(prefix) => format!("{}{}", prefix, name),
                None => name.to_string(),
            }
        }

        /// Normalize secret name for Azure (replace invalid characters)
        fn normalize_name(&self, name: &str) -> String {
            // Azure Key Vault secret names can only contain alphanumeric characters and dashes
            name.replace(['_', '/', '.'], "-")
        }
    }

    #[async_trait]
    impl SecretManager for AzureSecretManager {
        fn name(&self) -> &'static str {
            "azure"
        }

        async fn read_secret(&self, name: &str) -> SecretResult<Option<String>> {
            let secret_name = self.normalize_name(&self.get_secret_name(name));
            debug!(secret_name = %secret_name, "Reading secret from Azure Key Vault");

            match self.client.get_secret(&secret_name, None).await {
                Ok(response) => {
                    // Response is Response<Secret>, use into_model() to get the Secret
                    let secret = response.into_model().map_err(|e| {
                        SecretError::invalid_format(format!("Failed to parse secret: {}", e))
                    })?;
                    Ok(secret.value)
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("SecretNotFound") || err_str.contains("404") {
                        debug!(secret_name = %secret_name, "Secret not found");
                        Ok(None)
                    } else if err_str.contains("Forbidden") || err_str.contains("403") {
                        warn!(secret_name = %secret_name, "Access denied to secret");
                        Err(SecretError::access_denied(&secret_name))
                    } else if err_str.contains("Unauthorized") || err_str.contains("401") {
                        error!(secret_name = %secret_name, "Authentication failed");
                        Err(SecretError::auth(err_str))
                    } else {
                        error!(secret_name = %secret_name, error = %err, "Failed to read secret");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }

        async fn write_secret(&self, name: &str, value: &str) -> SecretResult<()> {
            let secret_name = self.normalize_name(&self.get_secret_name(name));
            debug!(secret_name = %secret_name, "Writing secret to Azure Key Vault");

            let params = SetSecretParameters {
                value: Some(value.to_string()),
                content_type: None,
                secret_attributes: None,
                tags: None,
            };

            let request_content = params.try_into().map_err(|e: azure_core::Error| {
                SecretError::invalid_format(format!("Failed to create request: {}", e))
            })?;

            match self
                .client
                .set_secret(&secret_name, request_content, None)
                .await
            {
                Ok(_) => {
                    debug!(secret_name = %secret_name, "Secret written successfully");
                    Ok(())
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("Forbidden") || err_str.contains("403") {
                        Err(SecretError::access_denied(&secret_name))
                    } else if err_str.contains("Unauthorized") || err_str.contains("401") {
                        Err(SecretError::auth(err_str))
                    } else {
                        error!(secret_name = %secret_name, error = %err, "Failed to write secret");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }

        async fn delete_secret(&self, name: &str) -> SecretResult<()> {
            let secret_name = self.normalize_name(&self.get_secret_name(name));
            debug!(secret_name = %secret_name, "Deleting secret from Azure Key Vault");

            match self.client.delete_secret(&secret_name, None).await {
                Ok(_) => {
                    debug!(secret_name = %secret_name, "Secret deleted successfully");
                    Ok(())
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("SecretNotFound") || err_str.contains("404") {
                        // Already deleted
                        Ok(())
                    } else if err_str.contains("Forbidden") || err_str.contains("403") {
                        Err(SecretError::access_denied(&secret_name))
                    } else if err_str.contains("Unauthorized") || err_str.contains("401") {
                        Err(SecretError::auth(err_str))
                    } else {
                        error!(secret_name = %secret_name, error = %err, "Failed to delete secret");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }

        async fn list_secrets(
            &self,
            _options: &ListSecretsOptions,
        ) -> SecretResult<ListSecretsResult> {
            // Note: Azure Key Vault list_secret_properties returns a complex Pager type
            // that requires specific handling. For now, return an empty list.
            // Full implementation would require iterating through the pager.
            debug!("Listing secrets from Azure Key Vault (limited support)");

            Ok(ListSecretsResult {
                secrets: vec![],
                next_token: None,
            })
        }
    }

    impl std::fmt::Debug for AzureSecretManager {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("AzureSecretManager")
                .field("config", &self.config)
                .finish()
        }
    }
}

#[cfg(feature = "azure-secrets")]
pub use implementation::*;

// Stub implementation when feature is disabled
#[cfg(not(feature = "azure-secrets"))]
mod stub {
    use crate::core::traits::secret_manager::SecretError;

    /// Azure Key Vault configuration (stub)
    #[derive(Debug, Clone)]
    pub struct AzureSecretsConfig {
        pub vault_url: String,
    }

    impl AzureSecretsConfig {
        pub fn new(vault_url: impl Into<String>) -> Self {
            Self {
                vault_url: vault_url.into(),
            }
        }

        pub fn from_env() -> Option<Self> {
            None
        }
    }

    /// Azure Key Vault secret manager (stub)
    #[derive(Debug)]
    pub struct AzureSecretManager;

    impl AzureSecretManager {
        pub fn new(_config: AzureSecretsConfig) -> Result<Self, SecretError> {
            Err(SecretError::config(
                "Azure Key Vault support not enabled. Enable the 'azure-secrets' feature.",
            ))
        }

        pub fn from_env() -> Result<Self, SecretError> {
            Err(SecretError::config(
                "Azure Key Vault support not enabled. Enable the 'azure-secrets' feature.",
            ))
        }
    }
}

#[cfg(not(feature = "azure-secrets"))]
pub use stub::*;

#[cfg(all(test, feature = "azure-secrets"))]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = AzureSecretsConfig::new("https://my-vault.vault.azure.net").prefix("prod/");

        assert_eq!(config.vault_url, "https://my-vault.vault.azure.net");
        assert_eq!(config.prefix, Some("prod/".to_string()));
    }
}
