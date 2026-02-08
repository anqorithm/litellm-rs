//! Stability AI Configuration
//!
//! Configuration for Stability AI image generation provider.

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

// Use the define_provider_config macro to create StabilityConfig
define_provider_config!(StabilityConfig {});

impl StabilityConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::new("stability");
        // Override default API base for Stability AI
        config.base.api_base = Some("https://api.stability.ai".to_string());
        config
    }

    /// Create configuration with API key
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        let mut config = Self::from_env();
        config.base.api_key = Some(api_key.into());
        config
    }
}

impl ProviderConfig for StabilityConfig {
    fn validate(&self) -> Result<(), String> {
        if self.base.api_key.is_none() {
            return Err("Stability AI API key is required".to_string());
        }
        Ok(())
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
    fn test_stability_config_default() {
        let config = StabilityConfig::from_env();
        assert_eq!(
            config.base.api_base,
            Some("https://api.stability.ai".to_string())
        );
        assert_eq!(config.base.timeout, 60);
    }

    #[test]
    fn test_stability_config_with_api_key() {
        let config = StabilityConfig::with_api_key("sk-test-key");
        assert_eq!(config.base.api_key, Some("sk-test-key".to_string()));
    }

    #[test]
    fn test_stability_validate_missing_api_key() {
        let config = StabilityConfig::from_env();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }

    #[test]
    fn test_stability_validate_success() {
        let config = StabilityConfig::with_api_key("sk-test-key");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_trait() {
        let config = StabilityConfig::with_api_key("test-key");
        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("https://api.stability.ai"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 3);
    }

    #[test]
    fn test_stability_config_custom_timeout() {
        let mut config = StabilityConfig::with_api_key("test-key");
        config.base.timeout = 120;
        assert_eq!(config.timeout(), std::time::Duration::from_secs(120));
    }
}
