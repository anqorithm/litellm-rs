//! Environment Variable Secret Manager
//!
//! Reads secrets from environment variables.

use async_trait::async_trait;
use std::env;

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
}

impl EnvSecretManager {
    /// Create a new environment secret manager
    pub fn new() -> Self {
        Self { prefix: None }
    }

    /// Create with a prefix for environment variable names
    ///
    /// For example, with prefix "LITELLM_", reading "API_KEY" will look for "LITELLM_API_KEY"
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: Some(prefix.into()),
        }
    }

    /// Get the full environment variable name with prefix
    fn get_env_name(&self, name: &str) -> String {
        match &self.prefix {
            Some(prefix) => format!("{}{}", prefix, name),
            None => name.to_string(),
        }
    }
}

#[async_trait]
impl SecretManager for EnvSecretManager {
    fn name(&self) -> &'static str {
        "env"
    }

    async fn read_secret(&self, name: &str) -> SecretResult<Option<String>> {
        let env_name = self.get_env_name(name);
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
        // Note: This only sets the variable for the current process
        // SAFETY: We're setting an environment variable which is safe in single-threaded
        // contexts or when properly synchronized. This is primarily for testing/development.
        unsafe {
            env::set_var(&env_name, value);
        }
        Ok(())
    }

    async fn delete_secret(&self, name: &str) -> SecretResult<()> {
        let env_name = self.get_env_name(name);
        // SAFETY: We're removing an environment variable which is safe in single-threaded
        // contexts or when properly synchronized. This is primarily for testing/development.
        unsafe {
            env::remove_var(&env_name);
        }
        Ok(())
    }

    async fn list_secrets(&self, options: &ListSecretsOptions) -> SecretResult<ListSecretsResult> {
        let mut secrets = Vec::new();

        for (key, _) in env::vars() {
            // Filter by prefix if configured
            let matches_manager_prefix = match &self.prefix {
                Some(prefix) => key.starts_with(prefix),
                None => true,
            };

            if !matches_manager_prefix {
                continue;
            }

            // Filter by user-provided prefix
            let matches_filter_prefix = match &options.prefix {
                Some(filter_prefix) => {
                    let key_without_manager_prefix = match &self.prefix {
                        Some(prefix) => key.strip_prefix(prefix).unwrap_or(&key),
                        None => &key,
                    };
                    key_without_manager_prefix.starts_with(filter_prefix)
                }
                None => true,
            };

            if !matches_filter_prefix {
                continue;
            }

            // Remove manager prefix from the key for the result
            let secret_name = match &self.prefix {
                Some(prefix) => key.strip_prefix(prefix).unwrap_or(&key).to_string(),
                None => key,
            };

            secrets.push(SecretMetadata::new(secret_name));

            // Check max results
            if let Some(max) = options.max_results {
                if secrets.len() >= max {
                    break;
                }
            }
        }

        Ok(ListSecretsResult {
            secrets,
            next_token: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // Helper to safely set env var in tests
    unsafe fn set_test_env(key: &str, value: &str) {
        unsafe { env::set_var(key, value) };
    }

    // Helper to safely remove env var in tests
    unsafe fn remove_test_env(key: &str) {
        unsafe { env::remove_var(key) };
    }

    #[tokio::test]
    async fn test_read_existing_secret() {
        let manager = EnvSecretManager::new();
        unsafe { set_test_env("TEST_SECRET_READ", "test_value") };

        let result = manager.read_secret("TEST_SECRET_READ").await.unwrap();
        assert_eq!(result, Some("test_value".to_string()));

        unsafe { remove_test_env("TEST_SECRET_READ") };
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

        assert_eq!(env::var("TEST_SECRET_WRITE").unwrap(), "written_value");

        unsafe { remove_test_env("TEST_SECRET_WRITE") };
    }

    #[tokio::test]
    async fn test_delete_secret() {
        let manager = EnvSecretManager::new();
        unsafe { set_test_env("TEST_SECRET_DELETE", "to_delete") };

        manager.delete_secret("TEST_SECRET_DELETE").await.unwrap();

        assert!(env::var("TEST_SECRET_DELETE").is_err());
    }

    #[tokio::test]
    async fn test_with_prefix() {
        let manager = EnvSecretManager::with_prefix("LITELLM_");
        unsafe { set_test_env("LITELLM_API_KEY", "prefixed_value") };

        let result = manager.read_secret("API_KEY").await.unwrap();
        assert_eq!(result, Some("prefixed_value".to_string()));

        unsafe { remove_test_env("LITELLM_API_KEY") };
    }

    #[tokio::test]
    async fn test_exists() {
        let manager = EnvSecretManager::new();
        unsafe { set_test_env("TEST_SECRET_EXISTS", "exists") };

        assert!(manager.exists("TEST_SECRET_EXISTS").await.unwrap());
        assert!(!manager.exists("NONEXISTENT_SECRET_67890").await.unwrap());

        unsafe { remove_test_env("TEST_SECRET_EXISTS") };
    }

    #[tokio::test]
    async fn test_list_secrets_with_prefix() {
        let manager = EnvSecretManager::with_prefix("TEST_LIST_");
        unsafe {
            set_test_env("TEST_LIST_SECRET1", "value1");
            set_test_env("TEST_LIST_SECRET2", "value2");
        }

        let result = manager
            .list_secrets(&ListSecretsOptions::new())
            .await
            .unwrap();

        assert!(result.secrets.len() >= 2);
        let names: Vec<_> = result.secrets.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"SECRET1"));
        assert!(names.contains(&"SECRET2"));

        unsafe {
            remove_test_env("TEST_LIST_SECRET1");
            remove_test_env("TEST_LIST_SECRET2");
        }
    }

    #[tokio::test]
    async fn test_name() {
        let manager = EnvSecretManager::new();
        assert_eq!(manager.name(), "env");
    }
}
