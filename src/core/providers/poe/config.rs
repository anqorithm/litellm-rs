//! Poe Provider Configuration

use crate::define_standalone_provider_config;

define_standalone_provider_config!(PoeConfig,
    provider: "Poe",
    env_prefix: "POE",
    default_base_url: "https://api.poe.com/v1",
    default_timeout: 30,
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poe_config_default() {
        let config = PoeConfig::default();
        assert_eq!(config.timeout, 30);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_poe_config_get_api_base() {
        let config = PoeConfig::default();
        assert_eq!(config.get_api_base(), "https://api.poe.com/v1");
    }

    #[test]
    fn test_poe_config_with_api_key() {
        let config = PoeConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert!(crate::core::traits::provider::ProviderConfig::validate(&config).is_ok());
    }
}
