//! Nebius Configuration
//!
//! Configuration for Nebius AI cloud platform

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

// Configuration using the provider config macro
define_provider_config!(NebiusConfig {});

impl NebiusConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self::new("nebius")
    }

    /// Set custom folder ID (Nebius uses folder IDs for organization)
    pub fn with_folder_id(mut self, folder_id: &str) -> Self {
        self.base
            .headers
            .insert("x-folder-id".to_string(), folder_id.to_string());
        self
    }
}

// Implement ProviderConfig trait
impl ProviderConfig for NebiusConfig {
    fn validate(&self) -> Result<(), String> {
        self.base.validate("nebius")
    }

    fn api_key(&self) -> Option<&str> {
        self.base.api_key.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        self.base.api_base.as_deref()
    }

    fn timeout(&self) -> std::time::Duration {
        self.base.timeout_duration()
    }

    fn max_retries(&self) -> u32 {
        self.base.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nebius_config() {
        let config = NebiusConfig::new("nebius");
        assert!(config.base.api_base.is_some());
        assert_eq!(config.base.timeout, 60);
    }

    #[test]
    fn test_nebius_config_default_retries() {
        let config = NebiusConfig::new("nebius");
        assert_eq!(config.base.max_retries, 3);
    }

    #[test]
    fn test_nebius_config_from_env() {
        let config = NebiusConfig::from_env();
        assert!(config.base.api_base.is_some());
    }

    #[test]
    fn test_nebius_validate_missing_api_key() {
        let config = NebiusConfig::new("nebius");
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }

    #[test]
    fn test_nebius_validate_success() {
        let mut config = NebiusConfig::new("nebius");
        config.base.api_key = Some("test-key".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_nebius_with_folder_id() {
        let config = NebiusConfig::new("nebius").with_folder_id("my-folder-123");
        assert_eq!(
            config.base.headers.get("x-folder-id"),
            Some(&"my-folder-123".to_string())
        );
    }

    #[test]
    fn test_provider_config_trait() {
        let mut config = NebiusConfig::new("nebius");
        config.base.api_key = Some("test-key".to_string());

        assert_eq!(config.api_key(), Some("test-key"));
        assert!(config.api_base().is_some());
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 3);
    }
}
