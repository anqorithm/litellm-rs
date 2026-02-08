//! Fireworks AI Provider Configuration
//!
//! Configuration for Fireworks AI API access including authentication and model settings.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Fireworks AI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FireworksConfig {
    /// API key for Fireworks AI authentication
    pub api_key: Option<String>,

    /// API base URL (default: <https://api.fireworks.ai/inference/v1>)
    pub api_base: Option<String>,

    /// Account ID for model listing (optional)
    pub account_id: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Whether to enable debug logging
    #[serde(default)]
    pub debug: bool,

    /// Whether to disable the transform=inline feature for images
    #[serde(default)]
    pub disable_transform_inline: bool,
}

impl Default for FireworksConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: None,
            account_id: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug: false,
            disable_transform_inline: false,
        }
    }
}

impl ProviderConfig for FireworksConfig {
    fn validate(&self) -> Result<(), String> {
        // API key can come from environment variable
        if self.api_key.is_none()
            && std::env::var("FIREWORKS_API_KEY").is_err()
            && std::env::var("FIREWORKS_AI_API_KEY").is_err()
            && std::env::var("FIREWORKSAI_API_KEY").is_err()
            && std::env::var("FIREWORKS_AI_TOKEN").is_err()
        {
            return Err(
                "Fireworks API key not provided and no environment variable set (FIREWORKS_API_KEY, FIREWORKS_AI_API_KEY, FIREWORKSAI_API_KEY, or FIREWORKS_AI_TOKEN)"
                    .to_string(),
            );
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

impl FireworksConfig {
    /// Get API key with environment variable fallback
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key.clone().or_else(|| {
            std::env::var("FIREWORKS_API_KEY")
                .or_else(|_| std::env::var("FIREWORKS_AI_API_KEY"))
                .or_else(|_| std::env::var("FIREWORKSAI_API_KEY"))
                .or_else(|_| std::env::var("FIREWORKS_AI_TOKEN"))
                .ok()
        })
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var("FIREWORKS_API_BASE").ok())
            .unwrap_or_else(|| "https://api.fireworks.ai/inference/v1".to_string())
    }

    /// Get account ID with environment variable fallback
    pub fn get_account_id(&self) -> Option<String> {
        self.account_id
            .clone()
            .or_else(|| std::env::var("FIREWORKS_ACCOUNT_ID").ok())
    }
}

fn default_timeout() -> u64 {
    120
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fireworks_config_default() {
        let config = FireworksConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert!(config.account_id.is_none());
        assert_eq!(config.timeout, 120);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
        assert!(!config.disable_transform_inline);
    }

    #[test]
    fn test_fireworks_config_get_api_base_default() {
        let config = FireworksConfig::default();
        assert_eq!(
            config.get_api_base(),
            "https://api.fireworks.ai/inference/v1"
        );
    }

    #[test]
    fn test_fireworks_config_get_api_base_custom() {
        let config = FireworksConfig {
            api_base: Some("https://custom.fireworks.ai".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_base(), "https://custom.fireworks.ai");
    }

    #[test]
    fn test_fireworks_config_get_api_key() {
        let config = FireworksConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_key(), Some("test-key".to_string()));
    }

    #[test]
    fn test_fireworks_config_provider_config_trait() {
        let config = FireworksConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://custom.fireworks.ai".to_string()),
            timeout: 60,
            max_retries: 5,
            ..Default::default()
        };

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("https://custom.fireworks.ai"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_fireworks_config_validation_with_key() {
        let config = FireworksConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_fireworks_config_validation_zero_timeout() {
        let config = FireworksConfig {
            api_key: Some("test-key".to_string()),
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_fireworks_config_serialization() {
        let config = FireworksConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://custom.fireworks.ai".to_string()),
            account_id: Some("account-123".to_string()),
            timeout: 45,
            max_retries: 2,
            debug: true,
            disable_transform_inline: true,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "test-key");
        assert_eq!(json["api_base"], "https://custom.fireworks.ai");
        assert_eq!(json["account_id"], "account-123");
        assert_eq!(json["timeout"], 45);
        assert_eq!(json["max_retries"], 2);
        assert_eq!(json["debug"], true);
        assert_eq!(json["disable_transform_inline"], true);
    }

    #[test]
    fn test_fireworks_config_deserialization() {
        let json = r#"{
            "api_key": "test-key",
            "timeout": 60,
            "debug": true
        }"#;

        let config: FireworksConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
    }
}
