//! Google Cloud Secret Manager implementation
//!
//! Provides integration with Google Cloud Secret Manager for secure secret storage.

#[cfg(feature = "gcp-secrets")]
mod implementation {
    use async_trait::async_trait;
    use google_cloud_secretmanager_v1::client::SecretManagerService;
    use tracing::{debug, error, warn};

    use crate::core::traits::secret_manager::{
        ListSecretsOptions, ListSecretsResult, SecretError, SecretManager, SecretMetadata,
        SecretResult,
    };

    /// Google Cloud Secret Manager configuration
    #[derive(Debug, Clone)]
    pub struct GcpSecretsConfig {
        /// GCP project ID
        pub project_id: String,
        /// Secret name prefix
        pub prefix: Option<String>,
    }

    impl GcpSecretsConfig {
        pub fn new(project_id: impl Into<String>) -> Self {
            Self {
                project_id: project_id.into(),
                prefix: None,
            }
        }

        pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
            self.prefix = Some(prefix.into());
            self
        }

        /// Create from environment variables
        pub fn from_env() -> Option<Self> {
            let project_id = std::env::var("GOOGLE_CLOUD_PROJECT")
                .or_else(|_| std::env::var("GCP_PROJECT_ID"))
                .or_else(|_| std::env::var("GCLOUD_PROJECT"))
                .ok()?;

            Some(Self {
                project_id,
                prefix: std::env::var("GCP_SECRETS_PREFIX").ok(),
            })
        }
    }

    /// Google Cloud Secret Manager client
    pub struct GcpSecretManager {
        client: SecretManagerService,
        config: GcpSecretsConfig,
    }

    impl GcpSecretManager {
        /// Create a new GCP Secret Manager client
        pub async fn new(config: GcpSecretsConfig) -> SecretResult<Self> {
            let client = SecretManagerService::builder().build().await.map_err(|e| {
                SecretError::connection(format!("Failed to create GCP client: {}", e))
            })?;

            Ok(Self { client, config })
        }

        /// Create from environment variables
        pub async fn from_env() -> SecretResult<Self> {
            let config = GcpSecretsConfig::from_env()
                .ok_or_else(|| SecretError::config("GCP project ID not found in environment"))?;
            Self::new(config).await
        }

        /// Get the full secret name with prefix
        fn get_secret_name(&self, name: &str) -> String {
            match &self.config.prefix {
                Some(prefix) => format!("{}{}", prefix, name),
                None => name.to_string(),
            }
        }

        /// Build the full secret resource name
        fn build_secret_path(&self, name: &str) -> String {
            format!(
                "projects/{}/secrets/{}",
                self.config.project_id,
                self.get_secret_name(name)
            )
        }

        /// Build the full secret version resource name
        fn build_version_path(&self, name: &str, version: &str) -> String {
            format!(
                "projects/{}/secrets/{}/versions/{}",
                self.config.project_id,
                self.get_secret_name(name),
                version
            )
        }

        /// Build the parent path for listing secrets
        fn build_parent_path(&self) -> String {
            format!("projects/{}", self.config.project_id)
        }
    }

    #[async_trait]
    impl SecretManager for GcpSecretManager {
        fn name(&self) -> &'static str {
            "gcp"
        }

        async fn read_secret(&self, name: &str) -> SecretResult<Option<String>> {
            let version_path = self.build_version_path(name, "latest");
            debug!(path = %version_path, "Reading secret from GCP Secret Manager");

            match self
                .client
                .access_secret_version()
                .set_name(&version_path)
                .send()
                .await
            {
                Ok(response) => {
                    if let Some(payload) = response.payload {
                        match String::from_utf8(payload.data.to_vec()) {
                            Ok(value) => Ok(Some(value)),
                            Err(_) => Err(SecretError::invalid_format(
                                "Secret data is not valid UTF-8",
                            )),
                        }
                    } else {
                        Ok(None)
                    }
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("NOT_FOUND") || err_str.contains("404") {
                        debug!(path = %version_path, "Secret not found");
                        Ok(None)
                    } else if err_str.contains("PERMISSION_DENIED") || err_str.contains("403") {
                        warn!(path = %version_path, "Access denied to secret");
                        Err(SecretError::access_denied(name))
                    } else if err_str.contains("UNAUTHENTICATED") || err_str.contains("401") {
                        error!(path = %version_path, "Authentication failed");
                        Err(SecretError::auth(err_str))
                    } else {
                        error!(path = %version_path, error = %err, "Failed to read secret");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }

        async fn write_secret(&self, name: &str, value: &str) -> SecretResult<()> {
            use google_cloud_secretmanager_v1::model::{
                Replication, Secret, SecretPayload, replication,
            };

            let secret_path = self.build_secret_path(name);
            debug!(path = %secret_path, "Writing secret to GCP Secret Manager");

            // First, try to add a new version to existing secret
            match self
                .client
                .add_secret_version()
                .set_parent(&secret_path)
                .set_payload(SecretPayload::new().set_data(value.as_bytes().to_vec()))
                .send()
                .await
            {
                Ok(_) => {
                    debug!(path = %secret_path, "Secret version added successfully");
                    Ok(())
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("NOT_FOUND") || err_str.contains("404") {
                        // Secret doesn't exist, create it first
                        debug!(path = %secret_path, "Secret not found, creating new secret");

                        let parent = self.build_parent_path();
                        let secret_id = self.get_secret_name(name);

                        // Create the secret
                        self.client
                            .create_secret()
                            .set_parent(&parent)
                            .set_secret_id(&secret_id)
                            .set_secret(Secret::new().set_replication(
                                Replication::new().set_automatic(replication::Automatic::new()),
                            ))
                            .send()
                            .await
                            .map_err(|e| {
                                SecretError::connection(format!("Failed to create secret: {}", e))
                            })?;

                        // Now add the version
                        self.client
                            .add_secret_version()
                            .set_parent(&secret_path)
                            .set_payload(SecretPayload::new().set_data(value.as_bytes().to_vec()))
                            .send()
                            .await
                            .map_err(|e| {
                                SecretError::connection(format!(
                                    "Failed to add secret version: {}",
                                    e
                                ))
                            })?;

                        Ok(())
                    } else if err_str.contains("PERMISSION_DENIED") || err_str.contains("403") {
                        Err(SecretError::access_denied(name))
                    } else if err_str.contains("UNAUTHENTICATED") || err_str.contains("401") {
                        Err(SecretError::auth(err_str))
                    } else {
                        error!(path = %secret_path, error = %err, "Failed to write secret");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }

        async fn delete_secret(&self, name: &str) -> SecretResult<()> {
            let secret_path = self.build_secret_path(name);
            debug!(path = %secret_path, "Deleting secret from GCP Secret Manager");

            match self
                .client
                .delete_secret()
                .set_name(&secret_path)
                .send()
                .await
            {
                Ok(_) => {
                    debug!(path = %secret_path, "Secret deleted successfully");
                    Ok(())
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("NOT_FOUND") || err_str.contains("404") {
                        // Already deleted
                        Ok(())
                    } else if err_str.contains("PERMISSION_DENIED") || err_str.contains("403") {
                        Err(SecretError::access_denied(name))
                    } else if err_str.contains("UNAUTHENTICATED") || err_str.contains("401") {
                        Err(SecretError::auth(err_str))
                    } else {
                        error!(path = %secret_path, error = %err, "Failed to delete secret");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }

        async fn list_secrets(
            &self,
            options: &ListSecretsOptions,
        ) -> SecretResult<ListSecretsResult> {
            let parent = self.build_parent_path();
            debug!(parent = %parent, "Listing secrets from GCP Secret Manager");

            let mut request = self.client.list_secrets().set_parent(&parent);

            if let Some(max) = options.max_results {
                request = request.set_page_size(max as i32);
            }

            if let Some(ref token) = options.next_token {
                request = request.set_page_token(token);
            }

            match request.send().await {
                Ok(response) => {
                    let secrets = response
                        .secrets
                        .into_iter()
                        .filter_map(|s| {
                            // Extract secret name from full resource name
                            s.name.split('/').next_back().map(|name| {
                                let mut metadata = SecretMetadata::new(name);
                                if let Some(created) = s.create_time {
                                    metadata.created_at = Some(created.seconds());
                                }
                                metadata
                            })
                        })
                        .collect();

                    Ok(ListSecretsResult {
                        secrets,
                        next_token: if response.next_page_token.is_empty() {
                            None
                        } else {
                            Some(response.next_page_token)
                        },
                    })
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("PERMISSION_DENIED") || err_str.contains("403") {
                        Err(SecretError::access_denied("list"))
                    } else if err_str.contains("UNAUTHENTICATED") || err_str.contains("401") {
                        Err(SecretError::auth(err_str))
                    } else {
                        error!(parent = %self.build_parent_path(), error = %err, "Failed to list secrets");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }
    }

    impl std::fmt::Debug for GcpSecretManager {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("GcpSecretManager")
                .field("config", &self.config)
                .finish()
        }
    }
}

#[cfg(feature = "gcp-secrets")]
pub use implementation::*;

// Stub implementation when feature is disabled
#[cfg(not(feature = "gcp-secrets"))]
mod stub {
    use crate::core::traits::secret_manager::SecretError;

    /// Google Cloud Secret Manager configuration (stub)
    #[derive(Debug, Clone)]
    pub struct GcpSecretsConfig {
        pub project_id: String,
    }

    impl GcpSecretsConfig {
        pub fn new(project_id: impl Into<String>) -> Self {
            Self {
                project_id: project_id.into(),
            }
        }

        pub fn from_env() -> Option<Self> {
            None
        }
    }

    /// Google Cloud Secret Manager client (stub)
    #[derive(Debug)]
    pub struct GcpSecretManager;

    impl GcpSecretManager {
        pub async fn new(_config: GcpSecretsConfig) -> Result<Self, SecretError> {
            Err(SecretError::config(
                "GCP Secret Manager support not enabled. Enable the 'gcp-secrets' feature.",
            ))
        }

        pub async fn from_env() -> Result<Self, SecretError> {
            Err(SecretError::config(
                "GCP Secret Manager support not enabled. Enable the 'gcp-secrets' feature.",
            ))
        }
    }
}

#[cfg(not(feature = "gcp-secrets"))]
pub use stub::*;

#[cfg(all(test, feature = "gcp-secrets"))]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = GcpSecretsConfig::new("my-project-123").prefix("prod/");

        assert_eq!(config.project_id, "my-project-123");
        assert_eq!(config.prefix, Some("prod/".to_string()));
    }
}
