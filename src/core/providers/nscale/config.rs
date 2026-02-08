//! Nscale Configuration
//!
//! Configuration for Nscale AI inference platform

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

// Configuration using the provider config macro
define_provider_config!(NscaleConfig {});

impl NscaleConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self::new("nscale")
    }
}

// Implement ProviderConfig trait
impl ProviderConfig for NscaleConfig {
    fn validate(&self) -> Result<(), String> {
        self.base.validate("nscale")
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
    fn test_nscale_config() {
        let config = NscaleConfig::new("nscale");
        assert!(config.base.api_base.is_some());
        assert_eq!(config.base.timeout, 60);
    }

    #[test]
    fn test_nscale_config_default_retries() {
        let config = NscaleConfig::new("nscale");
        assert_eq!(config.base.max_retries, 3);
    }

    #[test]
    fn test_nscale_config_from_env() {
        let config = NscaleConfig::from_env();
        assert!(config.base.api_base.is_some());
    }

    #[test]
    fn test_nscale_validate_missing_api_key() {
        let config = NscaleConfig::new("nscale");
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }

    #[test]
    fn test_nscale_validate_success() {
        let mut config = NscaleConfig::new("nscale");
        config.base.api_key = Some("test-key".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_trait() {
        let mut config = NscaleConfig::new("nscale");
        config.base.api_key = Some("test-key".to_string());

        assert_eq!(config.api_key(), Some("test-key"));
        assert!(config.api_base().is_some());
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 3);
    }
}
