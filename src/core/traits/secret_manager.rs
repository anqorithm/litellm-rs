//! Secret Manager trait definitions
//!
//! Provides unified interface for secret management across different backends
//! (environment variables, files, AWS Secrets Manager, HashiCorp Vault, etc.)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result type for secret manager operations
pub type SecretResult<T> = Result<T, SecretError>;

/// Secret manager error types
#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("Secret not found: {name}")]
    NotFound { name: String },

    #[error("Access denied to secret: {name}")]
    AccessDenied { name: String },

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Invalid secret format: {0}")]
    InvalidFormat(String),

    #[error("Secret expired: {name}")]
    Expired { name: String },

    #[error("Rate limited: retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },

    #[error("Secret manager error: {0}")]
    Other(String),
}

impl SecretError {
    pub fn not_found(name: impl Into<String>) -> Self {
        Self::NotFound { name: name.into() }
    }

    pub fn access_denied(name: impl Into<String>) -> Self {
        Self::AccessDenied { name: name.into() }
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Authentication(msg.into())
    }

    pub fn invalid_format(msg: impl Into<String>) -> Self {
        Self::InvalidFormat(msg.into())
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

/// Secret metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    /// Secret name
    pub name: String,
    /// Secret version (if supported)
    pub version: Option<String>,
    /// Creation timestamp (Unix seconds)
    pub created_at: Option<i64>,
    /// Last updated timestamp (Unix seconds)
    pub updated_at: Option<i64>,
    /// Expiration timestamp (Unix seconds)
    pub expires_at: Option<i64>,
    /// Custom tags/labels
    pub tags: HashMap<String, String>,
}

impl SecretMetadata {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            created_at: None,
            updated_at: None,
            expires_at: None,
            tags: HashMap::new(),
        }
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }
}

/// Secret value with optional metadata
#[derive(Debug, Clone)]
pub struct Secret {
    /// The secret value
    pub value: String,
    /// Optional metadata
    pub metadata: Option<SecretMetadata>,
}

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: SecretMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Options for reading secrets
#[derive(Debug, Clone, Default)]
pub struct ReadSecretOptions {
    /// Specific version to read (if supported)
    pub version: Option<String>,
    /// Whether to include metadata
    pub include_metadata: bool,
}

impl ReadSecretOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_metadata(mut self) -> Self {
        self.include_metadata = true;
        self
    }
}

/// Options for writing secrets
#[derive(Debug, Clone, Default)]
pub struct WriteSecretOptions {
    /// Description for the secret
    pub description: Option<String>,
    /// Tags/labels
    pub tags: HashMap<String, String>,
    /// Whether to overwrite if exists
    pub overwrite: bool,
}

impl WriteSecretOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    pub fn overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }
}

/// Options for listing secrets
#[derive(Debug, Clone, Default)]
pub struct ListSecretsOptions {
    /// Filter by prefix
    pub prefix: Option<String>,
    /// Maximum number of results
    pub max_results: Option<usize>,
    /// Pagination token
    pub next_token: Option<String>,
}

impl ListSecretsOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }

    pub fn next_token(mut self, token: impl Into<String>) -> Self {
        self.next_token = Some(token.into());
        self
    }
}

/// Result of listing secrets
#[derive(Debug, Clone)]
pub struct ListSecretsResult {
    /// Secret names/metadata
    pub secrets: Vec<SecretMetadata>,
    /// Token for next page (if more results available)
    pub next_token: Option<String>,
}

/// Core secret manager trait
#[async_trait]
pub trait SecretManager: Send + Sync {
    /// Get the secret manager name
    fn name(&self) -> &'static str;

    /// Read a secret by name
    async fn read_secret(&self, name: &str) -> SecretResult<Option<String>>;

    /// Read a secret with options
    async fn read_secret_with_options(
        &self,
        name: &str,
        options: &ReadSecretOptions,
    ) -> SecretResult<Option<Secret>> {
        // Default implementation ignores options
        let _ = options;
        match self.read_secret(name).await? {
            Some(value) => Ok(Some(Secret::new(value))),
            None => Ok(None),
        }
    }

    /// Write a secret
    async fn write_secret(&self, name: &str, value: &str) -> SecretResult<()>;

    /// Write a secret with options
    async fn write_secret_with_options(
        &self,
        name: &str,
        value: &str,
        options: &WriteSecretOptions,
    ) -> SecretResult<()> {
        // Default implementation ignores options
        let _ = options;
        self.write_secret(name, value).await
    }

    /// Delete a secret
    async fn delete_secret(&self, name: &str) -> SecretResult<()>;

    /// Rotate a secret (delete old, write new)
    async fn rotate_secret(&self, name: &str, new_value: &str) -> SecretResult<()> {
        // Default implementation: write overwrites
        self.write_secret(name, new_value).await
    }

    /// List secrets
    async fn list_secrets(&self, options: &ListSecretsOptions) -> SecretResult<ListSecretsResult>;

    /// Check if a secret exists
    async fn exists(&self, name: &str) -> SecretResult<bool> {
        Ok(self.read_secret(name).await?.is_some())
    }
}

/// Type alias for boxed secret manager
pub type BoxedSecretManager = std::sync::Arc<dyn SecretManager>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_error_constructors() {
        let err = SecretError::not_found("my-secret");
        assert!(matches!(err, SecretError::NotFound { .. }));
        assert!(err.to_string().contains("my-secret"));

        let err = SecretError::access_denied("restricted-secret");
        assert!(matches!(err, SecretError::AccessDenied { .. }));

        let err = SecretError::config("Invalid configuration");
        assert!(matches!(err, SecretError::Configuration(_)));

        let err = SecretError::connection("Connection refused");
        assert!(matches!(err, SecretError::Connection(_)));

        let err = SecretError::auth("Invalid credentials");
        assert!(matches!(err, SecretError::Authentication(_)));
    }

    #[test]
    fn test_secret_metadata_builder() {
        let metadata = SecretMetadata::new("api-key")
            .version("v1")
            .tag("env", "production")
            .tag("team", "platform");

        assert_eq!(metadata.name, "api-key");
        assert_eq!(metadata.version, Some("v1".to_string()));
        assert_eq!(metadata.tags.get("env"), Some(&"production".to_string()));
        assert_eq!(metadata.tags.get("team"), Some(&"platform".to_string()));
    }

    #[test]
    fn test_secret_builder() {
        let secret = Secret::new("super-secret-value")
            .with_metadata(SecretMetadata::new("my-secret").version("v2"));

        assert_eq!(secret.value, "super-secret-value");
        assert!(secret.metadata.is_some());
        assert_eq!(secret.metadata.unwrap().version, Some("v2".to_string()));
    }

    #[test]
    fn test_read_secret_options() {
        let options = ReadSecretOptions::new().version("v3").with_metadata();

        assert_eq!(options.version, Some("v3".to_string()));
        assert!(options.include_metadata);
    }

    #[test]
    fn test_write_secret_options() {
        let options = WriteSecretOptions::new()
            .description("API key for external service")
            .tag("service", "stripe")
            .overwrite(true);

        assert_eq!(
            options.description,
            Some("API key for external service".to_string())
        );
        assert_eq!(options.tags.get("service"), Some(&"stripe".to_string()));
        assert!(options.overwrite);
    }

    #[test]
    fn test_list_secrets_options() {
        let options = ListSecretsOptions::new()
            .prefix("prod/")
            .max_results(100)
            .next_token("abc123");

        assert_eq!(options.prefix, Some("prod/".to_string()));
        assert_eq!(options.max_results, Some(100));
        assert_eq!(options.next_token, Some("abc123".to_string()));
    }
}
