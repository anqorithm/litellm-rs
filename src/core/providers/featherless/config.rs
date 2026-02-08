//! Featherless Configuration

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

define_provider_config!(FeatherlessConfig {});

impl FeatherlessConfig {
    /// Create configuration from environment
    pub fn from_env() -> Self {
        Self::new("featherless")
    }

    /// Get the effective API base URL
    pub fn get_api_base(&self) -> String {
        self.base
            .api_base
            .clone()
            .unwrap_or_else(|| "https://api.featherless.ai/v1".to_string())
    }
}

impl ProviderConfig for FeatherlessConfig {
    fn validate(&self) -> Result<(), String> {
        self.base.validate("featherless")
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
    fn test_featherless_config() {
        let config = FeatherlessConfig::new("featherless");
        assert!(config.base.api_base.is_some());
    }

    #[test]
    fn test_featherless_validate_missing_api_key() {
        let config = FeatherlessConfig::new("featherless");
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_featherless_validate_success() {
        let mut config = FeatherlessConfig::new("featherless");
        config.base.api_key = Some("fl-test-key".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_featherless_get_api_base_default() {
        let mut config = FeatherlessConfig::new("featherless");
        config.base.api_base = None;
        assert_eq!(config.get_api_base(), "https://api.featherless.ai/v1");
    }
}
