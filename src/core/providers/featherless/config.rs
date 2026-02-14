//! Featherless Configuration

use crate::define_provider_config;

define_provider_config!(FeatherlessConfig, provider: "featherless");

impl FeatherlessConfig {
    /// Get the effective API base URL
    pub fn get_api_base(&self) -> String {
        self.base
            .api_base
            .clone()
            .unwrap_or_else(|| "https://api.featherless.ai/v1".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::provider::ProviderConfig;

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
