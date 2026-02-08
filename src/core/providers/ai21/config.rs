//! AI21 Configuration
//!
//! Configuration for AI21 Labs API

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

// Configuration
define_provider_config!(AI21Config {});

impl AI21Config {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self::new("ai21")
    }
}

// Implement ProviderConfig trait
impl ProviderConfig for AI21Config {
    fn validate(&self) -> Result<(), String> {
        self.base.validate("ai21")
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
    fn test_ai21_config() {
        let config = AI21Config::new("ai21");
        assert_eq!(
            config.base.api_base,
            Some("https://api.ai21.com/studio/v1".to_string())
        );
        assert_eq!(config.base.timeout, 60);
    }

    #[test]
    fn test_ai21_config_default_retries() {
        let config = AI21Config::new("ai21");
        assert_eq!(config.base.max_retries, 3);
    }

    #[test]
    fn test_ai21_config_from_env() {
        let config = AI21Config::from_env();
        assert!(config.base.api_base.is_some());
    }

    #[test]
    fn test_ai21_validate_missing_api_key() {
        let config = AI21Config::new("ai21");
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }

    #[test]
    fn test_ai21_validate_success() {
        let mut config = AI21Config::new("ai21");
        config.base.api_key = Some("test-api-key".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_trait() {
        let mut config = AI21Config::new("ai21");
        config.base.api_key = Some("test-key".to_string());

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("https://api.ai21.com/studio/v1"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 3);
    }

    #[test]
    fn test_ai21_config_custom_api_base() {
        let mut config = AI21Config::new("ai21");
        config.base.api_base = Some("https://custom.ai21.com".to_string());
        assert_eq!(config.api_base(), Some("https://custom.ai21.com"));
    }

    #[test]
    fn test_ai21_config_custom_timeout() {
        let mut config = AI21Config::new("ai21");
        config.base.timeout = 120;
        assert_eq!(config.timeout(), std::time::Duration::from_secs(120));
    }
}
