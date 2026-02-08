//! Volcengine Configuration
//!
//! Configuration for ByteDance's Volcengine AI platform

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

// Configuration using the provider config macro
define_provider_config!(VolcengineConfig {});

impl VolcengineConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self::new("volcengine")
    }

    /// Create with custom API base (for different regions)
    pub fn with_region(mut self, region: &str) -> Self {
        self.base.api_base = Some(match region {
            "cn-beijing" => "https://ark.cn-beijing.volces.com/api/v3".to_string(),
            "cn-shanghai" => "https://ark.cn-shanghai.volces.com/api/v3".to_string(),
            _ => format!("https://ark.{}.volces.com/api/v3", region),
        });
        self
    }
}

// Implement ProviderConfig trait
impl ProviderConfig for VolcengineConfig {
    fn validate(&self) -> Result<(), String> {
        self.base.validate("volcengine")
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
    fn test_volcengine_config() {
        let config = VolcengineConfig::new("volcengine");
        assert!(config.base.api_base.is_some());
        assert_eq!(config.base.timeout, 60);
    }

    #[test]
    fn test_volcengine_config_default_retries() {
        let config = VolcengineConfig::new("volcengine");
        assert_eq!(config.base.max_retries, 3);
    }

    #[test]
    fn test_volcengine_config_from_env() {
        let config = VolcengineConfig::from_env();
        assert!(config.base.api_base.is_some());
    }

    #[test]
    fn test_volcengine_validate_missing_api_key() {
        let config = VolcengineConfig::new("volcengine");
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }

    #[test]
    fn test_volcengine_validate_success() {
        let mut config = VolcengineConfig::new("volcengine");
        config.base.api_key = Some("test-key".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_volcengine_with_region() {
        let config = VolcengineConfig::new("volcengine").with_region("cn-beijing");
        assert_eq!(
            config.api_base(),
            Some("https://ark.cn-beijing.volces.com/api/v3")
        );
    }

    #[test]
    fn test_provider_config_trait() {
        let mut config = VolcengineConfig::new("volcengine");
        config.base.api_key = Some("test-key".to_string());

        assert_eq!(config.api_key(), Some("test-key"));
        assert!(config.api_base().is_some());
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 3);
    }
}
