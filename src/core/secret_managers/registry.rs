//! Secret Manager Registry
//!
//! Manages multiple secret manager backends and provides unified access.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::core::traits::secret_manager::{BoxedSecretManager, SecretError, SecretResult};

/// Registry for managing multiple secret manager backends
///
/// # Example
///
/// ```rust,ignore
/// use litellm_rs::core::secret_managers::{SecretManagerRegistry, EnvSecretManager, FileSecretManager};
///
/// let registry = SecretManagerRegistry::new();
/// registry.register("env", Arc::new(EnvSecretManager::new())).await;
/// registry.register("file", Arc::new(FileSecretManager::new("/etc/secrets"))).await;
///
/// // Read from specific backend
/// let api_key = registry.read_secret("env", "OPENAI_API_KEY").await?;
///
/// // Or use the default backend
/// registry.set_default("env").await;
/// let api_key = registry.read_secret_default("OPENAI_API_KEY").await?;
/// ```
pub struct SecretManagerRegistry {
    /// Registered secret managers
    managers: RwLock<HashMap<String, BoxedSecretManager>>,
    /// Default manager name
    default_manager: RwLock<Option<String>>,
}

impl SecretManagerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            managers: RwLock::new(HashMap::new()),
            default_manager: RwLock::new(None),
        }
    }

    /// Create a registry with default managers (env and file)
    pub fn with_defaults() -> Self {
        // Note: Registration is async, so we can't do it in the constructor
        // Users should call register_defaults() after creation
        Self::new()
    }

    /// Register default managers (env and file)
    pub async fn register_defaults(&self) {
        use super::{EnvSecretManager, FileSecretManager};

        self.register("env", Arc::new(EnvSecretManager::new()))
            .await;
        self.register("file", Arc::new(FileSecretManager::default()))
            .await;
        self.set_default("env").await;
    }

    /// Register a secret manager
    pub async fn register(&self, name: impl Into<String>, manager: BoxedSecretManager) {
        let name = name.into();
        debug!("Registering secret manager: {}", name);
        let mut managers = self.managers.write().await;
        managers.insert(name, manager);
    }

    /// Unregister a secret manager
    pub async fn unregister(&self, name: &str) -> bool {
        let mut managers = self.managers.write().await;
        let removed = managers.remove(name).is_some();
        if removed {
            debug!("Unregistered secret manager: {}", name);
        }
        removed
    }

    /// Set the default manager
    pub async fn set_default(&self, name: impl Into<String>) {
        let name = name.into();
        debug!("Setting default secret manager: {}", name);
        let mut default = self.default_manager.write().await;
        *default = Some(name);
    }

    /// Get the default manager name
    pub async fn get_default(&self) -> Option<String> {
        self.default_manager.read().await.clone()
    }

    /// List registered manager names
    pub async fn list_managers(&self) -> Vec<String> {
        let managers = self.managers.read().await;
        managers.keys().cloned().collect()
    }

    /// Check if a manager is registered
    pub async fn has_manager(&self, name: &str) -> bool {
        let managers = self.managers.read().await;
        managers.contains_key(name)
    }

    /// Read a secret from a specific manager
    pub async fn read_secret(&self, manager_name: &str, secret_name: &str) -> SecretResult<Option<String>> {
        let managers = self.managers.read().await;
        let manager = managers
            .get(manager_name)
            .ok_or_else(|| SecretError::config(format!("Unknown secret manager: {}", manager_name)))?;

        manager.read_secret(secret_name).await
    }

    /// Read a secret from the default manager
    pub async fn read_secret_default(&self, secret_name: &str) -> SecretResult<Option<String>> {
        let default_name = self
            .default_manager
            .read()
            .await
            .clone()
            .ok_or_else(|| SecretError::config("No default secret manager configured"))?;

        self.read_secret(&default_name, secret_name).await
    }

    /// Write a secret to a specific manager
    pub async fn write_secret(
        &self,
        manager_name: &str,
        secret_name: &str,
        value: &str,
    ) -> SecretResult<()> {
        let managers = self.managers.read().await;
        let manager = managers
            .get(manager_name)
            .ok_or_else(|| SecretError::config(format!("Unknown secret manager: {}", manager_name)))?;

        manager.write_secret(secret_name, value).await
    }

    /// Delete a secret from a specific manager
    pub async fn delete_secret(&self, manager_name: &str, secret_name: &str) -> SecretResult<()> {
        let managers = self.managers.read().await;
        let manager = managers
            .get(manager_name)
            .ok_or_else(|| SecretError::config(format!("Unknown secret manager: {}", manager_name)))?;

        manager.delete_secret(secret_name).await
    }

    /// Resolve a secret reference string
    ///
    /// Format: `${secret:manager:name}` or `${secret:name}` (uses default manager)
    ///
    /// # Examples
    ///
    /// - `${secret:env:OPENAI_API_KEY}` - Read from env manager
    /// - `${secret:file:api-key}` - Read from file manager
    /// - `${secret:OPENAI_API_KEY}` - Read from default manager
    pub async fn resolve_reference(&self, reference: &str) -> SecretResult<Option<String>> {
        // Check if it's a secret reference
        if !reference.starts_with("${secret:") || !reference.ends_with('}') {
            return Ok(Some(reference.to_string()));
        }

        // Extract the inner part
        let inner = &reference[9..reference.len() - 1]; // Remove "${secret:" and "}"

        // Parse manager:name or just name
        let (manager_name, secret_name) = if let Some(pos) = inner.find(':') {
            let manager = &inner[..pos];
            let name = &inner[pos + 1..];
            (Some(manager), name)
        } else {
            (None, inner)
        };

        match manager_name {
            Some(manager) => self.read_secret(manager, secret_name).await,
            None => self.read_secret_default(secret_name).await,
        }
    }

    /// Resolve all secret references in a string
    ///
    /// Replaces all `${secret:...}` patterns with their values.
    pub async fn resolve_all_references(&self, input: &str) -> SecretResult<String> {
        let mut result = input.to_string();
        let mut start = 0;

        while let Some(ref_start) = result[start..].find("${secret:") {
            let abs_start = start + ref_start;
            if let Some(ref_end) = result[abs_start..].find('}') {
                let abs_end = abs_start + ref_end + 1;
                let reference = &result[abs_start..abs_end];

                match self.resolve_reference(reference).await? {
                    Some(value) => {
                        result.replace_range(abs_start..abs_end, &value);
                        start = abs_start + value.len();
                    }
                    None => {
                        return Err(SecretError::not_found(reference));
                    }
                }
            } else {
                break;
            }
        }

        Ok(result)
    }
}

impl Default for SecretManagerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::secret_managers::EnvSecretManager;
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
    async fn test_register_and_read() {
        let registry = SecretManagerRegistry::new();
        registry
            .register("env", Arc::new(EnvSecretManager::new()))
            .await;

        unsafe { set_test_env("TEST_REGISTRY_SECRET", "test_value") };

        let result = registry
            .read_secret("env", "TEST_REGISTRY_SECRET")
            .await
            .unwrap();
        assert_eq!(result, Some("test_value".to_string()));

        unsafe { remove_test_env("TEST_REGISTRY_SECRET") };
    }

    #[tokio::test]
    async fn test_default_manager() {
        let registry = SecretManagerRegistry::new();
        registry
            .register("env", Arc::new(EnvSecretManager::new()))
            .await;
        registry.set_default("env").await;

        unsafe { set_test_env("TEST_DEFAULT_SECRET", "default_value") };

        let result = registry
            .read_secret_default("TEST_DEFAULT_SECRET")
            .await
            .unwrap();
        assert_eq!(result, Some("default_value".to_string()));

        unsafe { remove_test_env("TEST_DEFAULT_SECRET") };
    }

    #[tokio::test]
    async fn test_unknown_manager() {
        let registry = SecretManagerRegistry::new();

        let result = registry.read_secret("unknown", "secret").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_no_default_manager() {
        let registry = SecretManagerRegistry::new();

        let result = registry.read_secret_default("secret").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_resolve_reference_with_manager() {
        let registry = SecretManagerRegistry::new();
        registry
            .register("env", Arc::new(EnvSecretManager::new()))
            .await;

        unsafe { set_test_env("TEST_REF_SECRET", "resolved_value") };

        let result = registry
            .resolve_reference("${secret:env:TEST_REF_SECRET}")
            .await
            .unwrap();
        assert_eq!(result, Some("resolved_value".to_string()));

        unsafe { remove_test_env("TEST_REF_SECRET") };
    }

    #[tokio::test]
    async fn test_resolve_reference_default_manager() {
        let registry = SecretManagerRegistry::new();
        registry
            .register("env", Arc::new(EnvSecretManager::new()))
            .await;
        registry.set_default("env").await;

        unsafe { set_test_env("TEST_REF_DEFAULT", "default_resolved") };

        let result = registry
            .resolve_reference("${secret:TEST_REF_DEFAULT}")
            .await
            .unwrap();
        assert_eq!(result, Some("default_resolved".to_string()));

        unsafe { remove_test_env("TEST_REF_DEFAULT") };
    }

    #[tokio::test]
    async fn test_resolve_non_reference() {
        let registry = SecretManagerRegistry::new();

        let result = registry
            .resolve_reference("plain_string")
            .await
            .unwrap();
        assert_eq!(result, Some("plain_string".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_all_references() {
        let registry = SecretManagerRegistry::new();
        registry
            .register("env", Arc::new(EnvSecretManager::new()))
            .await;

        unsafe {
            set_test_env("TEST_ALL_KEY1", "value1");
            set_test_env("TEST_ALL_KEY2", "value2");
        }

        let input = "api_key=${secret:env:TEST_ALL_KEY1}&other=${secret:env:TEST_ALL_KEY2}";
        let result = registry.resolve_all_references(input).await.unwrap();
        assert_eq!(result, "api_key=value1&other=value2");

        unsafe {
            remove_test_env("TEST_ALL_KEY1");
            remove_test_env("TEST_ALL_KEY2");
        }
    }

    #[tokio::test]
    async fn test_list_managers() {
        let registry = SecretManagerRegistry::new();
        registry
            .register("env", Arc::new(EnvSecretManager::new()))
            .await;
        registry
            .register("env2", Arc::new(EnvSecretManager::new()))
            .await;

        let managers = registry.list_managers().await;
        assert_eq!(managers.len(), 2);
        assert!(managers.contains(&"env".to_string()));
        assert!(managers.contains(&"env2".to_string()));
    }

    #[tokio::test]
    async fn test_unregister() {
        let registry = SecretManagerRegistry::new();
        registry
            .register("env", Arc::new(EnvSecretManager::new()))
            .await;

        assert!(registry.has_manager("env").await);

        let removed = registry.unregister("env").await;
        assert!(removed);
        assert!(!registry.has_manager("env").await);
    }
}
