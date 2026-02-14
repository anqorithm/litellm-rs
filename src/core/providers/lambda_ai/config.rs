//! Lambda Labs AI Provider Configuration
//!
//! Configuration for Lambda Labs API access including authentication and model settings.

use crate::define_standalone_provider_config;

define_standalone_provider_config!(LambdaAIConfig,
    provider: "Lambda Labs",
    env_prefix: "LAMBDA",
    default_base_url: "https://api.lambdalabs.com/v1",
    default_timeout: 120,
    extra_fields: { debug: bool = false },
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::provider::ProviderConfig;

    #[test]
    fn test_lambda_ai_config_default() {
        let config = LambdaAIConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 120);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
    }

    #[test]
    fn test_lambda_ai_config_new() {
        let config = LambdaAIConfig::new("test-key");
        assert_eq!(config.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_lambda_ai_config_get_api_base_default() {
        let config = LambdaAIConfig::default();
        assert_eq!(config.get_api_base(), "https://api.lambdalabs.com/v1");
    }

    #[test]
    fn test_lambda_ai_config_get_api_base_custom() {
        let config = LambdaAIConfig::default().with_base_url("https://custom.lambda.com");
        assert_eq!(config.get_api_base(), "https://custom.lambda.com");
    }

    #[test]
    fn test_lambda_ai_config_get_api_key() {
        let config = LambdaAIConfig::new("test-key");
        assert_eq!(config.get_api_key(), Some("test-key".to_string()));
    }

    #[test]
    fn test_lambda_ai_config_provider_config_trait() {
        let config = LambdaAIConfig::new("test-key")
            .with_base_url("https://custom.lambda.com")
            .with_timeout(60)
            .with_max_retries(5);

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("https://custom.lambda.com"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_lambda_ai_config_validation_with_key() {
        let config = LambdaAIConfig::new("test-key");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_lambda_ai_config_validation_zero_timeout() {
        let config = LambdaAIConfig::new("test-key").with_timeout(0);
        assert!(config.validate().is_err());
    }
}
