//! Cerebras Configuration
//!
//! Configuration for Cerebras AI API

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

// Configuration
define_provider_config!(CerebrasConfig {});

impl CerebrasConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self::new("cerebras")
    }
}

// Implement ProviderConfig trait
impl ProviderConfig for CerebrasConfig {
    fn validate(&self) -> Result<(), String> {
        self.base.validate("cerebras")
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
    fn test_cerebras_config() {
        let config = CerebrasConfig::new("cerebras");
        assert_eq!(
            config.base.api_base,
            Some("https://api.cerebras.ai/v1".to_string())
        );
        assert_eq!(config.base.timeout, 60);
    }

    #[test]
    fn test_cerebras_config_default_retries() {
        let config = CerebrasConfig::new("cerebras");
        assert_eq!(config.base.max_retries, 3);
    }

    #[test]
    fn test_cerebras_config_from_env() {
        let config = CerebrasConfig::from_env();
        assert!(config.base.api_base.is_some());
    }

    #[test]
    fn test_cerebras_validate_missing_api_key() {
        let config = CerebrasConfig::new("cerebras");
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }

    #[test]
    fn test_cerebras_validate_success() {
        let mut config = CerebrasConfig::new("cerebras");
        config.base.api_key = Some("test-api-key".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_trait() {
        let mut config = CerebrasConfig::new("cerebras");
        config.base.api_key = Some("test-key".to_string());

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("https://api.cerebras.ai/v1"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 3);
    }

    #[test]
    fn test_cerebras_config_custom_api_base() {
        let mut config = CerebrasConfig::new("cerebras");
        config.base.api_base = Some("https://custom.cerebras.ai".to_string());
        assert_eq!(config.api_base(), Some("https://custom.cerebras.ai"));
    }

    #[test]
    fn test_cerebras_config_custom_timeout() {
        let mut config = CerebrasConfig::new("cerebras");
        config.base.timeout = 120;
        assert_eq!(config.timeout(), std::time::Duration::from_secs(120));
    }
}
