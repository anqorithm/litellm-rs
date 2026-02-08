//! DeepSeek Configuration
//!
//! Configuration

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

// Configuration
define_provider_config!(DeepSeekConfig {});

impl DeepSeekConfig {
    /// Create
    pub fn from_env() -> Self {
        Self::new("deepseek")
    }
}

// implementationProviderConfig trait
impl ProviderConfig for DeepSeekConfig {
    fn validate(&self) -> Result<(), String> {
        self.base.validate("deepseek")
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
    fn test_deepseek_config() {
        let config = DeepSeekConfig::new("deepseek");
        assert_eq!(
            config.base.api_base,
            Some("https://api.deepseek.com".to_string())
        );
        assert_eq!(config.base.timeout, 60);
    }

    #[test]
    fn test_deepseek_config_default_retries() {
        let config = DeepSeekConfig::new("deepseek");
        assert_eq!(config.base.max_retries, 3);
    }

    #[test]
    fn test_deepseek_config_from_env() {
        let config = DeepSeekConfig::from_env();
        assert!(config.base.api_base.is_some());
    }

    #[test]
    fn test_deepseek_validate_missing_api_key() {
        let config = DeepSeekConfig::new("deepseek");
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }

    #[test]
    fn test_deepseek_validate_success() {
        let mut config = DeepSeekConfig::new("deepseek");
        config.base.api_key = Some("sk-test-key".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_trait() {
        let mut config = DeepSeekConfig::new("deepseek");
        config.base.api_key = Some("test-key".to_string());

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("https://api.deepseek.com"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 3);
    }

    #[test]
    fn test_deepseek_config_custom_api_base() {
        let mut config = DeepSeekConfig::new("deepseek");
        config.base.api_base = Some("https://custom.deepseek.com".to_string());
        assert_eq!(config.api_base(), Some("https://custom.deepseek.com"));
    }

    #[test]
    fn test_deepseek_config_custom_timeout() {
        let mut config = DeepSeekConfig::new("deepseek");
        config.base.timeout = 120;
        assert_eq!(config.timeout(), std::time::Duration::from_secs(120));
    }
}
