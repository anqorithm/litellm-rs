//! Heroku Configuration
//!
//! Configuration for Heroku AI Inference API

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

/// Provider name constant
pub const PROVIDER_NAME: &str = "heroku";

/// Default API base URL for Heroku Inference
/// Note: In production, this is typically set via INFERENCE_URL environment variable
pub const DEFAULT_API_BASE: &str = "https://inference.heroku.com/v1";

// Configuration
// app_name: Heroku app name (optional, for multi-app setups)
define_provider_config!(HerokuConfig {
    app_name: Option<String> = None,
});

impl HerokuConfig {
    /// Create configuration from environment variables
    ///
    /// Heroku uses these environment variables:
    /// - INFERENCE_KEY: API key for authentication
    /// - INFERENCE_URL: Base URL for API calls
    /// - INFERENCE_MODEL_ID: Default model identifier
    ///
    /// Also supports standard HEROKU_API_KEY for compatibility
    pub fn from_env() -> Self {
        let mut config = Self::new(PROVIDER_NAME);

        // Try Heroku-specific env vars first, then fall back to standard naming
        if config.base.api_key.is_none() {
            config.base.api_key = std::env::var("INFERENCE_KEY").ok();
        }

        // Use INFERENCE_URL if available
        if let Ok(url) = std::env::var("INFERENCE_URL") {
            config.base.api_base = Some(url);
        }

        config
    }

    /// Create configuration with API key
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        let mut config = Self::new(PROVIDER_NAME);
        config.base.api_key = Some(api_key.into());
        config
    }

    /// Set the Heroku app name
    pub fn with_app_name(mut self, app_name: impl Into<String>) -> Self {
        self.app_name = Some(app_name.into());
        self
    }

    /// Set the API base URL
    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.base.api_base = Some(api_base.into());
        self
    }
}

// Implement ProviderConfig trait
impl ProviderConfig for HerokuConfig {
    fn validate(&self) -> Result<(), String> {
        self.base.validate(PROVIDER_NAME)
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
    fn test_heroku_config() {
        let config = HerokuConfig::new(PROVIDER_NAME);
        assert_eq!(config.base.timeout, 60);
        assert_eq!(config.app_name, None);
    }

    #[test]
    fn test_heroku_config_default_retries() {
        let config = HerokuConfig::new(PROVIDER_NAME);
        assert_eq!(config.base.max_retries, 3);
    }

    #[test]
    fn test_heroku_config_from_env() {
        let config = HerokuConfig::from_env();
        // Just check it doesn't panic
        assert!(config.base.timeout > 0);
    }

    #[test]
    fn test_heroku_validate_missing_api_key() {
        let config = HerokuConfig::new(PROVIDER_NAME);
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }

    #[test]
    fn test_heroku_validate_success() {
        let mut config = HerokuConfig::new(PROVIDER_NAME);
        config.base.api_key = Some("test-api-key".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_trait() {
        let mut config = HerokuConfig::new(PROVIDER_NAME);
        config.base.api_key = Some("test-key".to_string());

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 3);
    }

    #[test]
    fn test_heroku_config_with_api_key() {
        let config = HerokuConfig::with_api_key("my-api-key");
        assert_eq!(config.api_key(), Some("my-api-key"));
    }

    #[test]
    fn test_heroku_config_with_app_name() {
        let config = HerokuConfig::with_api_key("key").with_app_name("my-heroku-app");
        assert_eq!(config.app_name, Some("my-heroku-app".to_string()));
    }

    #[test]
    fn test_heroku_config_with_api_base() {
        let config = HerokuConfig::with_api_key("key")
            .with_api_base("https://custom.inference.heroku.com/v1");
        assert_eq!(
            config.api_base(),
            Some("https://custom.inference.heroku.com/v1")
        );
    }

    #[test]
    fn test_heroku_config_custom_timeout() {
        let mut config = HerokuConfig::new(PROVIDER_NAME);
        config.base.timeout = 120;
        assert_eq!(config.timeout(), std::time::Duration::from_secs(120));
    }
}
