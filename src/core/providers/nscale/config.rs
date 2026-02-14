//! Nscale Configuration
//!
//! Configuration for Nscale AI inference platform

use crate::define_provider_config;

define_provider_config!(NscaleConfig, provider: "nscale");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::provider::ProviderConfig;

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
