//! Lambda Labs AI Provider Configuration
//!
//! Configuration for Lambda Labs API access including authentication and model settings.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Default API base URL for Lambda Labs
const DEFAULT_API_BASE: &str = "https://api.lambdalabs.com/v1";

/// Environment variable name for Lambda Labs API key
const ENV_API_KEY: &str = "LAMBDA_API_KEY";

/// Environment variable name for Lambda Labs API base URL
const ENV_API_BASE: &str = "LAMBDA_API_BASE";

/// Lambda Labs AI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaAIConfig {
    /// API key for Lambda Labs authentication
    pub api_key: Option<String>,

    /// API base URL (default: <https://api.lambdalabs.com/v1>)
    pub api_base: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Whether to enable debug logging
    #[serde(default)]
    pub debug: bool,
}

impl Default for LambdaAIConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug: false,
        }
    }
}

impl ProviderConfig for LambdaAIConfig {
    fn validate(&self) -> Result<(), String> {
        // API key can come from environment variable
        if self.api_key.is_none() && std::env::var(ENV_API_KEY).is_err() {
            return Err(format!(
                "Lambda Labs API key not provided and {} environment variable not set",
                ENV_API_KEY
            ));
        }

        // Validate timeout
        if self.timeout == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        self.api_base.as_deref()
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl LambdaAIConfig {
    /// Create a new configuration with API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            ..Default::default()
        }
    }

    /// Get API key with environment variable fallback
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var(ENV_API_KEY).ok())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var(ENV_API_BASE).ok())
            .unwrap_or_else(|| DEFAULT_API_BASE.to_string())
    }

    /// Set the API base URL
    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = Some(api_base.into());
        self
    }

    /// Set the timeout in seconds
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Enable debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
}

fn default_timeout() -> u64 {
    120 // Lambda Labs models may take longer due to GPU inference
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let config = LambdaAIConfig::default().with_api_base("https://custom.lambda.com");
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
            .with_api_base("https://custom.lambda.com")
            .with_timeout(60)
            .with_max_retries(5);

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("https://custom.lambda.com"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_lambda_ai_config_validation_no_key() {
        // Clear env var for this test
        // SAFETY: This is safe in a single-threaded test context
        unsafe { std::env::remove_var(ENV_API_KEY) };
        let config = LambdaAIConfig::default();
        assert!(config.validate().is_err());
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

    #[test]
    fn test_lambda_ai_config_serialization() {
        let config = LambdaAIConfig::new("test-key")
            .with_api_base("https://custom.lambda.com")
            .with_timeout(45)
            .with_max_retries(2)
            .with_debug(true);

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "test-key");
        assert_eq!(json["api_base"], "https://custom.lambda.com");
        assert_eq!(json["timeout"], 45);
        assert_eq!(json["max_retries"], 2);
        assert_eq!(json["debug"], true);
    }

    #[test]
    fn test_lambda_ai_config_deserialization() {
        let json = r#"{
            "api_key": "test-key",
            "timeout": 60,
            "debug": true
        }"#;

        let config: LambdaAIConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
    }
}
