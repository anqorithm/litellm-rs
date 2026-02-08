//! Perplexity Configuration
//!
//! Configuration for Perplexity AI provider

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

// Configuration using the macro
define_provider_config!(PerplexityConfig {});

impl PerplexityConfig {
    /// Create configuration from environment
    pub fn from_env() -> Self {
        Self::new("perplexity")
    }

    /// Get the effective API base URL
    pub fn get_api_base(&self) -> String {
        self.base
            .api_base
            .clone()
            .unwrap_or_else(|| "https://api.perplexity.ai".to_string())
    }
}

// Implementation of ProviderConfig trait
impl ProviderConfig for PerplexityConfig {
    fn validate(&self) -> Result<(), String> {
        self.base.validate("perplexity")
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
    fn test_perplexity_config() {
        let config = PerplexityConfig::new("perplexity");
        // Default API base from BaseConfig::for_provider returns default for unknown providers
        assert!(config.base.api_base.is_some());
        assert_eq!(config.base.timeout, 60);
    }

    #[test]
    fn test_perplexity_config_default_retries() {
        let config = PerplexityConfig::new("perplexity");
        assert_eq!(config.base.max_retries, 3);
    }

    #[test]
    fn test_perplexity_config_from_env() {
        let config = PerplexityConfig::from_env();
        assert!(config.base.api_base.is_some() || config.base.api_key.is_none());
    }

    #[test]
    fn test_perplexity_validate_missing_api_key() {
        let config = PerplexityConfig::new("perplexity");
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }

    #[test]
    fn test_perplexity_validate_success() {
        let mut config = PerplexityConfig::new("perplexity");
        config.base.api_key = Some("pplx-test-key".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_trait() {
        let mut config = PerplexityConfig::new("perplexity");
        config.base.api_key = Some("test-key".to_string());

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 3);
    }

    #[test]
    fn test_perplexity_config_custom_api_base() {
        let mut config = PerplexityConfig::new("perplexity");
        config.base.api_base = Some("https://custom.perplexity.ai".to_string());
        assert_eq!(config.get_api_base(), "https://custom.perplexity.ai");
    }

    #[test]
    fn test_perplexity_config_custom_timeout() {
        let mut config = PerplexityConfig::new("perplexity");
        config.base.timeout = 120;
        assert_eq!(config.timeout(), std::time::Duration::from_secs(120));
    }

    #[test]
    fn test_perplexity_get_api_base_default() {
        let mut config = PerplexityConfig::new("perplexity");
        config.base.api_base = None;
        assert_eq!(config.get_api_base(), "https://api.perplexity.ai");
    }
}
