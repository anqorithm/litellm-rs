//! Environment Variable Secret Manager
//!
//! Reads secrets from environment variables.

use async_trait::async_trait;
use std::{
    collections::{HashMap, HashSet},
    env,
    sync::{Arc, RwLock},
};

use crate::core::traits::secret_manager::{
    ListSecretsOptions, ListSecretsResult, SecretError, SecretManager, SecretMetadata, SecretResult,
};

/// Secret manager that reads from environment variables
///
/// # Example
///
/// ```rust,ignore
/// use litellm_rs::core::secret_managers::EnvSecretManager;
///
/// let manager = EnvSecretManager::new();
/// let api_key = manager.read_secret("OPENAI_API_KEY").await?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct EnvSecretManager {
    /// Optional prefix for environment variable names
    prefix: Option<String>,
    /// Process-local overrides to avoid mutating global environment at runtime
    overrides: Arc<RwLock<HashMap<String, Option<String>>>>,
}

impl EnvSecretManager {
    /// Create a new environment secret manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a prefix for environment variable names
    ///
    /// For example, with prefix "LITELLM_", reading "API_KEY" will look for "LITELLM_API_KEY"
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: Some(prefix.into()),
            ..Self::default()
        }
    }

    /// Get the full environment variable name with prefix
    fn get_env_name(&self, name: &str) -> String {
        match &self.prefix {
            Some(prefix) => format!("{}{}", prefix, name),
            None => name.to_string(),
        }
    }

    fn get_override(&self, env_name: &str) -> Option<Option<String>> {
        let overrides = self
            .overrides
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        overrides.get(env_name).cloned()
    }

    fn set_override(&self, env_name: String, value: Option<String>) {
        let mut overrides = self
            .overrides
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        overrides.insert(env_name, value);
    }

    fn visible_secret_name(&self, key: &str, filter_prefix: Option<&str>) -> Option<String> {
        let key_without_manager_prefix = match &self.prefix {
            Some(prefix) => key.strip_prefix(prefix)?,
            None => key,
        };

        if let Some(filter_prefix) = filter_prefix {
            if !key_without_manager_prefix.starts_with(filter_prefix) {
                return None;
            }
        }

        Some(key_without_manager_prefix.to_string())
    }
}

#[async_trait]
impl SecretManager for EnvSecretManager {
    fn name(&self) -> &'static str {
        "env"
    }

    async fn read_secret(&self, name: &str) -> SecretResult<Option<String>> {
        let env_name = self.get_env_name(name);

        if let Some(override_value) = self.get_override(&env_name) {
            return Ok(override_value);
        }

        match env::var(&env_name) {
            Ok(value) => Ok(Some(value)),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(env::VarError::NotUnicode(_)) => Err(SecretError::invalid_format(format!(
                "Environment variable {} contains invalid UTF-8",
                env_name
            ))),
        }
    }

    async fn write_secret(&self, name: &str, value: &str) -> SecretResult<()> {
        let env_name = self.get_env_name(name);
        self.set_override(env_name, Some(value.to_string()));
        Ok(())
    }

    async fn delete_secret(&self, name: &str) -> SecretResult<()> {
        let env_name = self.get_env_name(name);
        self.set_override(env_name, None);
        Ok(())
    }

    async fn list_secrets(&self, options: &ListSecretsOptions) -> SecretResult<ListSecretsResult> {
        let filter_prefix = options.prefix.as_deref();
        let mut secret_names = HashSet::new();

        for (key, _) in env::vars() {
            if let Some(secret_name) = self.visible_secret_name(key.as_str(), filter_prefix) {
                secret_names.insert(secret_name);
            }
        }

        let overrides = self
            .overrides
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        for (key, value) in overrides.iter() {
            if let Some(secret_name) = self.visible_secret_name(key.as_str(), filter_prefix) {
                if value.is_some() {
                    secret_names.insert(secret_name);
                } else {
                    secret_names.remove(secret_name.as_str());
                }
            }
        }

        let mut secret_names = secret_names.into_iter().collect::<Vec<_>>();
        secret_names.sort_unstable();

        if let Some(max) = options.max_results {
            secret_names.truncate(max);
        }

        let secrets = secret_names
            .into_iter()
            .map(SecretMetadata::new)
            .collect::<Vec<_>>();

        Ok(ListSecretsResult {
            secrets,
            next_token: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_existing_secret() {
        let manager = EnvSecretManager::new();
        manager
            .write_secret("TEST_SECRET_READ", "test_value")
            .await
            .unwrap();

        let result = manager.read_secret("TEST_SECRET_READ").await.unwrap();
        assert_eq!(result, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_read_nonexistent_secret() {
        let manager = EnvSecretManager::new();

        let result = manager
            .read_secret("NONEXISTENT_SECRET_12345")
            .await
            .unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_write_secret() {
        let manager = EnvSecretManager::new();

        manager
            .write_secret("TEST_SECRET_WRITE", "written_value")
            .await
            .unwrap();

        let result = manager.read_secret("TEST_SECRET_WRITE").await.unwrap();
        assert_eq!(result, Some("written_value".to_string()));
    }

    #[tokio::test]
    async fn test_delete_secret() {
        let manager = EnvSecretManager::new();
        manager
            .write_secret("TEST_SECRET_DELETE", "to_delete")
            .await
            .unwrap();

        manager.delete_secret("TEST_SECRET_DELETE").await.unwrap();

        assert!(
            manager
                .read_secret("TEST_SECRET_DELETE")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_with_prefix() {
        let manager = EnvSecretManager::with_prefix("LITELLM_");
        manager
            .write_secret("API_KEY", "prefixed_value")
            .await
            .unwrap();

        let result = manager.read_secret("API_KEY").await.unwrap();
        assert_eq!(result, Some("prefixed_value".to_string()));
    }

    #[tokio::test]
    async fn test_exists() {
        let manager = EnvSecretManager::new();
        manager
            .write_secret("TEST_SECRET_EXISTS", "exists")
            .await
            .unwrap();

        assert!(manager.exists("TEST_SECRET_EXISTS").await.unwrap());
        assert!(!manager.exists("NONEXISTENT_SECRET_67890").await.unwrap());
    }

    #[tokio::test]
    async fn test_list_secrets_with_prefix() {
        let manager = EnvSecretManager::with_prefix("TEST_LIST_");
        manager.write_secret("SECRET1", "value1").await.unwrap();
        manager.write_secret("SECRET2", "value2").await.unwrap();

        let result = manager
            .list_secrets(&ListSecretsOptions::new())
            .await
            .unwrap();

        assert!(result.secrets.len() >= 2);
        let names: Vec<_> = result.secrets.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"SECRET1"));
        assert!(names.contains(&"SECRET2"));
    }

    #[tokio::test]
    async fn test_name() {
        let manager = EnvSecretManager::new();
        assert_eq!(manager.name(), "env");
    }
}
