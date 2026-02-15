//! AIML API Configuration

use crate::define_provider_config;

define_provider_config!(AimlConfig, env_key: "AIML_API_KEY", provider: "aiml_api");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::provider::ProviderConfig;

    #[test]
    fn test_default_config() {
        let config = AimlConfig::default();
        assert_eq!(config.base.timeout, 60);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AimlConfig::default();
        assert!(config.validate().is_err());

        config.base.api_key = Some("test-key".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_builder() {
        let config = AimlConfig::new("aiml_api")
            .with_api_key("test-key")
            .with_base_url("https://custom.api.com")
            .with_timeout(120);

        assert_eq!(config.base.api_key.as_deref(), Some("test-key"));
        assert_eq!(
            config.base.api_base.as_deref(),
            Some("https://custom.api.com")
        );
        assert_eq!(config.base.timeout, 120);
    }
}
