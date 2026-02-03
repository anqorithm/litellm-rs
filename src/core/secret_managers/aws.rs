//! AWS Secrets Manager implementation
//!
//! Provides integration with AWS Secrets Manager for secure secret storage.

#[cfg(feature = "aws-secrets")]
mod implementation {
    use async_trait::async_trait;
    use aws_config::BehaviorVersion;
    use aws_sdk_secretsmanager::Client;
    use tracing::{debug, error, warn};

    use crate::core::traits::secret_manager::{
        ListSecretsOptions, ListSecretsResult, SecretError, SecretManager, SecretMetadata,
        SecretResult,
    };

    /// AWS Secrets Manager configuration
    #[derive(Debug, Clone, Default)]
    pub struct AwsSecretsConfig {
        /// AWS region
        pub region: Option<String>,
        /// Secret name prefix
        pub prefix: Option<String>,
        /// Custom endpoint URL (for LocalStack, etc.)
        pub endpoint_url: Option<String>,
    }

    impl AwsSecretsConfig {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn region(mut self, region: impl Into<String>) -> Self {
            self.region = Some(region.into());
            self
        }

        pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
            self.prefix = Some(prefix.into());
            self
        }

        pub fn endpoint_url(mut self, url: impl Into<String>) -> Self {
            self.endpoint_url = Some(url.into());
            self
        }
    }

    /// AWS Secrets Manager client
    pub struct AwsSecretManager {
        client: Client,
        config: AwsSecretsConfig,
    }

    impl AwsSecretManager {
        /// Create a new AWS Secrets Manager client
        pub async fn new(config: AwsSecretsConfig) -> SecretResult<Self> {
            let mut aws_config_builder =
                aws_config::defaults(BehaviorVersion::latest());

            if let Some(ref region) = config.region {
                aws_config_builder =
                    aws_config_builder.region(aws_config::Region::new(region.clone()));
            }

            let aws_config = aws_config_builder.load().await;

            let mut client_config =
                aws_sdk_secretsmanager::config::Builder::from(&aws_config);

            if let Some(ref endpoint) = config.endpoint_url {
                client_config = client_config.endpoint_url(endpoint);
            }

            let client = Client::from_conf(client_config.build());

            Ok(Self { client, config })
        }

        /// Create from environment variables
        pub async fn from_env() -> SecretResult<Self> {
            let config = AwsSecretsConfig {
                region: std::env::var("AWS_REGION").ok(),
                prefix: std::env::var("AWS_SECRETS_PREFIX").ok(),
                endpoint_url: std::env::var("AWS_SECRETS_ENDPOINT").ok(),
            };
            Self::new(config).await
        }

        /// Get the full secret name with prefix
        fn get_secret_name(&self, name: &str) -> String {
            match &self.config.prefix {
                Some(prefix) => format!("{}{}", prefix, name),
                None => name.to_string(),
            }
        }
    }

    #[async_trait]
    impl SecretManager for AwsSecretManager {
        fn name(&self) -> &'static str {
            "aws"
        }

        async fn read_secret(&self, name: &str) -> SecretResult<Option<String>> {
            let secret_name = self.get_secret_name(name);
            debug!(secret_name = %secret_name, "Reading secret from AWS Secrets Manager");

            match self
                .client
                .get_secret_value()
                .secret_id(&secret_name)
                .send()
                .await
            {
                Ok(response) => {
                    if let Some(secret_string) = response.secret_string() {
                        Ok(Some(secret_string.to_string()))
                    } else if let Some(secret_binary) = response.secret_binary() {
                        // Try to convert binary to string
                        match String::from_utf8(secret_binary.as_ref().to_vec()) {
                            Ok(s) => Ok(Some(s)),
                            Err(_) => Err(SecretError::invalid_format(
                                "Secret is binary and cannot be converted to string",
                            )),
                        }
                    } else {
                        Ok(None)
                    }
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("ResourceNotFoundException") {
                        debug!(secret_name = %secret_name, "Secret not found");
                        Ok(None)
                    } else if err_str.contains("AccessDeniedException") {
                        warn!(secret_name = %secret_name, "Access denied to secret");
                        Err(SecretError::access_denied(&secret_name))
                    } else if err_str.contains("InvalidRequestException") {
                        error!(secret_name = %secret_name, error = %err, "Invalid request");
                        Err(SecretError::invalid_format(err_str))
                    } else {
                        error!(secret_name = %secret_name, error = %err, "Failed to read secret");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }

        async fn write_secret(&self, name: &str, value: &str) -> SecretResult<()> {
            let secret_name = self.get_secret_name(name);
            debug!(secret_name = %secret_name, "Writing secret to AWS Secrets Manager");

            // First try to update existing secret
            match self
                .client
                .put_secret_value()
                .secret_id(&secret_name)
                .secret_string(value)
                .send()
                .await
            {
                Ok(_) => {
                    debug!(secret_name = %secret_name, "Secret updated successfully");
                    Ok(())
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("ResourceNotFoundException") {
                        // Secret doesn't exist, create it
                        debug!(secret_name = %secret_name, "Secret not found, creating new secret");
                        self.client
                            .create_secret()
                            .name(&secret_name)
                            .secret_string(value)
                            .send()
                            .await
                            .map_err(|e| SecretError::connection(e.to_string()))?;
                        Ok(())
                    } else if err_str.contains("AccessDeniedException") {
                        Err(SecretError::access_denied(&secret_name))
                    } else {
                        error!(secret_name = %secret_name, error = %err, "Failed to write secret");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }

        async fn delete_secret(&self, name: &str) -> SecretResult<()> {
            let secret_name = self.get_secret_name(name);
            debug!(secret_name = %secret_name, "Deleting secret from AWS Secrets Manager");

            match self
                .client
                .delete_secret()
                .secret_id(&secret_name)
                .force_delete_without_recovery(true)
                .send()
                .await
            {
                Ok(_) => {
                    debug!(secret_name = %secret_name, "Secret deleted successfully");
                    Ok(())
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("ResourceNotFoundException") {
                        // Already deleted, treat as success
                        Ok(())
                    } else if err_str.contains("AccessDeniedException") {
                        Err(SecretError::access_denied(&secret_name))
                    } else {
                        error!(secret_name = %secret_name, error = %err, "Failed to delete secret");
                        Err(SecretError::connection(err_str))
                    }
                }
            }
        }

        async fn list_secrets(
            &self,
            options: &ListSecretsOptions,
        ) -> SecretResult<ListSecretsResult> {
            debug!("Listing secrets from AWS Secrets Manager");

            let mut request = self.client.list_secrets();

            if let Some(max) = options.max_results {
                request = request.max_results(max as i32);
            }

            if let Some(ref token) = options.next_token {
                request = request.next_token(token);
            }

            // Apply prefix filter
            let filter_prefix = match (&self.config.prefix, &options.prefix) {
                (Some(config_prefix), Some(opt_prefix)) => {
                    Some(format!("{}{}", config_prefix, opt_prefix))
                }
                (Some(config_prefix), None) => Some(config_prefix.clone()),
                (None, Some(opt_prefix)) => Some(opt_prefix.clone()),
                (None, None) => None,
            };

            if let Some(ref prefix) = filter_prefix {
                request = request.filters(
                    aws_sdk_secretsmanager::types::Filter::builder()
                        .key(aws_sdk_secretsmanager::types::FilterNameStringType::Name)
                        .values(prefix)
                        .build(),
                );
            }

            match request.send().await {
                Ok(response) => {
                    let secrets = response
                        .secret_list()
                        .iter()
                        .filter_map(|s| {
                            s.name().map(|name| {
                                let mut metadata = SecretMetadata::new(name);
                                if let Some(arn) = s.arn() {
                                    metadata.tags.insert("arn".to_string(), arn.to_string());
                                }
                                if let Some(created) = s.created_date() {
                                    metadata.created_at = Some(created.secs());
                                }
                                if let Some(updated) = s.last_changed_date() {
                                    metadata.updated_at = Some(updated.secs());
                                }
                                metadata
                            })
                        })
                        .collect();

                    Ok(ListSecretsResult {
                        secrets,
                        next_token: response.next_token().map(|s| s.to_string()),
                    })
                }
                Err(err) => {
                    error!(error = %err, "Failed to list secrets");
                    Err(SecretError::connection(err.to_string()))
                }
            }
        }
    }

    impl std::fmt::Debug for AwsSecretManager {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("AwsSecretManager")
                .field("config", &self.config)
                .finish()
        }
    }
}

#[cfg(feature = "aws-secrets")]
pub use implementation::*;

// Stub implementation when feature is disabled
#[cfg(not(feature = "aws-secrets"))]
mod stub {
    use crate::core::traits::secret_manager::SecretError;

    /// AWS Secrets Manager configuration (stub)
    #[derive(Debug, Clone, Default)]
    pub struct AwsSecretsConfig;

    impl AwsSecretsConfig {
        pub fn new() -> Self {
            Self
        }
    }

    /// AWS Secrets Manager client (stub)
    #[derive(Debug)]
    pub struct AwsSecretManager;

    impl AwsSecretManager {
        pub async fn new(_config: AwsSecretsConfig) -> Result<Self, SecretError> {
            Err(SecretError::config(
                "AWS Secrets Manager support not enabled. Enable the 'aws-secrets' feature.",
            ))
        }
    }
}

#[cfg(not(feature = "aws-secrets"))]
pub use stub::*;

#[cfg(all(test, feature = "aws-secrets"))]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = AwsSecretsConfig::new()
            .region("us-east-1")
            .prefix("prod/")
            .endpoint_url("http://localhost:4566");

        assert_eq!(config.region, Some("us-east-1".to_string()));
        assert_eq!(config.prefix, Some("prod/".to_string()));
        assert_eq!(config.endpoint_url, Some("http://localhost:4566".to_string()));
    }
}
