//! Baseten Provider Configuration
//!
//! Configuration for Baseten API access including authentication and model settings.

use crate::core::traits::ProviderConfig;
use std::sync::LazyLock;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Default API base URL for Baseten Model API
pub const BASETEN_API_BASE: &str = "https://inference.baseten.co/v1";

/// Regex for matching dedicated deployment model IDs
static DEDICATED_DEPLOYMENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9]{8}$").unwrap());

/// Baseten provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasetenConfig {
    /// API key for Baseten authentication
    pub api_key: Option<String>,

    /// API base URL (default: <https://inference.baseten.co/v1>)
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

impl Default for BasetenConfig {
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

impl ProviderConfig for BasetenConfig {
    fn validate(&self) -> Result<(), String> {
        // API key can come from environment variable
        if self.api_key.is_none() && std::env::var("BASETEN_API_KEY").is_err() {
            return Err(
                "Baseten API key not provided and BASETEN_API_KEY environment variable not set"
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

impl BasetenConfig {
    /// Get API key with environment variable fallback
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("BASETEN_API_KEY").ok())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var("BASETEN_API_BASE").ok())
            .unwrap_or_else(|| BASETEN_API_BASE.to_string())
    }

    /// Check if model is a dedicated deployment (8-character alphanumeric code)
    pub fn is_dedicated_deployment(model: &str) -> bool {
        // Remove 'baseten/' prefix if present
        let model_id = model.strip_prefix("baseten/").unwrap_or(model);

        // Check if it's an 8-character alphanumeric code
        DEDICATED_DEPLOYMENT_REGEX.is_match(model_id)
    }

    /// Get the appropriate API base URL for the given model
    pub fn get_api_base_for_model(model: &str) -> String {
        if Self::is_dedicated_deployment(model) {
            // Extract the model ID (remove 'baseten/' prefix if present)
            let model_id = model.strip_prefix("baseten/").unwrap_or(model);
            format!(
                "https://model-{}.api.baseten.co/environments/production/sync/v1",
                model_id
            )
        } else {
            // Use Model API
            BASETEN_API_BASE.to_string()
        }
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseten_config_default() {
        let config = BasetenConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 30);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
    }

    #[test]
    fn test_baseten_config_get_api_base_default() {
        let config = BasetenConfig::default();
        assert_eq!(config.get_api_base(), BASETEN_API_BASE);
    }

    #[test]
    fn test_baseten_config_get_api_base_custom() {
        let config = BasetenConfig {
            api_base: Some("https://custom.baseten.co".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_base(), "https://custom.baseten.co");
    }

    #[test]
    fn test_baseten_config_get_api_key() {
        let config = BasetenConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_key(), Some("test-key".to_string()));
    }

    #[test]
    fn test_is_dedicated_deployment() {
        // 8-character alphanumeric should be dedicated deployment
        assert!(BasetenConfig::is_dedicated_deployment("abc12345"));
        assert!(BasetenConfig::is_dedicated_deployment("baseten/abc12345"));
        assert!(BasetenConfig::is_dedicated_deployment("ABCD1234"));

        // Non-8-character codes should not be dedicated
        assert!(!BasetenConfig::is_dedicated_deployment("llama-3.1-70b"));
        assert!(!BasetenConfig::is_dedicated_deployment("ab12345")); // 7 chars
        assert!(!BasetenConfig::is_dedicated_deployment("abc123456")); // 9 chars
        assert!(!BasetenConfig::is_dedicated_deployment("abc-1234")); // has hyphen
    }

    #[test]
    fn test_get_api_base_for_model() {
        // Model API for regular models
        assert_eq!(
            BasetenConfig::get_api_base_for_model("llama-3.1-70b"),
            BASETEN_API_BASE
        );

        // Dedicated deployment URL
        assert_eq!(
            BasetenConfig::get_api_base_for_model("abc12345"),
            "https://model-abc12345.api.baseten.co/environments/production/sync/v1"
        );

        // With baseten/ prefix
        assert_eq!(
            BasetenConfig::get_api_base_for_model("baseten/xyz98765"),
            "https://model-xyz98765.api.baseten.co/environments/production/sync/v1"
        );
    }

    #[test]
    fn test_baseten_config_provider_config_trait() {
        let config = BasetenConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://custom.baseten.co".to_string()),
            timeout: 60,
            max_retries: 5,
            ..Default::default()
        };

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("https://custom.baseten.co"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_baseten_config_validation_with_key() {
        let config = BasetenConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_baseten_config_validation_zero_timeout() {
        let config = BasetenConfig {
            api_key: Some("test-key".to_string()),
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_baseten_config_serialization() {
        let config = BasetenConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://custom.baseten.co".to_string()),
            timeout: 45,
            max_retries: 2,
            debug: true,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "test-key");
        assert_eq!(json["api_base"], "https://custom.baseten.co");
        assert_eq!(json["timeout"], 45);
        assert_eq!(json["max_retries"], 2);
        assert_eq!(json["debug"], true);
    }

    #[test]
    fn test_baseten_config_deserialization() {
        let json = r#"{
            "api_key": "test-key",
            "timeout": 60,
            "debug": true
        }"#;

        let config: BasetenConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
    }
}
